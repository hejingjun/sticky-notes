# 桌面便签 — 代码对抗审查报告

**审查日期**: 2026-06-23 | **审查人**: 资深架构师 | **严重程度**: 🔴严重 🟡中等 🟢轻微

---

## 1. 架构层面

### 🟡 1.1 全局可变状态：`static PEN` / `static ONTOP` + unsafe 裸指针传递

- **文件**: `src-tauri/src/lib.rs:12-13`, `src-tauri/src/win32/subclass.rs:12-22`
- **当前做法**: 在 `lib.rs` 中定义了两个全局 `static` AtomicBool (`PEN`, `ONTOP`)。`PEN` 的裸指针通过 `static mut PEN_PTR: *const AtomicBool` 传递给 Win32 子类化回调。
- **问题**: 全局可变状态使得测试不可能，且破坏了 Tauri 的状态管理模式（所有其他状态都通过 `.manage()` 注入）。`unsafe` 裸指针传递在重构时极易出错。
- **建议**: 将 `PEN` 和 `ONTOP` 放入一个 `AppState` 结构体，通过 `tauri::State` 管理。对于子类化回调，使用 `SetWindowSubclass` 的 `dwRefData` 参数传递一个 `Arc<AtomicBool>` 的堆分配指针。
- **决议**: **经讨论降级为 🟡，推迟到下个迭代。** dwRefData 方案涉及 Win32 子类化核心路径，当前方案经过多轮调试才稳定（CHANGELOG #19 记录了从 Raw WndProc 到 SetWindowSubclass 的艰难迁移）。在无测试覆盖时动这条路径风险过高。

### 🔴 1.2 `write_file` 命令是任意文件写入后门 — **已修复**

- **文件**: `src-tauri/src/commands.rs:62-64`, `src-tauri/capabilities/default.json:14-18`
- **当前做法**: 前端可以调用 `invoke("write_file", { path, contents })` 写入任意路径，capability 授予了 `fs:allow-write` scope `**`。
- **问题**: 前端代码中的任何 XSS 或依赖注入漏洞都可能导致攻击者写入任意文件（如覆盖 `~/.bashrc` 或启动目录中的 exe）。CSV 导出不需要一个通用的文件写入命令。
- **建议**: 删除 `write_file` 命令。在 CSV 导出场景中，要么使用 `tauri-plugin-dialog` 的 save + `tauri-plugin-fs` 的 scoped write，要么让 Rust 端直接使用 dialog 插件返回的路径写入。
- **为什么更好**: 关闭一个提权路径，遵循最小权限原则。
- **修复**: 已删除 `write_file` 命令，改用 `@tauri-apps/plugin-fs` 的 `writeTextFile`。capabilities 改为 `fs:allow-write-text-file`。

### 🟡 1.3 WebDAV 密码明文存储

- **文件**: `src-tauri/src/shortcuts.rs:11-12`, `src-tauri/src/shortcuts.rs:64-68`
- **当前做法**: `webdav_password` 以明文 JSON 写入 `%APPDATA%/sticky-notes/shortcuts.json`。
- **建议**: 使用 OS 原生凭据存储 — 在 Windows 上使用 `Credential Manager` API（通过 `windows-sys` 的 `Win32_Security_Credentials`），在 macOS 上使用 Keychain，在 Linux 上使用 `secret-service`。或者至少用 `tauri-plugin-store` 的加密能力。
- **为什么更好**: 密码不应该以明文存储在磁盘上。

### 🟢 1.4 单例 `useNotes` composable — 隐藏的全局状态

- **文件**: `src/composables/useNotes.ts:6`
- **当前做法**: `const notes = ref<Note[]>([])` 定义在模块顶层，`useNotes()` 返回对同一个 ref 的引用。
- **问题**: 虽然对当前单窗口应用可行，但这是隐式的全局状态。任何调用 `useNotes()` 的地方都共享同一个引用。如果将来需要多窗口，会导致状态不同步。
- **建议**: 将 ref 移入 `useNotes()` 函数内部，使用 Vue 的 `provide`/`inject` 或简单的 prop drilling。
- **决议**: **跳过。** 当前是单窗口应用，模块级单例是有意为之的设计模式。等将来需要多窗口时再重构。

### 🟡 1.5 `check_reminders` 全表扫描 — **已修复**

