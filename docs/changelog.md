# Sticky Notes — 项目总结报告

> 文档版本: 1.1  
> 最后更新: 2026-06-26  
> 项目路径: `D:\Codex_Work\01 桌面便签开发`

---

## 一、项目背景

Sticky Notes 是一个 Windows 桌面便签 / Todo 应用。目标是做一款**优雅、轻巧、好用**的桌面工具，具备毛玻璃 UI、深度系统集成、本地优先 + WebDAV 云同步能力。

### 技术选型

经过 Electron / Tauri 技术预研（详见 `spike-electron/`、`spike-tauri/`），最终选择 **Tauri 2.x**。实测内存占用 27.3MB（Electron 约 150MB）。

### 技术栈（含具体版本）

| 层 | 技术 | 版本 |
|----|------|------|
| 桌面框架 | Tauri | 2.x（features: `tray-icon`） |
| 前端框架 | Vue 3 | ^3.5 |
| 构建工具 | Vite | ^6 |
| 语言（前端） | TypeScript | ~5.7 |
| 语言（后端） | Rust | edition 2021 |
| 包管理器 | pnpm | 11.5.3 |
| CSS 预处理 | 原生 CSS（Glassmorphism） | — |
| 数据库 | SQLite (rusqlite) | 0.31（bundled） |
| HTTP 客户端 | reqwest | 0.12（features: `json`） |
| 序列化 | serde + serde_json | 1 |
| 系统路径 | dirs-next | 2 |
| Win32 API | windows-sys | 0.59 |

**Tauri 插件：**

| 插件 | 用途 |
|------|------|
| tauri-plugin-shell | 打开外部链接 |
| tauri-plugin-global-shortcut | 全局快捷键注册 |
| tauri-plugin-autostart | 开机自启 |
| tauri-plugin-dialog | 文件保存对话框 |
| tauri-plugin-fs | 文件写入 |

---

## 二、项目结构

```
/
├── src/
│   ├── main.ts                    # 入口，挂载 Vue
│   ├── App.vue                    # 根组件（提醒弹窗、主题）
│   ├── types/note.ts              # 数据模型 + 颜色常量
│   ├── composables/useNotes.ts    # 核心数据逻辑（CRUD/拖拽排序）
│   ├── components/
│   │   ├── NoteCard.vue           # 便签卡片（编辑/颜色/日期/拖拽/子任务）
│   │   ├── NoteList.vue           # 便签列表（搜索/筛选/颜色过滤）
│   │   ├── ContextMenu.vue        # 右键菜单
│   │   └── SettingsModal.vue      # 设置面板（快捷键/透明度/主题/WebDAV）
│   └── styles/glass.css           # 毛玻璃主题样式
├── src-tauri/
│   ├── Cargo.toml                 # Rust 依赖
│   ├── tauri.conf.json            # Tauri 配置
│   ├── icons/icon.ico             # 应用图标
│   ├── capabilities/default.json  # Tauri 权限配置
│   ├── build.rs                   # Tauri 构建脚本
│   └── src/
│       ├── main.rs                # 入口 + windows_subsystem
│       ├── lib.rs                 # 命令注册 + 系统托盘 + 设置初始化
│       ├── commands.rs            # Tauri IPC 命令（增删改查/导出/提醒检查）
│       ├── db.rs                  # SQLite 数据访问层
│       ├── shortcuts.rs           # 设置持久化（JSON）
│       ├── tray_icon.rs           # 系统托盘图标 RGBA 数据
│       ├── sync/
│       │   └── mod.rs             # WebDAV 同步（fetch/push/merge）
│       └── win32/
│           ├── mod.rs             # 桌面嵌入 + WS_EX_LAYERED
│           └── subclass.rs        # Win+D 守卫 + WM_NCHITTEST 穿透
├── docs/
│   ├── requirements.md            # 需求文档
│   ├── architecture.md            # 架构设计文档
│   └── CHANGELOG.md               # 本文件
├── build.bat                      # 一键构建脚本
├── dist/                          # 构建产物（gitignored）
│   ├── index.html
│   ├── assets/
│   └── sticky-notes.exe
└── icon/
    └── image_887244228252091.png  # 用户上传的图标源文件
```

---

## 三、核心功能清单

### P0 — MVP（全部完成）

| 功能 | 说明 |
|------|------|
| 便签 CRUD | 创建、编辑、删除，仅标题 |
| Todo 勾选 | 点击复选框标记完成/未完成，划掉文字 |
| 毛玻璃窗口 | 无标题栏、圆角、backdrop-filter blur（32px） |
| Win+D 生存 | 窗口嵌入桌面层（Progman/WorkerW），不被 Win+D 隐藏 |
| 鼠标穿透 | `set_ignore_cursor_events` + 全局快捷键 Ctrl+Alt+Shift+P |
| 拖拽移动窗口 | 顶部 handle `@mousedown -> start_dragging()` |
| 右键菜单 | 新建/置顶/设置/导出/隐藏到托盘 |
| 本地持久化 | SQLite，`notes.db` 在 `%APPDATA%/sticky-notes/` |

