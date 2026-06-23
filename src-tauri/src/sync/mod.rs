use crate::db;
use serde::{Deserialize, Serialize};

/// Transport format: all notes serialized as a JSON array
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPayload {
    pub notes: Vec<db::Note>,
}

/// Fetch notes.json from WebDAV, parse, return (payload, etag).
pub async fn fetch(url: &str, user: &str, password: &str) -> Result<(SyncPayload, String), String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .basic_auth(user, Some(password))
        .send()
        .await
        .map_err(|e| format!("WebDAV GET 失败: {e}"))?;

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        // No remote file yet — safe to treat as empty
        return Ok((SyncPayload { notes: vec![] }, String::new()));
    }
    if !status.is_success() {
        return Err(format!("WebDAV GET 返回 {}", status));
    }

    let etag = resp
        .headers()
        .get("ETag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    if etag.is_empty() {
        eprintln!("[sticky-notes] 警告: 远程服务器未返回 ETag，增量同步不可用");
    }

    let text = resp.text().await.map_err(|e| format!("读取响应失败: {e}"))?;
    let payload: SyncPayload =
        serde_json::from_str(&text).map_err(|e| format!("解析 notes.json 失败: {e}"))?;

    Ok((payload, etag))
}

/// PUT notes.json to WebDAV with If-Match for ETag-based locking.
/// If etag is empty (first push), omit If-Match.
pub async fn push(url: &str, user: &str, password: &str, notes: &[db::Note], etag: &str) -> Result<(), String> {
    let payload = SyncPayload {
        notes: notes.to_vec(),
    };
    let body = serde_json::to_string(&payload).map_err(|e| format!("序列化失败: {e}"))?;

    let client = reqwest::Client::new();
    let mut req = client.put(url).basic_auth(user, Some(password));

    if !etag.is_empty() {
        req = req.header("If-Match", etag);
    }

    let resp = req
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("WebDAV PUT 失败: {e}"))?;

    let status = resp.status();
    if status == reqwest::StatusCode::PRECONDITION_FAILED {
        return Err("同步冲突: 远程文件已被修改，请重新同步".into());
    }
    if !status.is_success() {
        return Err(format!("WebDAV PUT 返回 {}", status));
    }
    Ok(())
}

/// Entity-level last-writer-wins merge.
/// For same id: keep the one with higher `updated_at`.
/// If equal updated_at but different content -> flag as conflict.
/// Returns merged list and conflict marker flag.
pub fn merge(local: Vec<db::Note>, remote: Vec<db::Note>) -> (Vec<db::Note>, bool) {
    let mut map: std::collections::HashMap<String, db::Note> = std::collections::HashMap::new();
    let mut has_conflict = false;

    for n in local {
        map.insert(n.id.clone(), n);
    }

    for n in remote {
        if let Some(existing) = map.get(&n.id) {
            if existing.updated_at == n.updated_at {
                // Same timestamp but different content? Flag conflict
                if existing.title != n.title
                    || existing.content != n.content
                    || existing.completed != n.completed
                    || existing.due_date != n.due_date
                {
                    has_conflict = true;
                }
                // Keep existing (local wins tie)
                continue;
            }
            if n.updated_at > existing.updated_at {
                map.insert(n.id.clone(), n);
            }
        } else {
            map.insert(n.id.clone(), n);
        }
    }

    let mut merged: Vec<db::Note> = map.into_values().collect();
    merged.sort_by(|a, b| a.order.cmp(&b.order));
    (merged, has_conflict)
}
