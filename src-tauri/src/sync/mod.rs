use crate::db;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;

/// Shared reqwest Client with 30s timeout and connection pooling.
/// Creating a new Client per request is expensive (new connection pool each time).
fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to create reqwest client")
    })
}

/// Transport format: all notes serialized as a JSON array
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPayload {
    pub notes: Vec<db::Note>,
}

/// Create a directory on the WebDAV server (MKCOL).
/// Silently succeeds if the directory already exists (405/409 are OK).
pub async fn ensure_dir(url: &str, user: &str, password: &str) -> Result<(), String> {
    log::info!("[sync] MKCOL {url}");
    let resp = client()
        .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), url)
        .basic_auth(user, Some(password))
        .send()
        .await
        .map_err(|e| {
            let msg = format!("WebDAV MKCOL 失败: {e}");
            log::error!("[sync] {msg}");
            msg
        })?;

    let status = resp.status();
    log::info!("[sync] MKCOL status: {status}");
    // 201 Created = success, 405/409 = already exists (safe to ignore)
    if status == reqwest::StatusCode::CREATED || status == 405 || status == 409 {
        return Ok(());
    }
    if !status.is_success() {
        let msg = format!("WebDAV MKCOL 返回 {status}");
        log::error!("[sync] {msg}");
        return Err(msg);
    }
    Ok(())
}