### P1 — 重要功能（全部完成）

| 功能 | 说明 |
|------|------|
| 子任务 | parent_id 两级树，可独立勾选/编辑/删除/设截止日期 |
| 拖拽排序 | HTML5 Drag API + 64 位十六进制中点算法 + 自动重平衡 |
| 便签置顶 | pinned 排最前 |
| 便签颜色 | 8 色预设（灰/红/橙/黄/绿/青/蓝/紫） |
| 系统托盘 | 隐藏到托盘，左键显示，右键菜单（显示/退出） |
| 开机自启 | 设置面板开关，通过 tauri-plugin-autostart |
| 记住窗口大小 | Moved/Resized 事件保存到 `window.json`，启动时恢复 |
| 30 天自动清理 | 启动时清理软删除 >30 天的记录，设置可开关 |

### P2 — 增强功能（全部完成）

| 功能 | 说明 |
|------|------|
| 搜索/筛选 | 关键词搜索标题+子任务，颜色筛选 |
| 导出 CSV | UTF-8 BOM，含状态/类型/耗时等轨迹字段 |
| 截止日期 + 提醒 | datetime-local 选择器，逾期标记，30 秒轮询弹窗 |
| WebDAV 同步 | 上传/下载分离，上传覆盖远程、下载覆盖本地，ETag 乐观锁，保留双向合并模式 |
| 主题切换 | 松绿 / 雾蓝 / 暖灰 三套毛玻璃配色 |
| 透明度调节 | 滑块 10%-90%，编辑时自动加深 |
| 快捷键自定义 | 设置面板按键捕获，动态重注册 |

### P3 — Todo/Done 体系（全部完成）

| 功能 | 说明 |
|------|------|
| Todo/Done 标签页 | 待办/已完成切换，状态 localStorage 持久化 |
| 已完成任务按日期分组 | Done 视图按完成时间分组（今天/昨天/日期/更早） |
| 主任务级联完成 | 勾选主任务时自动完成所有子任务；取消完成仅影响主任务 |
| completed_at 时间戳 | 新增字段，记录完成时刻，用于日期分组和 CSV 耗时计算 |
| 移除 content 字段 | UI 和数据模型全面移除 content，仅保留标题 |
| CSV 导出优化 | 新增状态/类型/父任务ID/完成时间/耗时(分钟)等列 |
| WebDAV 标签通用化 | 设置界面移除「坚果云」字样，适配所有 WebDAV 供应商 |

### 系统集成特性

| 功能 | 说明 |
|------|------|
| 全局快捷键 | `Ctrl+Alt+Shift+P` 切换穿透，可自定义 |
| 桌面嵌入 | 窗口作为桌面子窗口（Progman WorkerW），Win+D 不隐藏 |
| 穿透时保留拖拽区 | WM_NCHITTEST 保留顶部 30px 手柄区域可交互 |
| 系统托盘图标 | 32x32 RGBA 图标，左键显示窗口 |

---

## 四、技术架构详解

### 4.1 窗口穿透系统

**用户需求：** 窗口可以穿透点击到桌面，但又要能随时切回来。

**实现机制（`lib.rs` + `win32/subclass.rs`）：**

1. **`AtomicBool PEN`** — 全局穿透状态标识
2. **`set_penetrate()`** — 调用 `w.set_ignore_cursor_events(v)` 切换整个 WebView2 窗口的点击穿透
3. **WM_NCHITTEST 手柄保留** — 穿透模式下 subclass 拦截 `WM_NCHITTEST`：
   - 屏幕坐标 Y - 窗口顶部 <= 30px -> 返回 `DefSubclassProc`（允许鼠标事件穿透 DOM）
   - 其他区域 -> 返回 `HTTRANSPARENT`（点击穿透到桌面）
4. **全局快捷键** — `Ctrl+Alt+Shift+P` 在 Rust 侧注册：
   - 快捷键回调直接用 `PEN.load()` + `set_ignore_cursor_events()` + `app.emit("penetrate-changed")`
   - **注意：不调用 `set_focus()`** — 窗口有 `WS_EX_NOACTIVATE` 样式，`set_focus()` 永远无效