- **文件**: `src-tauri/src/commands.rs:42-59`
- **当前做法**: 加载所有未删除笔记到内存，遍历过滤出符合提醒条件的。
- **问题**: 随笔记数量增长，这个 O(n) 内存+CPU 操作每 30 秒执行一次会越来越慢。SQLite 完全可以直接用 WHERE 子句过滤，不需要加载全部数据。
- **建议**: 在 SQL 查询层过滤 — 直接写 `SELECT * FROM notes WHERE deleted_at IS NULL AND ((remind_at BETWEEN ? AND ?) OR (due_date <= ? AND completed = 0))`。
- **为什么更好**: 数据库层过滤，只传输需要的行，O(log n) vs O(n)。

---

## 2. 代码实现

### 🔴 2.1 事件监听器泄漏 — **已修复**

- **文件**: `src/App.vue:42`
- **当前做法**: 第 29 行 `unlistenPen = await listen("penetrate-changed", ...)` 注册了穿透监听器。第 41 行 `const unload = await listen("notes-reloaded", ...)` 创建了重载监听器。第 42 行 `unlistenPen = unload` **覆盖**了 `unlistenPen` 变量。
- **问题**: `penetrate-changed` 的取消监听函数被丢弃了。`onUnmounted` 中 `unlistenPen?.()` 取消的是 `notes-reloaded` 监听器（但 `unlistenTop` 没被覆盖，仍然能正确清理 `ontop-changed`）。**渗透模式的 Tauri 事件监听器永远不会被清理**。如果组件被重新挂载，会重复注册监听器。
- **建议**: 将 `unload` 存为单独的变量 `unlistenReload`，在 `onUnmounted` 中分别清理：
  ```ts
  let unlistenPen: (() => void) | null = null;
  let unlistenTop: (() => void) | null = null;
  let unlistenReload: (() => void) | null = null;
  ```
- **为什么更好**: 防止内存泄漏和重复事件监听。

### 🔴 2.2 CSV 导出存在 CSV 注入漏洞 — **已修复**

- **文件**: `src-tauri/src/commands.rs:67-85`
- **当前做法**: 手动拼接 CSV 字符串，只转义双引号。
- **问题**: 没有处理以 `=`、`+`、`-`、`@` 开头的字段值。这些在 Excel/Google Sheets 中会被解释为公式，构成 CSV 注入攻击（CWE-1236）。如果便签内容以 `=cmd|' /C calc'!A0` 开头，导出后用 Excel 打开可能执行恶意命令。
- **建议**: 对于以 `=`、`+`、`-`、`@` 开头的字段，在前面加单引号前缀 `'`，或使用 `csv` crate（`cargo add csv`）处理所有转义。
- **为什么更好**: 消除 CSV 注入风险。

### 🔴 2.3 数据库行访问使用位置索引（`SELECT *` + `row.get(N)`） — **已修复**

- **文件**: `src-tauri/src/db.rs:48-62`
- **当前做法**: `SELECT * FROM notes` 然后 `row.get(0)` 到 `row.get(13)` 按位置读取。
- **问题**: 如果任何人更改了列的顺序或添加/删除列，所有数据会在**不报错的情况下**被读到错误的字段里（因为 `rusqlite` 对不匹配不会有编译期检查）。这是一个"沉默的数据损坏"炸弹。
- **建议**: 使用命名列访问或 `rusqlite` 的 `query_map` with named params，或者使用 `#[derive(rusqlite::FromSql)]` 配合结构化查询。
- **为什么更好**: 列重排不会导致数据错位。

### 🟡 2.4 窗口状态每次像素移动都触发文件写入

- **文件**: `src-tauri/src/lib.rs:293-315`
- **当前做法**: `Moved` 和 `Resized` 事件的回调每次都直接执行 `std::fs::write`。
- **问题**: 拖拽窗口时每秒可能触发数十次事件 = 数十次磁盘写入。在 HDD 或 SSD 寿命有限的环境下是个问题。而且 `Moved` 事件处理中还先 `read_to_string` 再 `write`。
- **建议**: 使用 debounce：用 `std::time::Instant` 记录上次写入时间，至少间隔 500ms 才写入一次。或者使用 Tauri 的 `tauri-plugin-window-state` 插件，它内置了这个优化。
- **为什么更好**: 显著减少磁盘 I/O。