/// Fetch notes.json from WebDAV, parse, return (payload, etag).
pub async fn fetch(url: &str, user: &str, password: &str) -> Result<(SyncPayload, String), String> {
    log::info!("[sync] GET {url}");
    let resp = client()
        .get(url)
        .basic_auth(user, Some(password))
        .send()
        .await
        .map_err(|e| {
            let msg = format!("WebDAV GET 失败: {e}");
            log::error!("[sync] {msg}");
            msg
        })?;

    let status = resp.status();
    log::info!("[sync] GET status: {status}");
    // MKCOL handles directory creation; 404 on GET at file path
    // means first sync — return empty remote, not an error.
    if status == reqwest::StatusCode::NOT_FOUND {
        log::info!("[sync] 远程文件不存在（首次同步），视为空远程");
        return Ok((SyncPayload { notes: vec![] }, String::new()));
    }
    if !status.is_success() {
        let msg = format!("WebDAV GET 返回 {status}");
        log::error!("[sync] {msg}");
        return Err(msg);
    }

    let etag = resp
        .headers()
        .get("ETag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    log::info!("[sync] 远程 ETag: {etag}");
    if etag.is_empty() {
        log::warn!("[sync] 远程服务器未返回 ETag，增量同步不可用");
    }

    let text = resp.text().await.map_err(|e| {
        let msg = format!("读取响应失败: {e}");
        log::error!("[sync] {msg}");
        msg
    })?;
    log::info!("[sync] 响应体大小: {} bytes", text.len());

    let payload: SyncPayload = serde_json::from_str(&text).map_err(|e| {
        let msg = format!("解析 notes.json 失败: {e}");
        log::error!("[sync] {msg}");
        msg
    })?;
    log::info!("[sync] 解析到 {} 条远程笔记", payload.notes.len());

    Ok((payload, etag))
}

/// PUT notes.json to WebDAV with If-Match for ETag-based locking.
/// If etag is empty (first push), omit If-Match.
/// Returns the new ETag from the server response.
pub async fn push(url: &str, user: &str, password: &str, notes: &[db::Note], etag: &str) -> Result<String, String> {
    log::info!("[sync] PUT {url} (笔记数: {}, etag: {etag})", notes.len());
    let payload = SyncPayload {
        notes: notes.to_vec(),
    };
    let body = serde_json::to_string(&payload).map_err(|e| format!("序列化失败: {e}"))?;

    let mut req = client().put(url).basic_auth(user, Some(password));

    if !etag.is_empty() {
        req = req.header("If-Match", etag);
    }

    let resp = req
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| {
            let msg = format!("WebDAV PUT 失败: {e}");
            log::error!("[sync] {msg}");
            msg
        })?;

    let status = resp.status();
    log::info!("[sync] PUT status: {status}");
    if status == reqwest::StatusCode::PRECONDITION_FAILED {
        log::warn!("[sync] 412 Precondition Failed — 远程已被修改");
        return Err("同步冲突: 远程文件已被修改，请重新同步".into());
    }
    if !status.is_success() {
        let msg = format!("WebDAV PUT 返回 {status}");
        log::error!("[sync] {msg}");
        return Err(msg);
    }
    log::info!("[sync] PUT 成功");
    let new_etag = resp
        .headers()
        .get("ETag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    Ok(new_etag)
}

/// Entity-level last-writer-wins merge.
/// For same id: keep the one with higher `updated_at`.
/// If equal updated_at but different content -> flag as conflict.
/// Returns merged list, conflict flag, and list of (note_id, remote_version) for conflicts.
pub fn merge(local: Vec<db::Note>, remote: Vec<db::Note>) -> (Vec<db::Note>, bool, Vec<(String, db::Note)>) {
    log::info!("[sync] merge: 本地 {} 条, 远程 {} 条", local.len(), remote.len());
    let mut map: std::collections::HashMap<String, db::Note> = std::collections::HashMap::new();
    let mut has_conflict = false;
    let mut conflicts: Vec<(String, db::Note)> = Vec::new();

    for n in local {
        map.insert(n.id.clone(), n);
    }

    for n in remote {
        if let Some(existing) = map.get(&n.id) {
            if existing.updated_at == n.updated_at {
                // Same timestamp but different content? Flag conflict
                if existing.title != n.title
                    || existing.completed != n.completed
                    || existing.due_date != n.due_date
                    || existing.completed_at != n.completed_at
                    || existing.pinned != n.pinned
                    || existing.color != n.color
                    || existing.order != n.order
                    || existing.parent_id != n.parent_id
                    || existing.remind_at != n.remind_at
                {
                    log::warn!("[sync] 冲突: 笔记 {} 的本地和远程版本 timestamp 相同但内容不同", n.id);
                    has_conflict = true;
                    conflicts.push((n.id.clone(), n.clone()));
                }
                // Keep existing (local wins tie)
                continue;
            }
            if n.updated_at > existing.updated_at {
                log::debug!("[sync] 笔记 {} 远程版本更新，采用远程", n.id);
                map.insert(n.id.clone(), n);
            }
        } else {
            log::debug!("[sync] 笔记 {} 仅存在于远程，拉取到本地", n.id);
            map.insert(n.id.clone(), n);
        }
    }

    let mut merged: Vec<db::Note> = map.into_values().collect();
    merged.sort_by(|a, b| a.order.cmp(&b.order));
    log::info!("[sync] merge 完成: {} 条合并后笔记", merged.len());
    (merged, has_conflict, conflicts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_note(id: &str, title: &str, updated_at: i64) -> db::Note {
        db::Note {
            id: id.to_string(),
            title: title.to_string(),
            order: format!("{:010x}", updated_at),
            updated_at,
            ..Default::default()
        }
    }

    #[test]
    fn merge_empty_lists() {
        let (merged, conflict, _) = merge(vec![], vec![]);
        assert!(merged.is_empty());
        assert!(!conflict);
    }

    #[test]
    fn merge_local_only() {
        let local = vec![make_note("a", "local note", 100)];
        let (merged, conflict, _) = merge(local, vec![]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id, "a");
        assert!(!conflict);
    }

    #[test]
    fn merge_remote_only() {
        let remote = vec![make_note("b", "remote note", 200)];
        let (merged, conflict, _) = merge(vec![], remote);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].id, "b");
        assert!(!conflict);
    }

    #[test]
    fn merge_no_overlap() {
        let local = vec![make_note("a", "local", 100)];
        let remote = vec![make_note("b", "remote", 200)];
        let (merged, conflict, _) = merge(local, remote);
        assert_eq!(merged.len(), 2);
        assert!(!conflict);
    }

    #[test]
    fn merge_remote_newer_wins() {
        let local = vec![make_note("a", "old title", 100)];
        let remote = vec![make_note("a", "new title", 200)];
        let (merged, conflict, _) = merge(local, remote);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].title, "new title");
        assert!(!conflict);
    }

    #[test]
    fn merge_local_newer_wins() {
        let local = vec![make_note("a", "new title", 300)];
        let remote = vec![make_note("a", "old title", 100)];
        let (merged, conflict, _) = merge(local, remote);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].title, "new title");
        assert!(!conflict);
    }

    #[test]
    fn merge_same_timestamp_different_content_is_conflict() {
        let local = db::Note {
            id: "a".to_string(),
            title: "local version".to_string(),
            updated_at: 100,
            completed: false,
            ..Default::default()
        };
        let remote = db::Note {
            id: "a".to_string(),
            title: "remote version".to_string(),
            updated_at: 100,
            completed: true,
            ..Default::default()
        };
        let (merged, conflict, conflicts) = merge(vec![local], vec![remote]);
        assert_eq!(merged.len(), 1);
        assert!(conflict, "Same timestamp + different content should flag conflict");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].0, "a");
        // Local wins the tie
        assert_eq!(merged[0].title, "local version");
    }

    #[test]
    fn merge_same_timestamp_same_content_no_conflict() {
        let note = make_note("a", "identical", 100);
        let (merged, conflict, _) = merge(vec![note.clone()], vec![note]);
        assert_eq!(merged.len(), 1);
        assert!(!conflict);
    }

    #[test]
    fn merge_result_is_sorted_by_order() {
        let mut n1 = make_note("a", "first", 100);
        n1.order = "0000000001".to_string();
        let mut n2 = make_note("b", "second", 200);
        n2.order = "0000000002".to_string();
        let mut n3 = make_note("c", "third", 300);
        n3.order = "0000000000".to_string();
        // Remote provides n3 which has lowest order
        let (merged, _, _) = merge(vec![n1, n2], vec![n3]);
        assert_eq!(merged[0].id, "c"); // order 0000000000
        assert_eq!(merged[1].id, "a"); // order 0000000001
        assert_eq!(merged[2].id, "b"); // order 0000000002
    }

    #[test]
    fn merge_mixed_scenarios() {
        // local: a (local-only), b (will be overwritten by remote), c (newer local wins)
        let local = vec![
            make_note("a", "local only", 100),
            make_note("b", "local old", 100),
            make_note("c", "local newer", 300),
        ];
        // remote: b (newer remote), c (older remote), d (remote-only)
        let remote = vec![
            make_note("b", "remote newer", 200),
            make_note("c", "remote older", 100),
            make_note("d", "remote only", 200),
        ];
        let (merged, conflict, _) = merge(local, remote);
        assert_eq!(merged.len(), 4);
        assert!(!conflict);
        // Find each note
        let a = merged.iter().find(|n| n.id == "a").unwrap();
        let b = merged.iter().find(|n| n.id == "b").unwrap();
        let c = merged.iter().find(|n| n.id == "c").unwrap();
        let d = merged.iter().find(|n| n.id == "d").unwrap();
        assert_eq!(a.title, "local only");    // local-only, kept
        assert_eq!(b.title, "remote newer");  // remote updated_at 200 > 100
        assert_eq!(c.title, "local newer");   // local updated_at 300 > 100
        assert_eq!(d.title, "remote only");   // remote-only, pulled in
    }
}