**前端的双向同步机制：**
- 组件挂载时调用 `get_penetrate()` 读取初始状态（不是 `toggle_penetrate()`，后者会翻转状态）
- `listen("penetrate-changed")` 监听 Rust 端快捷键切换
- ~~点击按钮不依赖事件，直接从 `toggle_penetrate()` 返回值更新~~（按钮已移除，仅保留快捷键控制）

**已修复的问题：**
1. 快捷键不同步 UI — 快捷键回调没发事件到前端。修复：加 `app.emit("penetrate-changed", v)`
2. `set_focus()` 在 `WS_EX_NOACTIVATE` 下无效 — 快捷键回调里调了 `set_focus()` 但永远失败。修复：移除
3. 初始状态读取不能调用 `toggle_penetrate` — 这个命令会翻转状态。修复：新增 `get_penetrate()` 只读命令
4. 按钮点击和快捷键走不同路径 — 统一为 `set_penetrate()` 函数
5. 穿透按钮点击后自身也穿透，无法点击回来 — 移除按钮，仅保留快捷键控制

### 4.2 Win+D 桌面嵌入

**用户需求：** Win+D（显示桌面）不能把便签窗口隐藏。

**实现机制（`win32/mod.rs` + `win32/subclass.rs`）：**

1. **`embed_desktop(hwnd)` 流程：**
   1. 保存原始父窗口句柄
   2. 查找 `Progman` 窗口（桌面文件夹窗口）
   3. 发送 `0x052C` 消息触发 WorkerW 创建
   4. 枚举所有 `WorkerW` 窗口，找到包含 `SHELLDLL_DefView` 子窗口的那个
   5. 将应用窗口的父窗口设为该 `WorkerW`
   6. 窗口现在变为"桌面小工具"，Win+D 不会隐藏它
2. **`unembed_desktop(hwnd)`** — 恢复原始父窗口，用于置顶模式
3. **Subclass WM_SHOWWINDOW 守卫** — 当 `wParam=0, lParam=0`（Win+D 隐藏信号）时，1ms 后自动 `ShowWindow(SW_SHOWNOACTIVATE)` 恢复

**已修复的问题：**
1. 使用了 `SetWindowLongW` 而非 `SetWindowLongPtrW` — 在 64 位系统上不兼容。修复：替换为 `SetWindowLongPtrW`
2. 早期代码用了 `!WS_EX_APPWINDOW` 移除 APPWINDOW 样式 — 导致窗口在 Win10/11 下 Win+D 后彻底消失。修复：改为直接嵌入桌面层
3. 早期依赖 `Progman` 作为唯一父窗口 — Win10/11 上桌面架构复杂。修复：使用 WorkerW 枚举
4. `EnumWindows` 回调线程安全问题 — 修复：用 `SendMessageW` 同步通信

### 4.3 置顶 + 桌面嵌入的关系

```rust
// toggle_ontop 的逻辑：
if v {  // 开启置顶
    w.set_always_on_top(true);
    unembed_desktop(hwnd);  // 退出桌面嵌入
} else {  // 关闭置顶
    w.set_always_on_top(false);
    embed_desktop(hwnd);    // 嵌入桌面（Win+D 不隐藏）
}
```

### 4.4 排序系统

**需求：** 便签按用户拖拽的顺序排列，新增的排在最后。

**当前实现（`useNotes.ts`）：**
- 使用 **64 位十六进制字符串**作为排序键（定长 10 字符，如 `"0000000000"`、`"0000000001"`）
- `midOrder(prev, next)` 取两个 hex 值的中间点（BigInt 算术）
- 当两个排序键之间没有空隙时 -> 全部顶层便签重新分配 `hexOrder(0), hexOrder(1)...`
- 排序先按 `pinned`，再按 `order` 词法排序

**已修复的问题：**
1. 初期方案是字母序列 `a0, b0, ..., z0, aa0, ab0, ...`。`"aa0" < "z0"` 词法比较错误，`nextOrder("z0")` 返回 `"z01"` 然后无限增长。修复：改为定长 hex
2. 当前 hex 方案的限制：`midOrder` 计算 `BigInt(prev + next) / 2`，当相邻两个 hex 差距只有 1 时（如 `...01` 和 `...02`），`(01+02)/2=01` 不满足 `mid > lo`，触发全部重排。大量拖拽操作可能触发频繁重排。

### 4.5 WebDAV 同步

**三个核心函数（`sync/mod.rs`）：**

- **`fetch(url, user, password)`:** GET -> 解析 `SyncPayload { notes: Vec<Note> }` -> 返回 `(payload, etag)`
- **`push(url, user, password, notes, current_etag)`:** PUT + `If-Match: <etag>` -> 乐观锁。412 -> 冲突。返回新 ETag
- **`merge(local, remote)`:** Entity-level LWW。相同 `id`：取 `updated_at` 较大的。相等但内容不同 -> `has_conflict = true`（保留本地）。按 `order` 排序