### 🟡 2.5 `set_shortcut` 如果旧快捷键解析失败会整体失败

- **文件**: `src-tauri/src/lib.rs:63-91`
- **当前做法**: 第 73 行 `old_accel.parse().map_err(|_| "shortcut parse error")?` — 如果旧快捷键已损坏（配置文件被手动修改），整个设置新快捷键的操作失败，旧快捷键仍然活跃且无法被替换。
- **建议**: 旧快捷键解析失败时应降级处理 — 记录警告但继续注册新快捷键，或者先尝试反注册已知的旧快捷键再注册新的。
- **为什么更好**: 健壮性：一个损坏的配置不应该阻止用户修复它。

### 🟡 2.6 `SettingsManager` 每个字段更新都全量写盘 — **已修复**

- **文件**: `src-tauri/src/shortcuts.rs:49-74`
- **当前做法**: 5 个不同的 update 方法 (`update_penetrate`, `update_auto_purge`, `update_opacity`, `update_webdav`, `update_theme`)，每个方法都立即执行 `std::fs::write`。
- **问题**: 重复代码（每个方法只有一行不同），且修改多个设置时（如 App 启动时同时设置 open/opacity/theme）会触发多次写盘。
- **建议**: 抽取一个 `save()` 私有方法，所有 update 方法调用它；或者使用 `Drop` 实现延迟写盘。
- **为什么更好**: 减少重复代码，可加 debounce 逻辑。

### 🟡 2.7 `init_db` 使用 `unwrap()` 会 panic — **已修复**

- **文件**: `src-tauri/src/lib.rs:236`
- **当前做法**: `db::init_db(db_path.to_str().unwrap()).expect("DB init failed")`
- **问题**: 如果 `db_path` 含非 UTF-8 字符（Windows 上某些用户名），`to_str()` 返回 `None`，直接 `unwrap()` panic。虽然概率极低，但在桌面应用中比返回友好错误更差。
- **建议**: 使用 `db_path.to_string_lossy()` 或 `db_path.as_os_str().to_str().ok_or(...)` + 友好错误提示。
- **为什么更好**: panic 不会给用户任何解释。

### 🟡 2.8 ETag 在 sync 中从未被有效使用

- **文件**: `src-tauri/src/lib.rs:152-156, 182`
- **当前做法**: 从 `.sync_etag` 文件读取 `_local_etag`（注意下划线前缀意味着未使用），sync 完成后**保存的是远程返回的新 etag**，不是合并后的状态对应的 etag。
- **问题**: 第一次同步后保存的 etag 是远程的版本。如果本地有更新的笔记（LWW 本地获胜），push 到服务器后，服务器返回**那个 push** 对应的 etag。但如果在 push 之后、保存 etag 之前又有本地修改，etag 就和实际数据不一致了。此外，`_local_etag` 完全没被使用 — 有条件地传 `If-None-Match` 可以做增量同步。
- **建议**: 要么正确实现基于 ETag 的增量同步（用 `If-None-Match` 避免不必要的全量下载），要么删除无用的 etag 文件写入逻辑。
- **为什么更好**: 代码要么有用要么删除，半成品逻辑会误导未来的维护者。

### 🟢 2.9 多处静默吞错误 — **已修复**

- **文件**: 多处
  - `src-tauri/src/lib.rs:234`: `create_dir_all(...).ok()` — 目录创建失败静默忽略
  - `src-tauri/src/lib.rs:284`: `let _ = w.set_position(...)` — 窗口位置恢复失败静默忽略
  - `src-tauri/src/lib.rs:182`: `let _ = std::fs::write(&etag_path, ...)` — etag 写入失败静默忽略
  - `src/App.vue:58`: `catch (_) {}` — 提醒检查失败静默忽略
  - `src-tauri/src/sync/mod.rs:33`: `.unwrap_or("")` — ETag header 缺失时使用空字符串
- **建议**: 至少记录日志（`eprintln!`/`log::warn!`/`console.warn`），对关键操作（如 etag 写入失败）应向用户反馈。

### 🟢 2.10 前端 `onMounted` 使用了 `async` 但没有错误处理 — **已修复**

