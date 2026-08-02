# Requirements Document v5

## Project
A Windows desktop sticky-notes / todo app. Glass UI, deep system integration, local-first with WebDAV sync.

**核心定位：** 个人效率工具，辅以轻量级项目管理。
**核心卖点：** 嵌入桌面（Win+D 可见，零摩擦任务完成闭环）。

---

## V1 功能清单

### P0 — MVP（已完成）

| ID | Feature | Description |
|----|---------|-------------|
| F01 | Note CRUD | 创建、编辑、删除便签，自动保存 |
| F02 | Todo check | 复选框标记完成，Todo/Done 标签页分组 |
| F03 | Frameless glass window | 无标题栏，圆角，毛玻璃 backdrop-filter blur |
| F04 | Win+D survival | WM_SHOWWINDOW guard via SetWindowSubclass |
| F05 | Mouse click-through | set_ignore_cursor_events + Ctrl+Alt+Shift+P 切换 |
| F06 | Drag to move + resize | start_dragging() IPC + 边缘拖拽调整大小 |
| F07 | Right-click menu | 新建、置顶、设置、导出、隐藏到托盘 |
| F08 | Local persistence | SQLite 本地存储，重启不丢失（含窗口大小记忆） |

### P1 — Important（V1 必须完成）

| ID | Feature | Description |
|----|---------|-------------|
| F09 | Subtasks | 两级树结构，parentId 关联 |
| F10 | Fractional Index drag-sort | Base62 编码，单项重排，全局不洗牌 |
| F11 | Pin notes | 置顶便签固定在列表最前，视觉区分（边框/图标） |
| F12 | Note colors | 8 种预设颜色 |
| F13 | System tray | 关闭时隐藏到托盘，右键菜单 |
| F14 | Auto-start | 开机自动启动 |
| F16 | Tombstone cleanup | 软删除 > 30 天自动清除 |
| F22 | Completion cascade | 完成父任务时自动完成所有子任务 |
| F23 | Undo/Redo | Ctrl+Z 撤销 / Ctrl+Y 重做 |

### P2 — Enhanced（V1 之后）

| ID | Feature | Description |
|----|---------|-------------|
| F15 | Conflict badge | 同步冲突时显示提示图标，可查看被覆盖的旧版本 |
| F17 | WebDAV sync | 坚果云等 WebDAV 同步，近实时三触发点方案 |
| F18 | Due date + reminder | 截止日期 + 定时弹窗提醒 |
| F19 | Export | CSV 导出全部便签 |
| F20 | Search/filter | 关键词搜索 + 颜色筛选 |
| F21 | Todo/Done tabs | 可切换视图，Done 按日期分组 |
| F24 | Note templates | 右键从模板创建，减少重复输入 |
| F25 | Subtask collapse | 折叠已完成子任务或整个便签 |

---

## UI 改进计划

### 第一步：视觉细节打磨（V1 包含）

- **间距与留白：** 统一便签卡片内外间距，增加呼吸感
- **色彩系统：** 建立统一的色彩规范，减少杂乱感
- **字体层次：** 标题/内容/子任务的字号、粗细、颜色要有明确区分
- **毛玻璃主题优化：** 三种配色的细节打磨

### 第二步：信息架构重构（V1 之后）

- **降低信息密度：** 渐进式披露，常用功能突出，高级功能隐藏
- **便签卡片重构：** 简化布局，减少视觉元素
- **功能入口整理：** 合并、隐藏低频功能

---

## 同步方案（P2）

**三触发点近实时同步：**

1. **本地变更时立即推送** — 编辑完成（失焦保存）后立刻上传 WebDAV
2. **窗口获得焦点时拉取** — 检查远程 ETag，有变更则下载
3. **定时轮询兜底** — 每 60 秒检查一次

**冲突策略：** LWW（Last Write Wins）+ 冲突徽章提示，用户可查看被覆盖的旧版本。

---

## Non-functional

| ID | Requirement | Target |
|----|-------------|--------|
| NF01 | Memory | < 50MB resident |
| NF02 | Cold start | < 2s |
| NF03 | Installer size | < 20MB |
| NF04 | OS | Windows 10 1903+ / 11 |
| NF05 | DPI | 100%/125%/150%/200% no handle leak |
| NF06 | WebDAV concurrency | ETag lock, 412 retry |
| NF07 | Sort consistency | Frontend ASCII byte-compare matches SQLite COLLATE BINARY |
| NF08 | Language | Chinese UI |

## Out of scope (V1)

- Android / mobile
- Multi-user collaboration
- Markdown rendering
- Image attachments
- Voice input
- Gzip binary sync
- Field-level merge
- 自动更新机制
- 多语言支持

## V1 完成标准

- 所有 P0 功能稳定运行
- P1 功能基本完成（允许少量边缘情况未处理）
- UI 第一步优化完成（视觉细节打磨）
- 没有已知的崩溃或数据丢失 bug
- 文档更新（README、需求文档、变更日志）

## Reference

[xiajingren/xhznl-todo-list](https://github.com/xiajingren/xhznl-todo-list)