**同步命令（`lib.rs`）：**

- **`sync_notes`（保留，双向合并）：** 拉取远程 -> 合并 -> 写本地 DB -> 推远程 -> 保存新 ETag
- **`sync_push`（单向上传）：** 读本地 DB -> 推送到远程（覆盖远程）-> 保存新 ETag。不修改本地数据库
- **`sync_pull`（单向下载）：** 拉取远程 -> 写入本地 DB（覆盖本地）-> 保存 ETag。不推送到远程

**设计决策：** 双向合并（LWW）在某些场景会导致已完成任务被远程旧版本覆盖而回退为未完成。拆分为独立的上传/下载让用户明确控制数据流向，避免状态回退。

**注意：** `sync_notes` 是 `async fn`，`state.0.lock()` 返回的 `MutexGuard` 不能跨 `.await` 持有。必须缩放在 block 内：
```rust
let local_notes = {
    let conn = state.0.lock()?;
    db::list_notes(&conn)?
};  // MutexGuard dropped here, before .await
```
忘了做会导致 `future cannot be sent between threads safely` 编译错误。

### 4.6 窗口样式方案

**`win32/mod.rs`：**
- `apply_styles` 只设置了 `WS_EX_LAYERED`（用于 `SetLayeredWindowAttributes` 透明度）
- **不**设置/清除 `WS_EX_TOOLWINDOW`、`WS_EX_APPWINDOW`、`WS_EX_NOACTIVATE` — 这些由 Tauri 的 `skipTaskbar` 配置控制
- `SetLayeredWindowAttributes` 设置初始透明度 240（约 94%）

### 4.7 枚举窗口辅助函数（win32/mod.rs）

```rust
unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: isize) -> BOOL {
    // 查找 WorkerW 窗口，检查是否有 SHELLDLL_DefView 子窗口
}
```

**坑：** `windows-sys` 0.59 中 `HWND`、`BOOL` 等类型需要从 `windows_sys::Win32::Foundation` 导入或自定义类型别名。`w!()` 宏不可用，需要用 `encode_wide()` 函数手动编码 UTF-16 字符串。

---

## 五、核心数据模型

### Note 数据结构

```typescript
interface Note {
  id: string;              // crypto.randomUUID()
  title: string;
  parent_id: string|null;  // 子任务的父 ID
  order: string;           // 排序键（10 字符 hex）
  completed: boolean;
  pinned: boolean;
  color: string;           // 8 色之一
  created_at: number;      // Date.now()
  updated_at: number;
  deleted_at: number|null; // 软删除时间戳
  conflict_id: string|null; // 用于同步冲突标记
  due_date: number|null;   // 截止日期时间戳
  remind_at: number|null;  // 提醒时间戳
  completed_at: number|null; // 完成时间戳（用于 Done 视图分组）
}
```

### SQLite Schema

```sql
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL DEFAULT '',
    parent_id TEXT,
    [order] TEXT NOT NULL DEFAULT 'a0',
    completed INTEGER NOT NULL DEFAULT 0,
    pinned INTEGER NOT NULL DEFAULT 0,
    color TEXT NOT NULL DEFAULT '#333333',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    conflict_id TEXT,
    due_date INTEGER,
    remind_at INTEGER,
    completed_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_order ON notes([order]);
CREATE INDEX IF NOT EXISTS idx_parent ON notes(parent_id);
CREATE INDEX IF NOT EXISTS idx_updated ON notes(updated_at);
```

### 设置配置文件（shortcuts.json）

```json
{
  "penetrate": "Ctrl+Alt+Shift+P",
  "auto_purge": true,
  "opacity": 0.6,
  "theme": "green",
  "webdav_url": "",
  "webdav_user": "",
  "webdav_password": ""
}
```

存储在 `%APPDATA%/sticky-notes/shortcuts.json`。每次更新写入完整 JSON（不是 patch）。

---

## 六、所有已修复的问题与坑

### 6.1 前端问题

