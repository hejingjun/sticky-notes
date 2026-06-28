# Architecture Document v4 (Final)

## 1. Spike Results

Both Electron and Tauri passed all functional tests. **Tauri 2.x selected.**
Memory: Tauri 27.3MB vs Electron 150MB.

## 2. Tech Stack

```
Desktop: Tauri 2.x
UI:      Vue 3 + Vite 6 + TypeScript
Style:   CSS backdrop-filter glass
Win32:   windows-sys 0.59 (official MS crate)
Subclass: comctl32 SetWindowSubclass (safe chain, no WndProc overwrite)
Win+D:   WM_SHOWWINDOW intercept
Penetrate: set_ignore_cursor_events + Ctrl+Alt/P global fallback
Drag:    start_dragging() IPC
HTTP:    reqwest + ETag optimistic locking
Sync:    notes.json (plain text) + entity-level LWW merge
Storage: SQLite (tauri-plugin-sql)
Sort:    Fractional Indexing (Base62, max 32 bytes, rebalance with 3 gates)
Cleanup: Tombstone TTL 30 days
```

## 3. Win+D Guard

SetWindowSubclass intercepts WM_SHOWWINDOW (wParam=0, lParam=0).
Spawns thread to ShowWindow(SW_SHOWNOACTIVATE) within 1ms.
All other messages pass through DefSubclassProc.

## 4. Penetration

Tauri set_ignore_cursor_events(true) enables full-window pass-through.
Ctrl+Alt/P global shortcut restores interaction.
WM_NCHITTEST approach abandoned — incompatible with WebView2 child HWND.

## 5. Data Model

```typescript
interface Note {
  id: string;            // UUID
  title: string;
  parentId: string | null;
  order: string;         // Base62 fractional index (max 32 bytes)
  completed: boolean;
  pinned: boolean;
  color: string;
  createdAt: number;
  updatedAt: number;
  deletedAt: number | null;  // tombstone
  conflictId: string | null;
  dueDate: number | null;
  remindAt: number | null;
  completedAt: number | null; // completion timestamp
}
```

## 6. Fractional Indexing

Charset: 0-9A-Za-z (Base62). Max 32 bytes per order string.
Rebalance conditions: order > 32 bytes AND dirty_count == 0 AND just_synced.
Rebalance redistributes all nodes in level to uniform short strings.

## 7. WebDAV Sync

- Transport: notes.json (plain text, preserves server version history)
- Locking: ETag via If-Match header, HTTP 412 triggers re-fetch + merge + re-PUT
- Merge: Entity-level LWW. Same id + different updatedAt → keep newer. Same updatedAt + different fields → conflict badge.
- Orphan: parentId pointing to tombstone → demote to top-level + [Archive] prefix.

## 8. Sort Consistency

Frontend: pure ASCII byte comparison (NO localeCompare).
Backend: SQLite COLLATE BINARY.
Both produce identical ordering.

## 9. Risk Register

| Risk | Level | Mitigation |
|------|-------|------------|
| WebView2 runtime missing | Medium | Bootstrapper installer |
| Mica downgrade on Win10 | Low | CSS backdrop-filter fallback |
| WebDAV server variance | Medium | Test Nutstore first, then Synology |
| Fractional Index rebalance race | Low | Async per-level, SQLite transaction lock |

## 10. Module Layout

```
src-tauri/src/
  main.rs, lib.rs,
  win32/{mod.rs, subclass.rs},
  sync/{mod.rs, webdav.rs, merge.rs, fractional.rs},
  db.rs

src/
  App.vue,
  components/{NoteCard, NoteList, SubTaskList, ContextMenu, ConflictBadge},
  composables/{useNotes, useDrag, useSort},
  styles/glass.css,
  types/note.ts
```

## 11. Pre-dev Spikes (all completed)

- [x] Spike A: SetWindowSubclass + Win+D guard
- [x] Spike B: Fractional Indexing (8/8 tests pass)
- [x] Spike C: Penetration validated (set_ignore_cursor_events + keyboard escape)