- **文件**: `src/App.vue:26-43`
- **当前做法**: `onMounted(async () => { ... })` 中有多个 `await` 调用，如果 `get_penetrate`/`get_ontop`/`get_settings` 任何一个抛异常，后续的 `listen` 都不会注册。但整个回调外面没有 try-catch。
- **建议**: 将整个 `async` 回调包装在 try-catch 中，或者在每个关键 `await` 后检查，确保即使前一步失败，事件监听器仍然被注册。

---

## 3. 可靠性与边界情况

### 🟡 3.1 同步 merge 逻辑的冲突标记永远不会清除

- **文件**: `src-tauri/src/sync/mod.rs:79-112`
- **当前做法**: `merge()` 在 timestamp 相同时保留本地版本并设置 `has_conflict = true`，但不修改 `Note.conflict_id` 字段。且即使后续远程有了更新的版本，之前标记了 conflict 的笔记仍然保留 conflict 状态（因为本地不做清除）。
- **问题**: `conflict_id` 字段已经在数据库 schema 中定义，但 merge 逻辑完全不写入它。冲突一旦被标记就永远存在，除非用户手动编辑笔记（改变 `updated_at`）。
- **建议**: 在 merge 中正确设置 `conflict_id`：当远程有更新的版本时，如果本地笔记有 `conflict_id`，可以清除它（因为远程的新版本已经覆盖了冲突）。

### 🟡 3.2 无 SQLite WAL 模式

- **文件**: `src-tauri/src/db.rs:22-23`
- **当前做法**: `Connection::open(path)` 使用默认的 DELETE journal 模式。
- **建议**: 初始化后执行 `PRAGMA journal_mode=WAL;`。WAL 模式提供更好的并发读取性能，且在崩溃时更安全。
- **为什么更好**: 虽然不是必须的（只有一个连接），但 WAL 是 SQLite 的最佳实践，且没有副作用。

### 🟡 3.3 无数据库迁移策略

- **文件**: `src-tauri/src/db.rs:24-44`
- **当前做法**: `CREATE TABLE IF NOT EXISTS` — 只能创建初始 schema，不能处理 schema 变更。
- **建议**: 添加简单的迁移系统：在 DB 中维护一个 `schema_version` 表或使用 `user_version` pragma，每个版本号对应一组 ALTER TABLE 语句。
- **为什么更好**: 未来添加/删除字段时不会导致应用崩溃。

### 🟡 3.4 `write_file` 不使用原子写入

- **文件**: `src-tauri/src/commands.rs:62-64`
- **当前做法**: `std::fs::write(&path, &contents)` — 直接覆盖目标文件。
- **问题**: 如果写入过程中进程崩溃或磁盘满了，目标文件会被截断/损坏。
- **建议**: 写入到 `<path>.tmp`，确认写入成功后 `rename` 到目标路径（在 Windows 上 `rename` 是原子的）。
- **为什么更好**: 崩溃安全。

### 🟡 3.5 sync 每次创建新的 `reqwest::Client`

- **文件**: `src-tauri/src/sync/mod.rs:12, 51`
- **当前做法**: `fetch()` 和 `push()` 各自创建 `reqwest::Client::new()`。
- **建议**: 复用同一个 `Client` 实例（通过 Tauri state 管理），配置合理的超时时间（`.timeout(Duration::from_secs(30))`）。
- **为什么更好**: 连接池复用，减少 TCP 握手开销，超时明确化。

### 🟢 3.6 无版本的 sync 协议

- **文件**: `src-tauri/src/sync/mod.rs`
- **当前做法**: SyncPayload 只包含 `Vec<Note>`，没有版本号或协议版本字段。
- **问题**: 如果未来 Note 结构变化（添加/删除字段），不同版本客户端之间的 sync 会静默失败或数据丢失。
- **建议**: 在 `SyncPayload` 中添加 `version: u32` 字段，在 merge/push 前检查版本兼容性。

---

## 4. 构建与工程化

### 🔴 4.1 CSP 设为 null — 安全策略完全禁用 — **已修复**

- **文件**: `src-tauri/tauri.conf.json:28`
- **当前做法**: `"security": { "csp": null }`
- **问题**: 禁用 Content Security Policy 意味着任何注入的脚本都可以自由执行。这是一个安全底线被移除。即使 Tauri app 不直接面对 Web，如果加载了不受信任的内容或存在 XSS（比如 note 内容渲染），后果严重。
- **建议**: 设置最小权限的 CSP，如 `"default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:"`。