| # | 问题 | 根因 | 修复方案 |
|---|------|------|----------|
| 1 | 快捷键切换穿透但按钮不同步 | 快捷键回调没发事件给前端 | 加 `app.emit("penetrate-changed", v)` |
| 2 | `onMounted` 读取穿透状态时误翻转 | 调用了 `toggle_penetrate` 而非 `get_penetrate` | 新增只读 `get_penetrate` 命令 |
| 3 | 编辑框失焦后内容丢失 | 无 blur 自动保存 | `@blur="save"` + `@mousedown.prevent` 防止按钮抢焦 |
| 4 | 右键菜单溢出窗口边缘 | 位置没有钳制到视口 | `Math.min(clientX, innerWidth - 150)` 等 |
| 5 | ContextMenu 干扰编辑框原生右键 | 全局拦截了所有右键 | 过滤 `INPUT`/`TEXTAREA` 标签 |
| 6 | 搜索不到子任务 | filter 只过滤了顶层笔记 | 同时搜索子任务标题 |
| 7 | 子任务日期 + 按钮点了没反应 | 缺少 `showDuePicker = true` | 补上 `showDuePicker = true` |
| 8 | CSV 打开乱码 | UTF-8 无 BOM | CSV 头部加 `\u{feff}`（BOM） |
| 9 | 导出 CSV 点击无反应 | Tauri 权限配置缺少 `fs:allow-write` 和 scope | 补充 `capabilities/default.json`，改用 Rust 端 `write_file` 命令绕过前端 fs 权限 |
| 10 | .gitignore 漏掉 `dist.zip` 和 `dist/assets/` | 初始只忽略了 `dist/sticky-notes.exe` | 补上 `dist/index.html`, `dist/assets/`, `*.zip` |

### 6.2 Rust 后端问题

