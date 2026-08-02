# Changelog

本文件记录 Sticky Notes 的所有重要变更。格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)。

---

## [Unreleased]

### 新增
- F20 搜索/筛选（已有基础功能）
- F21 Todo/Done 标签页（已有基础功能）
- F19 数据导出 CSV（已有基础功能）

---

## [0.3.0] - 2026-08-01

### 新增
- **F17 WebDAV 近实时同步**：三触发点方案——编辑后 3 秒防抖推送、窗口获焦自动拉取、60 秒定时轮询
- **F15 同步冲突解决**：冲突时便签显示 ⚠️ 徽章，点击弹出对比面板，可选择保留本地或远程版本
- **F18 截止日期提醒优化**：提醒弹窗显示便签内容，新增"延后 15 分钟"snooze 按钮
- **F25 子任务折叠/展开**：有子任务的便签可折叠，折叠时显示"X/Y 已完成"摘要
- **撤销/重做**：`Ctrl+Z` 撤销、`Ctrl+Y` 重做，最多 50 步历史
- **单实例保护**：重复打开只激活已有窗口，不会创建多个实例
- **UI 设计系统**：建立统一的 Design Tokens（间距、字体、颜色、圆角、过渡动画）
- **渐进式披露**：便签卡片操作按钮默认隐藏，悬停时才显示
- **冲突表**：SQLite 新增 `conflicts` 表存储冲突的远程版本

### 变更
- 标签栏和标题栏字体从 11px 加大到 12px
- 截止日期按钮统一为 📅 图标
- 置顶按钮移到卡片最右侧
- 子任务缩进从 14px 减小到 8px
- 折叠三角形从 14px 加大到 16px
- 快捷键设置修改后底部提示栏实时更新

### 修复
- 快捷键在设置中修改后界面不更新的问题
- 子任务截止日期按钮点击无反应的问题
- Cargo.toml 删除不必要的 `staticlib` crate-type

---

## [0.2.0] - 2026-08-01

### 新增
- **架构重构**：lib.rs 从 495 行拆分为 4 个深层模块
  - `window.rs` — 窗口状态命令（穿透/置顶/透明度）
  - `settings_cmd.rs` — 设置相关命令
  - `sync_cmd.rs` — SyncEngine 深层模块
  - `commands.rs` — CRUD/导出/提醒命令
- **NoteCard.vue 拆分**：提取 ColorPicker 和 DueDatePicker 子组件
- **useTauriEvent composable**：自动管理 Tauri 事件监听器生命周期
- **排序逻辑深化**：提取 `rebalanceNotes` 和 `nextOrder` 到 ordering.ts
- **设计令牌系统**：glass.css 定义统一的间距、字体、颜色变量

### 变更
- lib.rs 从 495 行降到 206 行（-58%）
- NoteCard.vue 从 364 行降到 270 行（-26%）
- useNotes.ts 从 143 行降到 124 行（-13%）
- App.vue 从 207 行降到 198 行（-4%）
- commands.rs 从 152 行降到 121 行（-20%）
- 所有组件样式统一使用 Design Tokens
- tsconfig.json 排除 `__tests__` 目录

### 修复
- `window.rs` 的 `pub static` 改为 `pub(crate) static`
- `NoteCard.vue` 移除 `emit("reorder" as any)` 类型绕过

---

## [0.1.0] - 2026-06-29

### 新增
- 便签 CRUD（创建、编辑、删除）
- Todo/Done 标签页切换
- 拖拽排序（HTML5 Drag API + Fractional Index）
- 便签置顶
- 8 种预设颜色
- 子任务（两级树结构）
- 毛玻璃主题（松绿/雾蓝/暖灰）
- 鼠标穿透（自定义快捷键）
- 窗口置顶
- 嵌入桌面（Win+D 生存）
- 系统托盘
- WebDAV 同步（上传/下载）
- 截止日期 + 提醒
- 搜索/筛选
- CSV 数据导出
- 开机自启
- 自动保存
- 30 天自动清理
