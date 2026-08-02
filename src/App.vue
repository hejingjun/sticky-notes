<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useNotes } from "./composables/useNotes";
import { useTauriEvent } from "./composables/useTauriEvent";
import { useSync } from "./composables/useSync";
import NoteList from "./components/NoteList.vue";
import ContextMenu from "./components/ContextMenu.vue";
import SettingsModal from "./components/SettingsModal.vue";
import ConflictModal from "./components/ConflictModal.vue";

const { notes, load, add, remove, toggleComplete, update, addSubtask, reorder, undo, redo, canUndo, canRedo } = useNotes();
const { startSync, notifyChanged, isSyncing, syncError, webdavConfigured, checkConfig } = useSync();

let skipSync = false;
onMounted(async () => {
  skipSync = true;
  await load();
  skipSync = false;
  await startSync();
});

// Watch notes changes → trigger debounced push (trigger 1)
watch(notes, () => {
  if (!skipSync) notifyChanged();
}, { deep: false });

const penetrating = ref(false);
const ontop = ref(false);
const showSettings = ref(false);
const isEditing = ref(false);
const activeTab = ref<"todo" | "done">((localStorage.getItem("activeTab") as "todo" | "done") || "todo");
const reminders = ref<Array<{ id: string; title: string; content: string }>>([]);
const shortcut = ref("Ctrl+Alt+Shift+P");
const conflictNoteId = ref<string | null>(null);
let reminderTimer: ReturnType<typeof setInterval> | null = null;

// Auto-cleanup event listeners
useTauriEvent<boolean>("penetrate-changed", (e) => { penetrating.value = e.payload; });
useTauriEvent<boolean>("ontop-changed", (e) => { ontop.value = e.payload; });
useTauriEvent("notes-reloaded", async () => { skipSync = true; await load(); skipSync = false; });

watch(isEditing, (v) => {
  document.body.classList.toggle("editing", v);
});

watch(activeTab, (v) => {
  localStorage.setItem("activeTab", v);
});

onMounted(async () => {
  try {
    penetrating.value = await invoke<boolean>("get_penetrate");
    ontop.value = await invoke<boolean>("get_ontop");
  } catch (e) {
    console.warn("读取初始状态失败:", e);
  }
  try {
    const cfg: { theme: string; penetrate: string } = await invoke("get_settings");
    document.body.classList.add("theme-" + (cfg.theme || "green"));
    if (cfg.penetrate) shortcut.value = cfg.penetrate;
    await checkReminders();
    reminderTimer = setInterval(checkReminders, 30_000);
  } catch (e) {
    console.warn("初始化失败:", e);
  }
  // Ctrl+Z / Ctrl+Y undo/redo
  document.addEventListener("keydown", onKeydown);
});
onUnmounted(() => {
  if (reminderTimer) clearInterval(reminderTimer);
  document.removeEventListener("keydown", onKeydown);
});

function onKeydown(e: KeyboardEvent) {
  // Don't intercept if user is typing in an input
  const target = e.target as HTMLElement;
  if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) return;

  if (e.ctrlKey && !e.shiftKey && e.key === "z") {
    e.preventDefault();
    undo();
  } else if (e.ctrlKey && e.key === "y") {
    e.preventDefault();
    redo();
  }
}

async function checkReminders() {
  try {
    const due = await invoke<Array<{ id: string; title: string; content: string }>>("check_reminders");
    for (const n of due) {
      if (!reminders.value.some((r) => r.id === n.id)) {
        reminders.value.push({ id: n.id, title: n.title || "无标题", content: n.content || "" });
      }
    }
  } catch (e) { console.warn("check_reminders 失败:", e); }
}

function dismissReminder(id: string) {
  reminders.value = reminders.value.filter((r) => r.id !== id);
}

async function snoozeReminder(id: string) {
  // Set remind_at to 15 minutes from now
  const newRemindAt = Date.now() + 15 * 60 * 1000;
  try {
    const note = notes.value.find((n) => n.id === id);
    if (note) {
      await update({ ...note, remind_at: newRemindAt });
    }
  } catch (e) {
    console.warn("延后提醒失败:", e);
  }
  dismissReminder(id);
}

async function togglePen() {
  penetrating.value = await invoke<boolean>("toggle_penetrate");
}

async function toggleTop() {
  ontop.value = await invoke<boolean>("toggle_ontop");
}

async function doClose() {
  await invoke("hide_to_tray");
}

function startDrag() {
  invoke("start_drag");
}

async function exportNotes(format: string) {
  try {
    const csv = await invoke<string>("export_notes", { format });
    const [ { save }, { writeTextFile } ] = await Promise.all([
      import("@tauri-apps/plugin-dialog"),
      import("@tauri-apps/plugin-fs"),
    ]);
    const path = await save({
      filters: [{ name: "CSV", extensions: ["csv"] }],
      defaultPath: `便签_${new Date().toISOString().slice(0, 10)}.csv`,
    });
    if (path) {
      await writeTextFile(path, csv);
    }
  } catch (e) {
    console.error("导出失败:", e);
  }
}
</script>