| # | 问题 | 根因 | 修复方案 |
|---|------|------|----------|
| 11 | 64 位系统上窗口样式设置可能失败 | 用了 `SetWindowLongW` 而非 `SetWindowLongPtrW` | 替换为 `SetWindowLongPtrW` |
| 12 | Win+D 后窗口彻底消失 | 移除了 `WS_EX_APPWINDOW` 样式 | 改为嵌入桌面层（Progman/WorkerW） |
| 13 | 窗口置顶按钮无效 | `win32/mod.rs` 中硬编码 `HWND_TOPMOST` 覆盖了 Tauri 设置 | 移除 `HWND_TOPMOST` |
| 14 | `sync_notes` 编译失败 `future not Send` | `MutexGuard` 跨 `.await` 持有 | 把锁放在 block 域内 |
| 15 | Release 编译后弹 cmd 黑窗 | 缺少 `windows_subsystem = "windows"` | 在 `main.rs` 加 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` |
| 16 | `dist/sticky-notes.exe` 运行找不到 localhost:1420 | 用 `cargo build --release` 覆盖了 `pnpm tauri build` 的输出。`cargo` 不内嵌前端页面 | 只能用 `pnpm tauri build` 构建生产版，`build.bat` 先删旧的 exe 再跑完整构建 |
| 17 | `build.bat` 中 `del /f /q "dist\sticky-notes.exe"` 有时失败 | 文件被进程锁定 | 构建前先手动清理 |
| 18 | `w!()` 宏在 `windows-sys` 0.59 中不工作 | 该宏在 `windows-strings` crate 中 | 自行写 `encode_wide()` 函数编码 UTF-16 |
| 24 | 启动时置顶按钮显示为"已置顶"但窗口实际未置顶 | Rust `ONTOP` 初始值为 `true`，前端 `ontop` ref 也是 `true`，但 `tauri.conf.json` 的 `alwaysOnTop` 为 `false`，三者不一致 | 将 `ONTOP` 和前端初始值都改为 `false`，与实际窗口状态一致 |

### 6.3 Win32 子类化问题

| # | 问题 | 根因 | 修复方案 |
|---|------|------|----------|
| 19 | Raw WndProc 覆盖导致 Tauri/WebView2 DPI 处理损坏 | 用 `SetWindowLongPtrW(h, GWLP_WNDPROC, ...)` 替换了原始的窗口过程 | 改用 `SetWindowSubclass` + `DefSubclassProc`（安全的子类化链） |
| 20 | `data-tauri-drag-region` 在 WebView2 中不工作 | WebView2 子 HWND 架构下 HTML drag region 无效 | 移到 Win32 `WM_NCHITTEST` 手动处理 |

### 6.4 构建与部署问题

| # | 问题 | 根因 | 修复方案 |
|---|------|------|----------|
| 21 | `pnpm` 不在系统 PATH 中 | Codex 运行时环境 | `build.bat` 中手动设置 PATH |
| 22 | PowerShell `Out-File` 输出 UTF-8 BOM，导致 Rust 编译 JSON 解析失败 | PowerShell 默认编码带 BOM | 使用 `File.WriteAllText` 或 `-Encoding utf8` 参数 |
| 23 | `Rc.exe` 编译资源失败 | 中文字符编码问题 | 项目名用纯英文 "sticky-notes" |

### 6.5 代码审查修复（2026-06-24）

| # | 问题 | 根因 | 修复方案 |
|---|------|------|----------|
| 24 | `write_file` 任意文件写入后门 | 前端可调用写入任意路径 | 删除命令，改用 `writeTextFile` plugin |
| 25 | CSV 注入漏洞 (CWE-1236) | 未处理 `=+-@` 开头字段 | `csv_escape()` 加单引号前缀 |
| 26 | `SELECT *` + 位置索引读取 | 列重排会导致数据错位 | 改为命名列 `row.get("id")` |
| 27 | 事件监听器泄漏 | `unlistenPen` 被 `unload` 覆盖 | 新增 `unlistenReload` 变量 |
| 28 | `check_reminders` 全表扫描 | 加载所有笔记到内存过滤 | 改为 SQL WHERE 子句过滤 |
| 29 | `now_ms()` 重复代码 | 三处相同 SystemTime 计算 | 提取 `now_ms()` 辅助函数 |
| 30 | `set_shortcut` 旧键解析失败整体失败 | 损坏配置阻止修复 | 降级处理，记录日志继续 |
| 31 | `init_db` 中 `to_str().unwrap()` panic | 非 UTF-8 路径 | 改用 `to_string_lossy()` |
| 32 | 托盘图标 PNG 损坏（110 字节空白） | 源文件损坏 | 改为程序化 RGBA 生成 |
| 33 | WebDAV 同步 405 错误 | GET/PUT 发到目录路径 | `ensure_dir()` MKCOL + 文件路径构建 |
| 34 | CSP 设为 null | 安全策略完全禁用 | 设为具体 CSP 策略 |
| 35 | `fs:allow-write` scope `**` | 权限过大 | 改为 `fs:allow-write-text-file` |
| 36 | 同步逻辑导致任务状态回退 | 双向合并（LWW）会把已完成的任务重新拉回未完成 | 拆分 `sync_push`（上传覆盖远程）和 `sync_pull`（下载覆盖本地），设置面板改为独立的「上传」和「下载」按钮 |
| 37 | 快捷键修改后底部提示不更新 | `Ctrl+Alt+Shift+P` 硬编码在 App.vue 模板中 | 新增 `shortcut` ref，`onMounted` 时从 `get_settings` 读取，模板和 title 均使用动态值 |

---

## 七、关键文件逐文件说明

### 7.1 前端

#### `src/App.vue` — 根组件

- `onMounted` 加载穿透状态、置顶状态、主题、快捷键配置
- Todo/Done 标签栏（`activeTab`，localStorage 持久化）
- `setInterval(checkReminders, 30000)` 轮询提醒
- `listen("notes-reloaded")` 监听 WebDAV 同步完成后的重新加载
- `exportNotes()` 动态 import dialog + writeTextFile 保存文件
- `watch(isEditing)` 切换 `body.editing` 类控制编辑时背景加深
- 提醒弹窗：轮询到逾期/提醒便签时显示顶部 toast，点击「知道了」关闭
- `shortcut` ref 从 `get_settings` 读取，底部提示和穿透按钮 title 均使用动态值

#### `src/composables/useNotes.ts` — 数据核心

- 顶层 `const notes = ref<Note[]>([])` 是模块级单例
- `load()` 从 SQLite 读取所有 notes，按 pinned + order 排序
- `add()` 新增顶层便签，`addSubtask()` 新增子任务
- `update()` 更新触发响应式 + 排序
- `reorder(id, beforeId)` 拖拽排序：尝试中点插入，失败则全量重排
- `toggleComplete(note)` 切换完成状态并设置 `completed_at`，主任务完成时级联完成所有子任务
- **注意：** `update()` 和 `reorder()` 中重新排序时不要创建新数组去替换 `notes.value` 后丢失已展开的编辑状态。正确做法是更新 `copy[idx]` 再整体替换。

#### `src/components/NoteCard.vue` — 最复杂的组件

**编辑状态：**
- 单个便签编辑（`editing` ref）和子任务编辑（`subEditingId` ref）独立
- 编辑时 blur 自动保存（`onBlur` 调用 `save`）
- 按钮用 `@mousedown.prevent` 避免触发表单 blur
- 仅编辑标题（content 字段已移除），Enter 键快速保存

**子任务：**
- 从 `allNotes` prop 中 filter `parent_id === note.id`
- 每个子任务有独立 checkbox、编辑、删除、截止日期按钮
- 子任务的日期弹出框用独立的 `subDueId` 跟踪

**拖拽：**
- HTML5 Drag API，`draggable` 在编辑模式下关闭
- `dragstart` 设源卡片半透明，`dragover` 标记目标绿色高亮
- `drop` 调用 `emit("reorder", draggedId, beforeId)`

**截止日期弹窗：**
- `showDuePicker` 控制显示
- `saveDue()` 保存时判断 `subDueId` 为空走父便签，非空走子任务
- `initDueEdit(t)` 支持传入子任务 Note 参数
- 逾期标记：`due_date < Date.now() && !completed` 显示红色「逾期」badge

#### `src/components/SettingsModal.vue` — 设置面板

- 快捷键：点击进入监听模式 -> 键盘按下时 `formatShortcut` 生成 accelerator 字符串
- 透明度：拖拽时实时更新 CSS 变量 `--glass-opacity`，松开时保存到配置
- WebDAV：通用 WebDAV URL/账号/密码输入 + 密码显示/隐藏切换 + 保存配置 + 独立的「上传」和「下载」按钮（分别调用 `sync_push` 和 `sync_pull`）
- 主题：点击切换、更新 `body.theme-*` 类、持久化到配置
- 开机自启、30天清理：开关按钮

### 7.2 后端

#### `src-tauri/src/lib.rs` — 核心枢纽

**setup 钩子执行顺序：**

1. 获取主窗口
2. 恢复窗口位置/大小（window.json）
3. 注册窗口 Moved/Resized 事件监听
4. 注册全局快捷键
5. 创建系统托盘
6. Win32 样式 + 桌面嵌入 + subclass 守卫

**注意：** 窗口状态监听用的是 `w.on_window_event()` 而非 `app.on_window_event()`。每个监听器是独立的，不能用同一个闭包覆盖。

#### `src-tauri/src/sync/mod.rs` — WebDAV 同步

- `SyncPayload` 是传输格式：`{ notes: Vec<db::Note> }`
- `fetch()` 处理 404（首次同步）为空远程
- `push()` 用 `If-Match` 做乐观锁
- `merge()` 是纯函数式，不写数据库

#### `src-tauri/src/win32/` — Windows 集成

**`mod.rs`：**
- `embed_desktop()` 和 `unembed_desktop()` 用于切换桌面嵌入
- `apply_styles()` 只做 WS_EX_LAYERED + 透明度
- `encode_wide()` 辅助函数替代不可用的 `w!()` 宏

**`subclass.rs`：**
- `install_guard()` 安装子类化回调
- `PEN_PTR` 是 `static mut` 的裸指针，指向 `lib.rs` 的 `PEN AtomicBool`。必须是 `static mut` 因为 `SetWindowSubclass` 的 callback 签名不能带闭包
- WM_NCHITTEST 中 `lparam` 解包方式：`((lparam as u32 >> 16) & 0xFFFF) as i16 as i32` 得到屏幕 Y 坐标

---

## 八、构建与部署

### 生产构建

```bash
# 方式 1：双击 build.bat
build.bat