### 🔴 4.2 `tray_icon.rs` — 132 行不可维护的硬编码像素数组 — **已修复**

- **文件**: `src-tauri/src/tray_icon.rs`
- **当前做法**: 用 `vec![207,206,206,255,255,0,0,0,...]` 硬编码了 32×32 RGBA 像素数据（4096 字节 = 132 行代码）。
- **问题**: 
  1. 完全不可维护 — 没有人能通过读这些数字理解图标长什么样
  2. 如果图标需要修改，必须重新生成这个数组
  3. 代码审查时无法判断这段代码的正确性
  4. 它占用了 132 行代码的"视觉预算"
- **建议**: 
  1. 将图标存为 `icon.png`，用 `include_bytes!("../icons/32x32.png")` 在编译时嵌入，使用 `tauri::image::Image::from_bytes()` 加载
  2. 或者至少用代码生成一个简单的几何形状（如纯色圆角矩形），而不是包含大量噪声的手写像素数据
- **为什么更好**: 可维护、可替换、代码量减少 99%。

### 🟡 4.3 `reqwest` 的 `json` feature 未被使用

- **文件**: `src-tauri/Cargo.toml:19`
- **当前做法**: `reqwest = { version = "0.12", features = ["json"] }` — 启用了 `json` feature。
- **问题**: 代码中实际使用 `serde_json::to_string` 和 `serde_json::from_str` 手动序列化，没有使用 `reqwest` 的 `.json()` 方法。`json` feature 增加了编译时间和二进制大小但没有被使用。
- **建议**: 要么改用 `resp.json::<SyncPayload>().await` 和 `client.put(url).json(&payload)`（利用 `json` feature），要么从 features 中删除 `"json"`。
- **为什么更好**: 减少编译开销。

### 🟡 4.4 `pnpm-workspace.yaml` 内容为空

- **文件**: `pnpm-workspace.yaml`
- **当前做法**: 文件存在但 `packages: []`。
- **建议**: 如果不需要 monorepo workspace，直接删除这个文件。
- **为什么更好**: 减少维护负担。

### 🟡 4.5 零测试覆盖

- **文件**: 全局
- **当前做法**: 没有 Rust 单元测试/集成测试，没有前端 vitest 测试。
- **建议**: 
  - 至少为 `sync::merge()` 编写单元测试（纯函数，极易测试）
  - 为 `useNotes` 的 `midOrder()` 和 `needsRebalance()` 编写测试
  - 为 `db.rs` 编写集成测试（使用内存 SQLite `:memory:`）
- **为什么更好**: merge 逻辑和排序算法是数据正确性的核心，不应该只靠手动测试。

### 🟢 4.6 TypeScript `build` 脚本中 `vue-tsc --noEmit` 耗时但没有缓存

- **文件**: `package.json:8`
- **当前做法**: `"build": "vue-tsc --noEmit && vite build"`
- **建议**: 考虑使用 `vue-tsc` 的 `--incremental` 或在 CI 中使用 `--watch` 分离类型检查和构建。

---

## 5. 更简洁的实现

### 🔴 5.1 手动窗口状态管理 → 应使用 `tauri-plugin-window-state`

- **文件**: `src-tauri/src/lib.rs:239, 276-316`（约 40 行）
- **当前做法**: 手动读取/解析/写入 `window.json`，处理 Position/Size 事件的回调。
- **建议**: 使用 `tauri-plugin-window-state` 插件。一行 `.plugin(tauri_plugin_window_state::Builder::default().build())` 替代全部 40 行。
- **为什么更好**: 内置去抖动、窗口最大化/最小化状态保存、边界处理、跨平台兼容。

### 🟡 5.2 重复的 `SystemTime::now()` 时间戳计算 — **已修复**

- **文件**: `src-tauri/src/commands.rs:23-26, 33-37, 44-47`
- **当前做法**: 三处几乎完全相同的代码块计算当前 epoch millis：
  ```rust
  std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis() as i64
  ```
- **建议**: 在 `commands.rs` 中定义一个 `fn now_ms() -> i64` 辅助函数。
- **为什么更好**: DRY 原则，一行调用替代 4 行。

### 🟡 5.3 `shortcuts.rs` 的 N×代码重复 — **已修复**