<template>
  <div class="app">
    <!-- Reminder toast -->
    <div v-for="r in reminders" :key="r.id" class="reminder-toast">
      <span class="reminder-icon">⏰</span>
      <div class="reminder-body">
        <div class="reminder-title">{{ r.title }}</div>
        <div v-if="r.content" class="reminder-content">{{ r.content }}</div>
        <div class="reminder-actions">
          <button class="remind-btn" @click="dismissReminder(r.id)">知道了</button>
          <button class="remind-btn snooze" @click="snoozeReminder(r.id)">延后 15 分钟</button>
        </div>
      </div>
    </div>

    <div class="handle" @mousedown="startDrag">
      <span class="title">便签</span>
      <div class="btns">
        <button class="btn" :class="{ active: ontop }" @mousedown.stop @click="toggleTop" title="置顶切换">
          {{ ontop ? '📌' : '📍' }}
        </button>
        <button class="btn close" @mousedown.stop @click="doClose" title="关闭">✕</button>
      </div>
    </div>
    <div class="tabs">
      <button class="tab" :class="{ active: activeTab === 'todo' }" @click="activeTab = 'todo'">待办</button>
      <button class="tab" :class="{ active: activeTab === 'done' }" @click="activeTab = 'done'">已完成</button>
    </div>
    <div class="body">
      <ContextMenu :on-add="add" :on-top="toggleTop" :on-settings="() => showSettings = true" :on-export="exportNotes" :ontop="ontop" :on-close="doClose" />
      <NoteList :notes="notes" :tab="activeTab" @toggle="toggleComplete" @update="(n: any) => update(n)" @remove="remove" @add="add" @add-subtask="addSubtask" @editing="(v: boolean) => isEditing = v" @reorder="reorder" @resolve-conflict="(id: string) => conflictNoteId = id" />
    </div>
    <div class="hint">
      {{ shortcut }} 穿透 · Ctrl+Z 撤销
      <span v-if="isSyncing" class="sync-indicator">同步中...</span>
      <span v-else-if="syncError" class="sync-error" :title="syncError">同步失败</span>
    </div>

    <SettingsModal :show="showSettings" @close="showSettings = false; checkConfig()" @shortcut-updated="(s: string) => shortcut = s" />
    <ConflictModal :note-id="conflictNoteId" @close="conflictNoteId = null" @resolved="load()" />
  </div>
</template>

<style scoped>
.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  border-radius: var(--glass-radius);
  overflow: hidden;
}

/* ---- Handle bar ---- */
.handle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sp-sm) var(--sp-md);
  background: var(--surface-0);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  cursor: grab;
}
.handle:active { cursor: grabbing; }

.title {
  font-size: var(--font-base);
  color: var(--text-secondary);
  font-weight: 600;
  letter-spacing: 0.5px;
  user-select: none;
  flex: 1;
}

.btns { display: flex; gap: var(--sp-xs); }

.btn {
  width: 28px;
  height: 22px;
  border: none;
  border-radius: 5px;
  background: var(--surface-1);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: var(--font-base);
  transition: all var(--duration) var(--ease);
}
.btn:hover { background: var(--surface-3); color: var(--text-primary); }
.btn.active { background: var(--accent-dim); color: var(--accent); }
.btn.close:hover { background: var(--danger-dim); color: var(--danger); }

/* ---- Tabs ---- */
.tabs {
  display: flex;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-subtle);
}

.tab {
  flex: 1;
  padding: 8px 0;
  border: none;
  background: none;
  color: var(--text-tertiary);
  font-size: var(--font-base);
  font-weight: 500;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--duration) var(--ease);
}
.tab:hover { color: var(--text-secondary); }
.tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

/* ---- Body ---- */
.body { flex: 1; overflow: hidden; }

/* ---- Hint ---- */
.hint {
  font-size: var(--font-xs);
  color: var(--text-disabled);
  text-align: center;
  padding: var(--sp-xs) 0;
  flex-shrink: 0;
  letter-spacing: 0.3px;
}

/* ---- Reminder toast ---- */
.reminder-toast {
  position: absolute;
  top: 44px;
  left: var(--sp-sm);
  right: var(--sp-sm);
  z-index: 999;
  display: flex;
  align-items: flex-start;
  gap: var(--sp-sm);
  background: rgba(30, 30, 30, 0.96);
  backdrop-filter: blur(16px);
  border: 1px solid var(--warning-dim);
  border-radius: 8px;
  padding: var(--sp-md);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  animation: slideDown 0.25s var(--ease);
}

@keyframes slideDown {
  from { transform: translateY(-10px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.reminder-icon { font-size: 18px; flex-shrink: 0; }

.reminder-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--sp-sm);
}

.reminder-title {
  font-size: var(--font-base);
  color: var(--warning);
  font-weight: 600;
}

.reminder-content {
  font-size: var(--font-sm);
  color: var(--text-secondary);
  line-height: 1.4;
  max-height: 40px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.reminder-actions { display: flex; gap: var(--sp-sm); }

.remind-btn {
  padding: 3px var(--sp-md);
  border: 1px solid var(--border-light);
  border-radius: 4px;
  font-size: var(--font-xs);
  cursor: pointer;
  background: var(--surface-1);
  color: var(--text-secondary);
  transition: all var(--duration) var(--ease);
}
.remind-btn:hover { background: var(--surface-3); color: var(--text-primary); }
.remind-btn.snooze { border-color: var(--accent); color: var(--accent); }
.remind-btn.snooze:hover { background: var(--accent-dim); }

/* ---- Sync status ---- */
.sync-indicator {
  color: var(--accent);
  margin-left: var(--sp-sm);
  animation: pulse 1.5s ease-in-out infinite;
}
@keyframes pulse {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}
.sync-error {
  color: var(--danger);
  margin-left: var(--sp-sm);
  cursor: help;
}
</style>