# 方式 2：手动执行
pnpm install
pnpm tauri build

# 产物位置：
# src-tauri/target/release/sticky-notes.exe
# 约 17MB
```

**关键：** 必须用 `pnpm tauri build` 而不是 `cargo build --release`。前者会先 Vite 打包前端（HTML/JS/CSS 到 `dist/`），然后 Rust 编译并把这些文件嵌入二进制。后者只编译 Rust，出来的 exe 没有前端页面，运行时找 `localhost:1420` 会报错。

### 运行时依赖

- Windows 10 1903+ / Windows 11
- WebView2 Runtime（Win11 已内置，Win10 可能需要安装）

### 配置文件位置

- `%APPDATA%/sticky-notes/notes.db` — SQLite 数据库
- `%APPDATA%/sticky-notes/shortcuts.json` — 设置
- `%APPDATA%/sticky-notes/window.json` — 窗口位置
- `%APPDATA%/sticky-notes/.sync_etag` — WebDAV 同步 ETag 缓存
- `%APPDATA%/sticky-notes/app.log` — 运行日志（env_logger，INFO 级别）

### 数据模型变更注意事项

如果后续需要 `shortcuts.json` 添加新字段，**`SettingsConfig` 的 `Default` impl 必须提供合理的默认值**，因为 `serde::from_str()` 解析旧文件时，缺失的字段不会自动获得默认值（除非 serde 的 `#[serde(default)]` 在字段级别标注）。当前实现通过 `unwrap_or_default()` 兜底处理。

---

## 九、尚未实现的需求

| 功能 | 优先级 | 说明 |
|------|--------|------|
| F15 冲突标记 | P1 | WebDAV merge 时已标记 `has_conflict`，但前端无冲突 UI。需要 `conflict_id` 字段和冲突对比界面 |
| 拖拽排序重平衡优化 | — | 当前 `midOrder` 在相邻 key 差距为 1 时触发全量重排。可改用更精细的分数索引算法 |
| 通知系统增强 | — | 当前提醒是前端轮询 + 简单弹窗。可改用系统原生通知 |
| WebDAV 密码加密 | — | `shortcuts.json` 中 `webdav_password` 明文存储 |
| 子任务多级嵌套 | — | 目前只有两级（父 -> 子） |
| 窗口尺寸限制 | — | 没有设置最小/最大窗口尺寸 |
| 多语言支持 | — | 目前仅中文 UI |
| 自动更新 | — | 无 |

