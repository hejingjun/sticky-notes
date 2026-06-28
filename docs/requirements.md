# Requirements Document v4

## Project
A Windows desktop sticky-notes / todo app. Glass UI, deep system integration, local-first with WebDAV sync.

## P0 鈥?MVP

| ID | Feature | Description |
|----|---------|-------------|
| F01 | Note CRUD | Create, edit, delete notes |
| F02 | Todo check | Toggle complete/incomplete with visual distinction |
| F03 | Frameless glass window | No title bar, rounded corners, backdrop-filter blur |
| F04 | Win+D survival | WM_SHOWWINDOW guard via SetWindowSubclass |
| F05 | Mouse click-through | set_ignore_cursor_events + Ctrl+Alt+P fallback |
| F06 | Drag to move + resize | start_dragging() IPC + resizable edge drags | start_dragging() IPC on handle area |
| F07 | Right-click menu | New note, toggle on-top, settings, export, hide to tray |
| F08 | Local persistence | SQLite, survives restart |

## P1 鈥?Important

| ID | Feature | Description |
|----|---------|-------------|
| F09 | Subtasks | Two-level tree via parentId |
| F10 | Fractional Index drag-sort | Base62, single-item reorder, no global reshuffle |
| F11 | Pin notes | Pinned items always on top |
| F12 | Note colors | 6-8 preset colors |
| F13 | System tray | Minimize to tray, right-click menu |
| F14 | Auto-start | Launch on Windows boot
| F14a | Remember window size | Save/restore last window dimensions on restart | Launch on Windows boot |
| F15 | Conflict badge | Show conflict icon when sync collision, user picks version |
| F16 | Tombstone cleanup | Soft-delete > 30 days auto-purge |

## P2 鈥?Enhanced

| ID | Feature | Description |
|----|---------|-------------|
| F17 | WebDAV sync | Nutstore/Synology/Nextcloud, ETag lock, entity LWW merge |
| F18 | Due date + reminder | Set deadline, notification popup |
| F19 | Export | CSV with trajectory fields (status, type, duration) |
| F20 | Search/filter | Keyword (title + subtasks), color |
| F21 | Todo/Done tabs | Switchable views, Done grouped by date |
| F22 | Completion cascade | Completing parent auto-completes all subtasks |

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
- Gzip binary sync (plain text preserves WebDAV version history)
- Field-level merge (entity LWW prevents tree corruption)

## Roadmap

```
Spike (3d)  鈫?Technical probes: subclass / fractional-index / DPI
P0 (1w)     鈫?Usable sticky notes
P1 (1w)     鈫?Full experience + conflict handling
P2 (1w)     鈫?Cloud sync + enhanced features
```

## Reference

[xiajingren/xhznl-todo-list](https://github.com/xiajingren/xhznl-todo-list)