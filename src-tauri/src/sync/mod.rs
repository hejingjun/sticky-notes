use crate::db;
use serde::{Deserialize, Serialize};

/// Transport format: all notes serialized as a JSON array
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPayload {
    pub notes: Vec<db::Note>,
}

/// Create a directory on the WebDAV server (MKCOL).
/// Silently succeeds if the directory already exists (405/409 are OK).
pub async fn ensure_dir(url: &str, user: &str, password: &str) -> Result<(), String> {
    log::info!("[sync] MKCOL {url}");
    let client = reqwest::Client::new();
    let resp = client
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
    let client = reqwest::Client::new();
    let resp = client
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
pub async fn push(url: &str, user: &str, password: &str, notes: &[db::Note], etag: &str) -> Result<(), String> {
    log::info!("[sync] PUT {url} (笔记数: {}, etag: {etag})", notes.len());
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
    Ok(())
}

/// Entity-level last-writer-wins merge.
/// For same id: keep the one with higher `updated_at`.
/// If equal updated_at but different content -> flag as conflict.
/// Returns merged list and conflict marker flag.
pub fn merge(local: Vec<db::Note>, remote: Vec<db::Note>) -> (Vec<db::Note>, bool) {
    log::info!("[sync] merge: 本地 {} 条, 远程 {} 条", local.len(), remote.len());
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
                    log::warn!("[sync] 冲突: 笔记 {} 的本地和远程版本 timestamp 相同但内容不同", n.id);
                    has_conflict = true;
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
    (merged, has_conflict)
}
