# 架构改进方案 — 桌面便签项目

基于 `/codebase-design` 词汇表（模块、接口、深度、接缝、适配器、杠杆、局部性）的系统评估。

---

## 一、当前模块评估

### 1. db.rs — 深层模块（良好）

| 维度 | 评估 |
|------|------|
| **接口** | 5 个函数：init_db, list_notes, upsert_note, soft_delete, purge_old |
| **实现** | 140 行，包含 SQL、迁移、行映射 |
| **深度** | 高 — 调用者只需传 Connection + Note，不需要知道 SQL 细节 |
| **杠杆** | 高 — commands.rs、sync/mod.rs、lib.rs 都通过同一组函数操作数据库 |
| **局部性** | 高 — 修改 SQL 只影响 db.rs，不扩散到调用者 |

**结论：无需重构。** 这是一个设计良好的深层模块，接口小，实现隐藏充分。

**注意：** commands.rs 中的 `check_reminders` 直接写 SQL，绕过了 db.rs 的接口，破坏了 db 的局部性。应将该 SQL 移入 db.rs。

---

### 2. commands.rs — 浅层模块（需要改进）

| 维度 | 评估 |
|------|------|
| **接口** | 6 个 #[tauri::command]：list_notes, save_note, delete_note, purge_old, check_reminders, export_notes |
| **实现** | 152 行，但大部分是样板代码（lock mutex → 调 db 函数 → map_err） |
| **深度** | 低 — 每个命令只是 db 函数的薄适配器，调用者（前端）需要知道每个命令的参数 |
| **杠杆** | 低 — 每个命令只被一个前端调用点使用 |
| **局部性** | 中 — check_reminders 和 export_notes 有内联逻辑（SQL 查询、CSV 生成），这些应该属于 db 或独立模块 |

**问题：**
- `check_reminders` 直接在 commands.rs 中写 SQL，绕过了 db.rs 的接口，破坏了 db 的局部性
- `export_notes` 包含 30 行 CSV 生成逻辑，这是一个独立的关注点
- `ts_to_iso` 是一个通用时间工具函数，不应该在 commands.rs 中

**改进方案：**
- 将 `check_reminders` 的 SQL 移入 db.rs（db::check_reminders）
- 将 `export_notes` 的 CSV 逻辑提取为独立模块 `export.rs`
- 将 `ts_to_iso` 移入 `utils/time.rs`

**删除测试：** 删除 commands.rs，复杂性会在 lib.rs 中重新出现 — 它在赚取它的位置，但可以更深。

---

### 3. lib.rs — 浅层模块（最需要改进）

| 维度 | 评估 |
|------|------|
| **接口** | 24 个 #[tauri::command] + run() 初始化 |
| **实现** | 495 行，混合了窗口管理、设置管理、同步、托盘、快捷键、Win32 |
| **深度** | 极低 — 接口（24 个命令）几乎和实现一样复杂 |
| **杠杆** | 低 — 每个命令只被一个前端调用点使用，但它们共享的逻辑（获取 WebDAV 配置、构建 URL）没有被提取 |
| **局部性** | 极低 — 修改同步逻辑需要在 495 行中搜索；修改窗口行为需要理解同步代码的上下文 |

**问题（按严重程度排序）：**

1. **同步命令（sync_notes/sync_push/sync_pull）** 占 ~180 行，每个都有重复的样板代码：
   - 获取 WebDAV 配置（~8 行，重复 3 次）
   - 构建 URL（~5 行，重复 3 次）
   - 读写 ETag（~5 行，重复 3 次）
   
   **接缝分析：** 三个同步命令共享相同的"配置获取 → URL 构建 → ETag 管理"模式。一个适配器 = 假设接缝，两个 = 真接缝。这里有三个，提取 SyncEngine 是正确的。

2. **窗口状态命令（toggle_penetrate/get_penetrate/toggle_ontop/get_ontop/set_opacity）** 混合了全局状态管理和 Tauri 命令

3. **设置命令（get_settings/set_shortcut/toggle_auto_purge/save_webdav/set_theme）** 都是 SettingsManager 的薄适配器

4. **run() 函数** ~130 行，混合了插件注册、快捷键注册、托盘创建、Win32 初始化