- **文件**: `src-tauri/src/shortcuts.rs:49-74`
- **当前做法**: 5 个 update 方法，每个方法都是 `self.config.X = value; let _ = std::fs::write(...)`。
- **建议**: 抽取私有方法 `fn save(&self)`，每个 update 方法变为 `{ self.config.X = value; self.save(); }`，或者使用 builder 模式。
- **为什么更好**: 写盘逻辑集中在一处，未来要加 debounce 或错误处理只需改一处。

### 🟡 5.4 `ContextMenu.vue` 接收 8 个 props，`NoteList.vue` 转发 7 个 events

- **文件**: `src/components/ContextMenu.vue:4`, `src/components/NoteList.vue:8-16`
- **当前做法**: ContextMenu 有 6 个 props（`onAdd`, `onTop`, `onSettings`, `onExport`, `ontop`, `onClose`），NoteList 定义了 7 个 emit 事件逐层转发。
- **问题**: 每增加一个功能就需要修改 3 个文件的 props/emits 声明。这是典型的"prop drilling"和"event bubbling"问题。
- **建议**: 对于 context menu 的操作，考虑用 `provide`/`inject` 传递一个 `actions` 对象。或者直接用 composable 暴露方法，让深层组件直接调用。
- **为什么更好**: 减少样板代码，添加新功能只需改 composable。

### 🟢 5.5 `glass.css` 主题代码重复

- **文件**: `src/styles/glass.css:30-57`
- **当前做法**: 三个主题（green/blue/gray）各自独立声明 `background`、`backdrop-filter`。
- **建议**: 使用 CSS 自定义属性：
  ```css
  :root {
    --theme-bg: 18, 38, 30;
    --theme-saturate: 160%;
    --theme-accent: rgba(74, 200, 140, 0.08);
  }
  body.theme-green { --theme-bg: 18, 38, 30; --theme-saturate: 160%; }
  body.theme-blue  { --theme-bg: 16, 26, 40; --theme-saturate: 170%; }
  body.theme-gray  { --theme-bg: 26, 26, 28; --theme-saturate: 150%; }
  body::before {
    background: rgba(var(--theme-bg), var(--glass-opacity, 0.55));
    backdrop-filter: blur(32px) saturate(var(--theme-saturate));
  }
  ```
- **为什么更好**: 添加新主题只需设置 3 个变量，而不是复制 6 行代码。

### 🟢 5.6 `check_reminders` 可以用 SQL 完成全部过滤 — **已修复**

- **文件**: `src-tauri/src/commands.rs:42-59`
- **当前做法**: 加载全部笔记，在 Rust 中遍历过滤。
- **建议**: 直接在 SQL 中过滤（见 1.5），去掉整个 `filter` 闭包（17 行 → 1 个 SQL 查询）。

---

## 汇总统计

| 严重程度 | 数量 | 已修复 | 关键项 |
|---------|------|--------|--------|
| 🔴 严重 | 6 | 5 | 监听器泄漏✓、CSP✓、CSV注入✓、图标✓、列访问✓、任意文件写入✓ |
| 🟡 中等 | 16 | 5 | 密码明文、ETag逻辑、写盘风暴、测试缺失、WAL模式等 |
| 🟢 轻微 | 6 | 4 | 错误静默✓、CSS重复、prop drilling等 |

---

## 建议修复优先级

1. **已修复** ✅:
   - App.vue 事件监听器泄漏 — 新增 `unlistenReload` 变量
   - tauri.conf.json CSP 设为具体值
   - commands.rs 删除 `write_file` 命令，改用 dialog + fs plugin
   - tray_icon.rs 改为程序化 RGBA 生成
   - db.rs 命名列替代位置索引
   - commands.rs CSV 注入防护 (`csv_escape`)
   - shortcuts.rs 提取 `save()` 方法
   - 添加 `now_ms()` 辅助函数
   - lib.rs `unwrap` → `to_string_lossy()`
   - env_logger 日志基础设施（输出到 `%APPDATA%/sticky-notes/app.log`）

2. **下个迭代** (🟡):
   - 替换 `tauri-plugin-window-state`
   - WAL 模式、reqwest Client 复用
   - 基本测试覆盖（至少 merge + midOrder）

3. **长期** (🟢):
   - CSS 自定义属性重构
   - 数据库迁移框架
   - WebDAV 密码加密存储
