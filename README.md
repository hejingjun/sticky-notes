<!--
 * @LastEditTime: 2026-06-29
-->

# Sticky Notes — 桌面便签

一款优雅、轻巧的 Windows 桌面便签工具。毛玻璃界面、拖拽排序、子任务、WebDAV 同步。

![screenshot](docs/screenshot.png)

## 功能

| 功能 | 说明 |
|------|------|
| 📝 便签 CRUD | 创建、编辑、删除，标题 + 内容，自动保存 |
| ✅ Todo/Done | 点击复选框标记完成，Todo/Done 标签页分组查看 |
| 🔄 拖拽排序 | 直接拖拽便签调整顺序 |
| 📌 便签置顶 | 重要便签固定在列表最前 |
| 🎨 便签颜色 | 8 种预设颜色（灰/红/橙/黄/绿/青/蓝/紫） |
| 📂 子任务 | 每个便签可添加子任务，独立勾选/编辑/删除 |
| ❄️ 毛玻璃主题 | 🌲松绿 / 🌊雾蓝 / 🪨暖灰 三种配色，透明度可调（10%-90%） |
| 🖱️ 鼠标穿透 | `Ctrl+Alt+Shift+P` 快捷键切换穿透模式 |
| 📌 窗口置顶 | `Ctrl+Shift+T` 切换窗口置顶，始终在其他窗口之上 |
| 🖥️ 嵌入桌面 | 窗口嵌入桌面图标层下方，成为桌面小组件 |
| 🗔 系统托盘 | 关闭时隐藏到托盘，不干扰桌面 |
| 🔄 WebDAV 同步 | 支持坚果云等 WebDAV 网盘同步 |
| ⏰ 截止日期 | 设置截止时间和提醒，定时弹窗通知 |
| 🔍 搜索筛选 | 关键词搜索 + 颜色筛选 |
| 📤 数据导出 | CSV 导出全部便签 |
| 🚀 开机自启 | 设置开机自动启动 |
| 💾 自动保存 | 编辑失焦自动保存，内容不丢失 |
| 🗑️ 自动清理 | 已删除便签超过 30 天自动清除 |

## 快速开始（使用）

### 方式一：下载 Release（推荐）

1. 前往 [Releases](https://github.com/hejingjun/sticky-notes/releases) 页面
2. 下载最新版本的 `sticky-notes.exe`
3. 双击运行即可

> 无需安装，单文件运行。数据保存在 `%APPDATA%/sticky-notes/` 目录下。

### 方式二：从源码构建

#### 环境要求

| 依赖 | 版本 | 说明 |
|------|------|------|
| [Node.js](https://nodejs.org/) | 18+ | 前端构建 |
| [Rust](https://www.rust-lang.org/tools/install) | 最新 stable | 后端编译 |
| [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) | 最新 | C++ 编译工具链（MSVC） |
| [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) | - | Win11 已内置，Win10 需单独安装 |

#### 构建步骤

```bash
# 1. 克隆仓库
git clone https://github.com/hejingjun/sticky-notes.git
cd sticky-notes

# 2. 安装前端依赖
npm install

# 3. 开发模式（热重载）
npm run tauri dev

# 4. 生产构建
npm run tauri build
```

构建完成后，EXE 文件位于：
```
src-tauri/target/release/sticky-notes.exe
```

或直接运行 `build.bat`（Windows）一键构建，产物输出到 `dist/sticky-notes.exe`。

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+Shift+T` | 切换窗口置顶 |
| `Ctrl+Alt+Shift+P` | 切换鼠标穿透（可在设置中修改） |

## WebDAV 同步配置

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

> 同步使用 `notes.json` 纯文本格式 + ETag 乐观锁 + LWW 合并策略，多设备编辑不会丢失数据。

## 系统要求

- Windows 10 1903+ / Windows 11
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)（通常 Win11 已内置）
- 运行内存 < 50MB，磁盘占用 < 20MB

## 技术栈

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
│   ├── App.vue               # 主应用（标签页/提醒弹窗/主题）
│   ├── components/
│   │   ├── NoteCard.vue      # 便签卡片（编辑/颜色/日期/拖拽）
│   │   ├── NoteList.vue      # 便签列表（搜索/筛选/Todo/Done）
│   │   ├── ContextMenu.vue   # 右键菜单
│   │   └── SettingsModal.vue # 设置面板（WebDAV/主题/快捷键/自启）
│   ├── composables/
│   │   └── useNotes.ts       # 便签数据逻辑（排序/CRUD/拖拽）
│   ├── types/note.ts         # 类型定义 + 颜色常量
│   └── styles/glass.css      # 毛玻璃主题样式
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs            # 主入口（命令注册/托盘/嵌入桌面）
│   │   ├── commands.rs       # Tauri 命令（CRUD/导出/提醒）
│   │   ├── db.rs             # SQLite 数据库层
│   │   ├── shortcuts.rs      # 设置持久化（JSON 文件）
│   │   ├── sync/mod.rs       # WebDAV 同步（MKCOL/GET/PUT/LWW）
│   │   ├── tray_icon.rs      # 系统托盘图标
│   │   └── win32/            # Win32 API（桌面嵌入/窗口子类化）
│   ├── icons/                # 应用图标（多尺寸）
│   └── Cargo.toml            # Rust 依赖
├── build.bat                 # Windows 一键构建脚本
└── package.json              # 前端依赖
```

## 数据存储

| 文件 | 路径 | 说明 |
|------|------|------|
| 数据库 | `%APPDATA%/sticky-notes/notes.db` | SQLite 数据库 |
| 设置 | `%APPDATA%/sticky-notes/shortcuts.json` | 快捷键、WebDAV 配置等 |
| 同步状态 | `%APPDATA%/sticky-notes/.sync_etag` | WebDAV ETag |

## 致谢

- 本项目使用 [Claude Code](https://claude.ai/code) 作为 AI 辅助开发工具
- 设计参考 [xiajingren/xhznl-todo-list](https://github.com/xiajingren/xhznl-todo-list)

## 许可证

MIT
