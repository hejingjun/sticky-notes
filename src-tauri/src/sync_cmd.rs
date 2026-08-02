use crate::{commands::DbState, db, shortcuts::SettingsManager, sync};
use std::path::PathBuf;

/// SyncEngine — 深层模块，封装 WebDAV 同步的全部逻辑。
///
/// 接口小（sync/push/pull），实现隐藏了：
/// - WebDAV 配置获取
/// - URL 构建
/// - ETag 管理
///
/// 注意：Connection 不是 Send，不能跨 await 持有。
/// 所有 async I/O 在锁定连接之前完成。
pub struct SyncEngine {
    file_url: String,
    dir_url: String,
    user: String,
    password: String,
    etag_path: PathBuf,
}

impl SyncEngine {
    pub fn from_config(sm: &SettingsManager) -> Result<Self, String> {
        let cfg = sm.get_config();
        if cfg.webdav_url.is_empty() {
            return Err("请先配置 WebDAV 地址".into());
        }
        let base = cfg.webdav_url.trim_end_matches('/');
        let remote_dir = format!("{}/sticky-notes", base);
        let etag_path = dirs_next::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sticky-notes")
            .join(".sync_etag");

        Ok(Self {
            file_url: format!("{}/notes.json", remote_dir),
            dir_url: remote_dir,
            user: cfg.webdav_user.clone(),
            password: cfg.webdav_password.clone(),
            etag_path,
        })
    }

    fn read_etag(&self) -> String {
        std::fs::read_to_string(&self.etag_path).unwrap_or_default()
    }

    fn write_etag(&self, etag: &str) {
        if let Some(parent) = self.etag_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(&self.etag_path, etag) {
            log::error!("[sync] ETag 写入失败: {e}");
        }
    }

    /// 全量同步：拉取远程 → 合并 → 写入本地 → 推送远程
    /// 注意：db_state 在所有 async I/O 完成后才锁定
    pub async fn sync(&self, db_state: &DbState) -> Result<(Vec<db::Note>, bool), String> {
        // 1. 异步 I/O：确保目录 + 拉取远程（不持有连接）
        sync::ensure_dir(&self.dir_url, &self.user, &self.password).await?;
        let (remote_payload, remote_etag) = sync::fetch(&self.file_url, &self.user, &self.password).await?;

        // 2. 读取本地笔记（短暂锁定）
        let local_notes = {
            let conn = db_state.0.lock().map_err(|e| e.to_string())?;
            db::list_notes(&conn).map_err(|e| e.to_string())
        }?;

        // 3. 合并（纯内存操作）
        let (merged, has_conflict, conflicts) = sync::merge(local_notes, remote_payload.notes);

        // 4. 事务性写入（短暂锁定）
        {
            let conn = db_state.0.lock().map_err(|e| e.to_string())?;
            conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
            for n in &merged {
                if let Err(e) = db::upsert_note(&conn, n) {
                    let _ = conn.execute_batch("ROLLBACK");
                    return Err(e.to_string());
                }
            }
            // Save conflicting remote versions
            for (note_id, remote_note) in &conflicts {
                if let Err(e) = db::save_conflict(&conn, note_id, remote_note) {
                    log::warn!("[sync] 保存冲突版本失败: {e}");
                }
            }
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
        }

        // 5. 异步推送（不持有连接）
        let push_etag = sync::push(&self.file_url, &self.user, &self.password, &merged, &remote_etag).await?;
        let save_etag = if push_etag.is_empty() { &remote_etag } else { &push_etag };
        self.write_etag(save_etag);

        Ok((merged, has_conflict))
    }

    /// 仅推送本地到远程
    pub async fn push(&self, db_state: &DbState) -> Result<String, String> {
        // 1. 读取本地笔记（短暂锁定）
        let (local_notes, local_etag) = {
            let conn = db_state.0.lock().map_err(|e| e.to_string())?;
            let notes = db::list_notes(&conn).map_err(|e| e.to_string())?;
            let etag = self.read_etag();
            (notes, etag)
        };

        // 2. 异步推送（不持有连接）
        sync::ensure_dir(&self.dir_url, &self.user, &self.password).await?;
        let push_etag = sync::push(&self.file_url, &self.user, &self.password, &local_notes, &local_etag).await?;
        let save_etag = if push_etag.is_empty() { local_etag } else { push_etag };
        self.write_etag(&save_etag);

        Ok(format!("已上传 {} 条笔记", local_notes.len()))
    }

    /// 仅拉取远程到本地
    pub async fn pull(&self, db_state: &DbState) -> Result<String, String> {
        // 1. 异步拉取（不持有连接）
        sync::ensure_dir(&self.dir_url, &self.user, &self.password).await?;
        let (remote_payload, remote_etag) = sync::fetch(&self.file_url, &self.user, &self.password).await?;
        let remote_count = remote_payload.notes.len();

        // 2. 事务性写入（短暂锁定）
        {
            let conn = db_state.0.lock().map_err(|e| e.to_string())?;
            conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
            for note in &remote_payload.notes {
                if let Err(e) = db::upsert_note(&conn, note) {
                    let _ = conn.execute_batch("ROLLBACK");
                    return Err(e.to_string());
                }
            }
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
        }

        // 3. 保存 ETag
        self.write_etag(&remote_etag);

        Ok(format!("已下载 {} 条笔记", remote_count))
    }
}