**改进方案：** 拆分为 4 个深层模块

```
lib.rs (run() + 模块注册, ~80 行)
  ├── window.rs      — 穿透/置顶/透明度状态 + 命令
  ├── settings_cmd.rs — 设置相关命令（薄适配器）
  ├── sync_cmd.rs    — 同步命令（消除重复样板）
  └── commands.rs    — 保留 CRUD/导出/提醒
```

**删除测试：** 删除 lib.rs 中的同步代码，复杂性不会消失 — 它会分散到每个调用者。这证明同步逻辑值得被提取为深层模块。

---

### 4. NoteCard.vue — 浅层组件（需要改进）

| 维度 | 评估 |
|------|------|
| **接口** | 6 个 emit 事件 + 2 个 props |
| **实现** | 364 行，混合了 6 个独立的 UI 状态机 |
| **深度** | 低 — 接口（6 个 emit）和实现（364 行 + 6 个 ref 状态）几乎一样复杂 |
| **杠杆** | 低 — 每个 emit 只被 NoteList.vue 一个调用点使用 |
| **局部性** | 低 — 修改颜色选择器需要理解子任务编辑的上下文 |

**问题：**
- 6 个独立的 ref 状态（editing, subEditingId, showColors, showDuePicker, editDueDate, editRemindAt）相互交织
- 主标题编辑和子任务编辑是两套独立的状态机，但共享同一个组件
- 颜色选择器和日期选择器是独立的 UI 关注点，但嵌入在主组件中

**改进方案：** 拆分为子组件

```
NoteCard.vue (~150 行, 容器 + 拖拽)
  ├── ColorPicker.vue    — 颜色选择器 (showColors 状态内化)
  ├── DueDatePicker.vue  — 截止日期选择器 (showDuePicker 等状态内化)
  └── SubtaskList.vue    — 子任务列表 (subEditingId 状态内化)
```

**删除测试：** 删除 NoteCard.vue，复杂性会在 NoteList.vue 中重新出现 — 它在赚取它的位置。但内部的子组件拆分能让每个子模块更深。

---

### 5. useNotes.ts — 中等深度（可改进）

| 维度 | 评估 |
|------|------|
| **接口** | 9 个函数：load, add, addSubtask, update, remove, toggleComplete, togglePin, reorder |
| **实现** | 143 行 |
| **深度** | 中 — 接口合理，但 add() 和 reorder() 包含内联的排序/重平衡逻辑 |
| **杠杆** | 高 — 被 App.vue 调用，驱动整个前端数据流 |
| **局部性** | 中 — 排序策略变更只影响 ordering.ts，但重平衡逻辑散布在 add() 和 reorder() 中 |

**问题：**
- `add()` 中的旧格式迁移逻辑（~15 行）应该是一次性迁移，不应该每次 add 都检查
- `reorder()` 中的重平衡逻辑（~15 行）与 add() 中的重复

**改进方案：**
- 提取 `rebalanceNotes(notes, insertAt)` 和 `nextOrder(notes, parentId)` 到 ordering.ts
- 将旧格式迁移逻辑移到 load() 中的一次性检查

---

### 6. NoteList.vue — 合理深度（小改进）

| 维度 | 评估 |
|------|------|
| **接口** | 7 个 emit 事件 + 2 个 props |
| **实现** | 177 行 |
| **深度** | 中 — 搜索/筛选/分组逻辑被 computed 属性很好地封装 |
| **杠杆** | 中 — 作为 NoteCard 的容器，提供了搜索/筛选/分组能力 |
| **局部性** | 中 — 搜索逻辑集中在 filteredNotes computed 中 |

**小改进：** `dateKey()` 函数可以提取到 utils/date.ts，因为其他地方也可能用到。

---

## 二、重构优先级列表