---

## 十、Git Commit 时间线

```
dbecf09 fix: show '首次同步' instead of error on first-time sync
fa3902f fix: WebDAV sync 405 error — use MKCOL for directory, PUT/GET file paths
eeb52ce chore: allow git tag command in Claude settings
2015895 chore: allow shell navigation commands in Claude settings
dc27af8 feat: add logging infrastructure and password visibility toggle
45d5f25 chore: allow tasklist command in Claude settings
3d73332 fix: redesign tray icon as bold orange rounded-square with white checkmark
e3ed9ed fix: replace PNG-based tray icon with procedural hardcoded RGBA bitmap
9451863 chore: fix build.bat step numbering (1/3 → 1/4)
8a85248 chore: update permissions and mark resolved code-review items with final verdict
d1369d5 build: fix build.bat permission error by deleting stale exe before tauri build
b5e3f79 build: fix production binary not embedding frontend, update build.bat
```

---

## 十一、已知限制与技术债务

1. **排序频繁重排：** hex 中点算法在密集排列时触发全量重写所有排序键。数据量大（>100 条）时可能性能问题。见 `useNotes.ts` 的 `reorder()` 函数和 `midOrder()` 函数
2. **WebDAV 密码明文存储：** `shortcuts.json` 中的 `webdav_password` 未加密
3. **子任务不支持递归嵌套：** 只有两级（父 -> 子），子任务不能再有子任务
4. **提醒仅前端轮询：** 应用关闭时不会有任何提醒
5. **窗口嵌入退出时未恢复：** 应用关闭时没有调用 `unembed_desktop`，理论上不影响下次启动
6. **托盘图标编码为硬编码数组：** `tray_icon.rs` 中的 RGBA 数据是生成的，修改图标需要重新生成
7. **缺少错误处理 UI：** 多数 IPC 调用在 catch 中只 `console.error`，用户看不到错误
8. **子任务日期弹窗与父便签共用位置：** 子任务的日期弹窗不能独立定位，跟随父便签按钮位置

---

## 十二、审查注意事项

### 审查时需重点检查的模块

1. **`useNotes.ts`** — 排序逻辑（`midOrder` + `reorder` + `needsRebalance`）、pin + order 排序优先级、添加便签时旧 hex 格式检查
2. **`NoteCard.vue`** — 编辑/子任务编辑/日期编辑三个独立状态机的正确性、拖拽事件正确 emit、blur 自动保存和按钮 `@mousedown.prevent` 配合
3. **`lib.rs`** — 命令注册列表完整性、`sync_notes` async 中的 MutexGuard 生命周期、`on_window_event` 不能覆盖
4. **`win32/mod.rs`** — `embed_desktop` 的 WorkerW 枚举逻辑、`encode_wide` 函数
5. **`win32/subclass.rs`** — `static mut PEN_PTR` 线程安全性、WM_NCHITTEST lparam 解析
6. **`tauri.conf.json`** — `frontendDist: "../dist"`、`beforeBuildCommand: "pnpm build"`
7. **`capabilities/default.json`** — 所有权限声明必须与实际 IPC 调用匹配
8. **`settings.json` 新字段兼容性** — `SettingsConfig::default()` 必须包含默认值

### 审查时可能遇到的问题

- **Build 产物不正确：** 确认 `dist/sticky-notes.exe` 是由 `pnpm tauri build` 生成的，不是 `cargo build`。最简单的验证方法：删除 `dist/` 下所有前端文件（`index.html`、`assets/`），如果 exe 还能正常运行就对了
- **Win+D 测试：** 按 Win+D 窗口应该立即恢复，不要隐藏。如果隐藏了，检查 `embed_desktop` 是否在 `setup` 中被调用
- **Edit auto-save：** 点击编辑框外部区域应该自动保存，不应该点「保存」按钮。按钮用 `@mousedown.prevent` 防止 blur 事件不触发
- **快捷键更改：** 改快捷键后旧快捷键应该被取消注册，新快捷键立即生效。`set_shortcut` 命令中先 `unregister` 旧键再注册新键
- **排序正确性：** 添加便签、拖拽后、pin 切换后，排序应该立即刷新。验证 `useNotes.ts` 中所有修改 `notes.value` 的地方都调用了 `sortNotes`

---

*本文档基于完整的源码分析、Git 历史和开发过程中的问题记录生成。*
