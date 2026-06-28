<!--
 * @LastEditTime: 2025-06-22
-->

# Sticky Notes — 桌面便签

一款优雅、轻巧的 Windows 桌面便签工具。毛玻璃界面、拖拽排序、子任务、WebDAV 同步。

![screenshot](docs/screenshot.png)

## 功能

| 功能 | 说明 |
|------|------|
| 📝 便签 CRUD | 创建、编辑、删除，标题 + 内容 |
| ✅ Todo 勾选 | 点击复选框标记完成/未完成 |
| 🔄 拖拽排序 | 直接拖拽便签调整顺序 |
| 📌 便签置顶 | 重要便签固定在列表最前 |
| 🎨 便签颜色 | 8 种预设颜色（灰/红/橙/黄/绿/青/蓝/紫） |
| 📂 子任务 | 每个便签可添加子任务，独立勾选/编辑/删除 |
| ❄️ 毛玻璃主题 | 🌲松绿 / 🌊雾蓝 / 🪨暖灰 三种配色 |
| 🖱️ 鼠标穿透 | `Ctrl+Alt+Shift+P` 快捷键切换穿透模式（仅快捷键控制） |
| 📌 窗口置顶 | 始终保持在其他窗口之上 |
| 🗔 系统托盘 | 关闭时隐藏到托盘，不干扰桌面 |
| 🔄 WebDAV 同步 | 支持坚果云等 WebDAV 网盘同步 |
| ⏰ 截止日期 | 设置截止时间和提醒 |
| 🔍 搜索筛选 | 关键词搜索 + 颜色筛选 |
| 📤 导出 CSV | 导出全部便签到 CSV 文件 |
| 🚀 开机自启 | 设置开机自动启动 |
| 💾 自动保存 | 编辑失焦自动保存，内容不丢失 |

## 系统要求

- Windows 10 1903+ / Windows 11
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)（通常 Win11 已内置）
- 内存 < 50MB

## 快速开始

1. 下载 `sticky-notes.zip`
2. 解压后运行 `sticky-notes.exe`
3. 右键点击空白区域弹出菜单

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Alt+Shift+P` | 切换鼠标穿透（可在设置中修改） |

### WebDAV 同步配置

1. 右键 → **设置**
2. 填写 **服务器地址**、**用户名**、**密码**
3. 点击 **保存配置** → **立即同步**

#### 推荐免费 WebDAV 服务

| 服务 | 地址格式 | 说明 |
|------|----------|------|
| [坚果云](https://www.jianguoyun.com) | `https://dav.jianguoyun.com/dav/` | 国内最快，月 1GB 上传流量，需用[应用密码](https://help.jianguoyun.com/?p=2064) |
| [InfiniCLOUD](https://infini-cloud.net/) | `https://webdav.infini-cloud.net/` | 日本服务，免费 20GB+，速度尚可 |
| [Koofr](https://koofr.eu/) | `https://app.koofr.net/dav/` | 欧洲服务，稳定性高 |
| 自建 Alist | 自定 | 适合有 NAS 的用户 |

> 同步使用 notes.json 纯文本格式 + ETag 乐观锁 + LWW 合并策略，多设备编辑不会丢失数据。

## 开发

```bash
# 安装依赖
pnpm install

# 开发模式
pnpm tauri dev

# 构建
pnpm tauri build
```

### 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | Tauri 2.x (Rust) |
| 前端 UI | Vue 3 + Vite 6 + TypeScript |
| 样式 | CSS Glassmorphism (毛玻璃) |
| 存储 | SQLite (rusqlite) |
| 同步 | reqwest WebDAV + ETag + LWW Merge |
| Win32 API | windows-sys 0.59 |

## 项目结构

```
sticky-notes/
├── src/                      # Vue 3 前端
│   ├── App.vue               # 主应用（提醒弹窗、主题）
│   ├── components/
│   │   ├── NoteCard.vue      # 便签卡片（编辑/颜色/日期/拖拽）
│   │   ├── NoteList.vue      # 便签列表（搜索/筛选）
│   │   ├── ContextMenu.vue   # 右键菜单
│   │   └── SettingsModal.vue # 设置面板
│   ├── composables/
│   │   └── useNotes.ts       # 便签数据逻辑（排序/CRUD/拖拽）
│   ├── types/note.ts         # 类型定义 + 颜色常量
│   └── styles/glass.css      # 毛玻璃主题样式
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs            # 主入口（命令注册/托盘/嵌入桌面）
│   │   ├── commands.rs       # Tauri 命令
│   │   ├── db.rs             # SQLite 数据库
│   │   ├── shortcuts.rs      # 设置持久化
│   │   ├── sync/mod.rs       # WebDAV 同步
│   │   └── win32/            # Win32 API 样式/子类化
│   └── Cargo.toml
└── package.json
```

## 致谢

- 本项目使用 [Claude Code](https://claude.ai/code) 作为 AI 辅助开发工具
- 设计参考 [xiajingren/xhznl-todo-list](https://github.com/xiajingren/xhznl-todo-list)

## 许可证

MIT