| 优先级 | 改进项 | 强度 | 深度提升 | 杠杆提升 | 局部性提升 | 工作量 |
|--------|--------|------|---------|---------|-----------|--------|
| **P1** | lib.rs 拆分（提取 sync_cmd.rs） | Strong | 极低→高 | 低→高 | 极低→高 | 2-3h |
| **P2** | lib.rs 拆分（提取 window.rs + settings_cmd.rs） | Strong | 极低→中 | 低→中 | 极低→中 | 1-2h |
| **P3** | NoteCard.vue 拆分子组件 | Strong | 低→高 | 低→中 | 低→高 | 2-3h |
| **P4** | useNotes.ts 排序逻辑深化 | Worth exploring | 中→高 | 高→高 | 中→高 | 1h |
| **P5** | commands.rs 内联逻辑提取 | Worth exploring | 低→中 | 低→中 | 中→高 | 1h |
| **P6** | 前端事件监听统一（useTauriEvent） | Speculative | — | — | — | 0.5h |

---

## 三、重构前后对比

### Rust 后端

**Before：**
```
lib.rs (495行, 24个命令)
  ├── 窗口管理命令 (穿透/置顶/透明度)
  ├── 设置命令 (主题/快捷键/WebDAV/自启)
  ├── 同步命令 (sync_notes/sync_push/sync_pull, 重复样板)
  ├── 托盘初始化
  ├── 快捷键注册
  └── Win32 桌面嵌入
commands.rs (152行)
  ├── CRUD 适配器
  ├── check_reminders (内联SQL)
  └── export_notes (内联CSV逻辑)
db.rs (140行) ← 良好，无需改动
```

**After：**
```
lib.rs (~80行, run() + 模块注册)
window.rs (~60行)
  ├── toggle_penetrate / get_penetrate
  ├── toggle_ontop / get_ontop
  └── set_opacity
settings_cmd.rs (~80行)
  ├── get_settings / set_shortcut
  ├── toggle_auto_purge / set_theme / save_webdav
  └── is_autostart / toggle_autostart
sync_cmd.rs (~120行, 消除重复)
  ├── SyncEngine::new(config)
  ├── engine.sync() / engine.push() / engine.pull()
  └── 统一的 URL 构建 + ETag 管理
commands.rs (~100行, 精简后)
  ├── CRUD 适配器
  └── check_reminders / export_notes (逻辑移出)
db.rs (~160行, +check_reminders SQL)
export.rs (~50行, CSV 逻辑)
```

### Vue 前端

**Before：**
```
NoteCard.vue (364行)
  ├── 主标题编辑状态
  ├── 子任务编辑状态
  ├── 颜色选择器状态
  ├── 日期选择器状态
  ├── 拖拽逻辑
  └── 6个 emit
```

**After：**
```
NoteCard.vue (~150行, 容器)
  ├── ColorPicker.vue (~50行)
  ├── DueDatePicker.vue (~40行)
  └── SubtaskList.vue (~80行)
```

---

## 四、关键原则

1. **深度是接口的属性** — 拆分的目标不是减少代码行数，而是让每个模块的接口更小、更深层
2. **删除测试** — 每个提取的模块都应该通过删除测试：删除它，复杂性是否在调用者中重新出现？
3. **接口是测试表面** — 拆分后的模块应该可以通过接口独立测试，不需要 mock 整个 Tauri 环境
4. **一个适配器 = 假设接缝，两个 = 真接缝** — sync_cmd.rs 的三个同步命令共享相同的 WebDAV 配置获取逻辑，这证明提取 SyncEngine 是正确的

---

## 五、补充：已知安全与稳定性问题

基于之前的架构审查，以下问题应在重构过程中一并修复：

| 优先级 | 问题 | 文件 | 说明 |
|--------|------|------|------|
| 立即 | 默认 opacity=0.6 触发窗口不可见 | shortcuts.rs | 改为 1.0 |
| 立即 | subclass ShowWindow 竞争 | win32/subclass.rs | 加 IsWindow 检查 |
| 立即 | reqwest Client 无超时 | sync/mod.rs | 共享 Client + 30s 超时 |
| 短期 | sync upsert 非事务性 | lib.rs | 包裹在事务中 |
| 短期 | merge 冲突检测字段不完整 | sync/mod.rs | 补充缺失字段 |
| 短期 | WebDAV 密码明文存储 | shortcuts.rs | 使用 Windows Credential Manager |
| 中期 | 前端批量 save IPC | useNotes.ts | 新增 save_notes_batch 命令 |
