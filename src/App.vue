<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useNotes } from "./composables/useNotes";
import NoteList from "./components/NoteList.vue";
import ContextMenu from "./components/ContextMenu.vue";
import SettingsModal from "./components/SettingsModal.vue";

const { notes, load, add, remove, toggleComplete, update, addSubtask, reorder } = useNotes();
onMounted(() => load());

const penetrating = ref(false);
const ontop = ref(true);
const showSettings = ref(false);
const isEditing = ref(false);
const reminders = ref<Array<{ id: string; title: string }>>([]);
let unlistenPen: (() => void) | null = null;
let unlistenTop: (() => void) | null = null;
let unlistenReload: (() => void) | null = null;
let reminderTimer: ReturnType<typeof setInterval> | null = null;

watch(isEditing, (v) => {
  document.body.classList.toggle("editing", v);
});

onMounted(async () => {
  try {
    penetrating.value = await invoke<boolean>("get_penetrate");
    ontop.value = await invoke<boolean>("get_ontop");
  } catch (e) {
    console.warn("读取初始状态失败:", e);
  }
  try {
    unlistenPen = await listen<boolean>("penetrate-changed", (e) => {
      penetrating.value = e.payload;
    });
    unlistenTop = await listen<boolean>("ontop-changed", (e) => {
      ontop.value = e.payload;
    });
    const cfg: { theme: string } = await invoke("get_settings");
    document.body.classList.add("theme-" + (cfg.theme || "green"));
    await checkReminders();
    reminderTimer = setInterval(checkReminders, 30_000);
    const unload = await listen("notes-reloaded", () => load());
    unlistenReload = unload;
  } catch (e) {
    console.warn("初始化事件监听失败:", e);
  }
});
onUnmounted(() => {
  unlistenPen?.();
  unlistenTop?.();
  unlistenReload?.();
  if (reminderTimer) clearInterval(reminderTimer);
});

async function checkReminders() {
  try {
    const due = await invoke<Array<{ id: string; title: string; content: string }>>("check_reminders");
    for (const n of due) {
      if (!reminders.value.some((r) => r.id === n.id)) {
        reminders.value.push({ id: n.id, title: n.title || "无标题" });
      }
    }
  } catch (e) { console.warn("check_reminders 失败:", e); }
}

function dismissReminder(id: string) {
  reminders.value = reminders.value.filter((r) => r.id !== id);
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
        <div class="reminder-actions">
          <button class="remind-btn" @click="dismissReminder(r.id)">知道了</button>
        </div>
      </div>
    </div>

    <div class="handle" @mousedown="startDrag">
      <span class="title">便签</span>
      <div class="btns">
        <button class="btn" :class="{ active: penetrating }" @mousedown.stop @click="togglePen" title="穿透切换 (Ctrl+Alt+Shift+P)">
          {{ penetrating ? '☑' : '☐' }}
        </button>
        <button class="btn" :class="{ active: ontop }" @mousedown.stop @click="toggleTop" title="置顶切换">
          {{ ontop ? '📌' : '📍' }}
        </button>
        <button class="btn close" @mousedown.stop @click="doClose" title="关闭">✕</button>
      </div>
    </div>
    <div class="body">
      <ContextMenu :on-add="add" :on-pen="togglePen" :on-top="toggleTop" :on-settings="() => showSettings = true" :on-export="exportNotes" :ontop="ontop" :penetrating="penetrating" :on-close="doClose" />
      <NoteList :notes="notes" @toggle="toggleComplete" @update="(n: any) => update(n)" @remove="remove" @add="add" @add-subtask="addSubtask" @editing="(v: boolean) => isEditing = v" @reorder="reorder" />
    </div>
    <div class="hint">Ctrl+Alt+Shift+P 穿透</div>

    <SettingsModal :show="showSettings" @close="showSettings = false" />
  </div>
</template>

<style scoped>
.app { height: 100vh; display: flex; flex-direction: column; border-radius: 12px; overflow: hidden; }
.handle {
  display: flex; align-items: center; justify-content: space-between;
  padding: 6px 12px; background: rgba(255,255,255,0.05);
  border-bottom: 1px solid rgba(255,255,255,0.06); flex-shrink: 0;
  cursor: grab;
}
.handle:active { cursor: grabbing; }
.title { font-size: 12px; color: rgba(255,255,255,0.6); font-weight: 600; user-select: none; flex: 1; }
.btns { display: flex; gap: 4px; }
.btn {
  width: 26px; height: 22px; border: none; border-radius: 5px;
  background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.5);
  cursor: pointer; font-size: 12px;
}
.btn:hover { background: rgba(255,255,255,0.18); color: #fff; }
.btn.active { background: rgba(74,222,128,0.3); color: #4ade80; }
.btn.close:hover { background: rgba(255,80,80,0.4); color: #ff6b6b; }
.body { flex: 1; overflow: hidden; }
.hint { font-size: 10px; color: rgba(255,255,255,0.2); text-align: center; padding: 4px; flex-shrink: 0; }

/* Reminder toast */
.reminder-toast {
  position: absolute; top: 40px; left: 8px; right: 8px; z-index: 999;
  display: flex; align-items: flex-start; gap: 8px;
  background: rgba(30,30,30,0.96); backdrop-filter: blur(16px);
  border: 1px solid rgba(255,200,0,0.3); border-radius: 8px;
  padding: 10px 12px; box-shadow: 0 4px 20px rgba(0,0,0,0.4);
  animation: slideDown 0.25s ease-out;
}
@keyframes slideDown {
  from { transform: translateY(-10px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}
.reminder-icon { font-size: 18px; flex-shrink: 0; }
.reminder-body { flex: 1; display: flex; flex-direction: column; gap: 6px; }
.reminder-title { font-size: 12px; color: #e5c07b; font-weight: 600; }
.remind-btn {
  align-self: flex-start; padding: 2px 12px; border: 1px solid rgba(255,255,255,0.15);
  border-radius: 4px; font-size: 10px; cursor: pointer;
  background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.6);
}
.remind-btn:hover { background: rgba(255,255,255,0.15); color: #fff; }
</style>
