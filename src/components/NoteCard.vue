<script setup lang="ts">
import { ref, computed, watch } from "vue";
import type { Note } from "../types/note";
import ColorPicker from "./ColorPicker.vue";
import DueDatePicker from "./DueDatePicker.vue";

const props = defineProps<{ note: Note; allNotes: Note[] }>();
const emit = defineEmits<{
  toggle: [note: Note];
  update: [note: Note];
  remove: [id: string];
  addSubtask: [parentId: string];
  editing: [v: boolean];
  reorder: [draggedId: string, beforeId: string | null];
  'resolve-conflict': [noteId: string];
}>();

// Main editing
const editing = ref(false);
const editTitle = ref("");
watch(editing, (v) => emit("editing", v));

function startEdit() {
  editTitle.value = props.note.title;
  editing.value = true;
}
function save() {
  editing.value = false;
  if (editTitle.value !== props.note.title) {
    emit("update", { ...props.note, title: editTitle.value });
  }
}
function cancel() { editing.value = false; }

// Subtask editing
const subEditingId = ref<string | null>(null);
const subEditTitle = ref("");

function startSubEdit(s: Note) {
  subEditingId.value = s.id;
  subEditTitle.value = s.title;
}
function saveSubEdit(s: Note) {
  if (subEditingId.value !== s.id) return;
  subEditingId.value = null;
  if (subEditTitle.value !== s.title) {
    emit("update", { ...s, title: subEditTitle.value });
  }
}
function cancelSubEdit() { subEditingId.value = null; }

const subtasks = computed(() =>
  props.allNotes.filter((n) => n.parent_id === props.note.id)
);

const displayTitle = computed(() => props.note.title || "新便签");

// Subtask collapse
const collapsed = ref(false);
const subtaskSummary = computed(() => {
  const total = subtasks.value.length;
  if (total === 0) return "";
  const done = subtasks.value.filter((s) => s.completed).length;
  return `${done}/${total} 已完成`;
});

// Color picker
const showColors = ref(false);

// Due date picker
const showDuePicker = ref(false);
const subDueId = ref<string | null>(null);
const isOverdue = computed(() => {
  if (!props.note.due_date || props.note.completed) return false;
  return props.note.due_date < Date.now();
});

function onDueSave(due: number | null, remind: number | null) {
  if (subDueId.value) {
    const s = props.allNotes.find((n) => n.id === subDueId.value);
    if (s) emit("update", { ...s, due_date: due, remind_at: remind });
    subDueId.value = null;
  } else {
    emit("update", { ...props.note, due_date: due, remind_at: remind });
  }
  showDuePicker.value = false;
}
function onDueClear() {
  if (subDueId.value) {
    const s = props.allNotes.find((n) => n.id === subDueId.value);
    if (s) emit("update", { ...s, due_date: null, remind_at: null });
    subDueId.value = null;
  } else {
    emit("update", { ...props.note, due_date: null, remind_at: null });
  }
  showDuePicker.value = false;
}

// Drag — [DRAG-FIX-1] 用非响应式变量避免 dragover 频繁触发 re-render
let _isDragOver = false;
const dragging = ref(false);

function onDragStart(e: DragEvent) {
  dragging.value = true; // [DRAG-FIX-2] 用 ref 只在 dragstart/dragend 时触发一次 re-render
  e.dataTransfer?.setData("text/plain", props.note.id);
  if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
}
function onDragEnd() {
  dragging.value = false;
  // [DRAG-FIX-3] dragend 后清理 dragOver 样式（非响应式，直接操作 DOM）
  const card = document.querySelector('.card.drag-over');
  if (card) card.classList.remove('drag-over');
}
function onDragOver(e: DragEvent) {
  // [DRAG-FIX-4] 不修改 reactive ref，避免频繁 re-render 破坏拖拽
  if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
  // [DRAG-FIX-5] 用 DOM 操作添加样式，不用 Vue 响应式
  const el = e.currentTarget as HTMLElement;
  if (!_isDragOver) {
    _isDragOver = true;
    el.classList.add('drag-over');
  }
}
function onDragLeave(e: DragEvent) {
  // [DRAG-FIX-6] 只在真正离开 card 时移除样式（检查 relatedTarget）
  const el = e.currentTarget as HTMLElement;
  const related = e.relatedTarget as HTMLElement | null;
  if (related && el.contains(related)) return; // 还在 card 内部，不处理
  _isDragOver = false;
  el.classList.remove('drag-over');
}
function onDrop(e: DragEvent) {
  _isDragOver = false;
  (e.currentTarget as HTMLElement).classList.remove('drag-over');
  const draggedId = e.dataTransfer?.getData("text/plain");
  if (draggedId && draggedId !== props.note.id) {
    emit("reorder", draggedId, props.note.id);
  }
}
</script>

<template>
  <div
    class="card"
    :class="{ editing, dragging }"
    :draggable="!editing"
    :style="{ borderLeftColor: note.color }"
    @dragstart="onDragStart"
    @dragend="onDragEnd"
    @dragenter.prevent
    @dragover.prevent="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <!-- Collapsed view -->
    <div
      v-if="!editing"
      class="row"
      @click="startEdit"
    >
      <button class="check" :class="{ done: note.completed }" @click.stop="emit('toggle', note)">
        {{ note.completed ? '✓' : '' }}
      </button>
      <div class="info">
        <span class="title-text" :class="{ done: note.completed }">{{ displayTitle }}</span>
        <span v-if="isOverdue" class="overdue-badge">逾期</span>
        <span v-else-if="note.due_date" class="due-badge">{{ new Date(note.due_date).toLocaleDateString("zh-CN", { month: "2-digit", day: "2-digit" }) }}</span>
      </div>
      <!-- Hover actions -->
      <div class="hover-actions">
        <button v-if="note.conflict_id" class="conflict-btn" @click.stop="emit('resolve-conflict', note.id)" title="有冲突，点击解决">⚠️</button>
        <DueDatePicker
          :due-date="note.due_date"
          :remind-at="note.remind_at"
          :show="showDuePicker && !subDueId"
          :is-overdue="isOverdue"
          @save="onDueSave"
          @clear="onDueClear"
          @toggle="showDuePicker = !showDuePicker"
        />
        <ColorPicker
          :color="note.color"
          :show="showColors"
          @select="(c) => { emit('update', { ...note, color: c }); showColors = false; }"
          @close="showColors = !showColors"
        />
        <button class="icon-btn" @click.stop="startEdit" title="编辑">✎</button>
        <button class="del" @click.stop="emit('remove', note.id)" title="删除">&times;</button>
      </div>
      <button class="pin-btn" :class="{ pinned: note.pinned }" @click.stop="emit('update', { ...note, pinned: !note.pinned })" title="置顶">📌</button>
    </div>

    <!-- Editing view -->
    <div v-else class="edit-form">
      <input
        v-model="editTitle"
        class="edit-title"
        placeholder="标题"
        @keydown.escape="cancel"
        @keydown.enter="save"
      />
      <div class="edit-actions">
        <button class="act save" @mousedown.prevent="save">保存</button>
        <button class="act cancel" @mousedown.prevent="cancel">取消</button>
      </div>
    </div>

    <!-- Subtasks -->
    <div class="subs" v-if="subtasks.length > 0">
      <!-- Collapsed state: single row with triangle + summary -->
      <div v-if="collapsed" class="sub-row collapse-summary-row" @click.stop="collapsed = false">
        <span class="collapse-icon">▸</span>
        <span class="collapse-text">{{ subtaskSummary }}</span>
      </div>
      <!-- Expanded state: subtask list with triangle on first row -->
      <template v-else>
        <div v-for="(s, idx) in subtasks" :key="s.id" class="sub-row" :class="{ editing: subEditingId === s.id }">
          <button v-if="idx === 0" class="collapse-icon-btn" @click.stop="collapsed = true">▾</button>
          <span v-else class="collapse-spacer"></span>
          <button class="check" :class="{ done: s.completed }" @click="emit('toggle', s)">
            {{ s.completed ? '✓' : '' }}
          </button>
          <template v-if="subEditingId !== s.id">
            <div class="sub-info" @click="startSubEdit(s)">
              <span class="sub-title" :class="{ done: s.completed }">{{ s.title || '子任务' }}</span>
              <span v-if="s.due_date && s.due_date < Date.now() && !s.completed" class="overdue-badge">逾期</span>
              <span v-else-if="s.due_date" class="due-badge">{{ new Date(s.due_date).toLocaleDateString("zh-CN", { month: "2-digit", day: "2-digit" }) }}</span>
            </div>
            <div class="sub-hover-actions">
              <DueDatePicker
                v-if="subDueId === s.id"
                :due-date="s.due_date"
                :remind-at="s.remind_at"
                :show="showDuePicker"
                :is-overdue="s.due_date ? s.due_date < Date.now() && !s.completed : false"
                @save="onDueSave"
                @clear="onDueClear"
                @toggle="showDuePicker = !showDuePicker"
              />
              <button v-else class="due-btn-sm" @click.stop="subDueId = s.id; showDuePicker = true" title="截止日期">📅</button>
              <button class="del" @click="emit('remove', s.id)" title="删除">&times;</button>
            </div>
          </template>
          <div v-else class="sub-edit">
            <input v-model="subEditTitle" class="edit-title" placeholder="子任务标题" @keydown.escape="cancelSubEdit" @keydown.enter="saveSubEdit(s)" />
            <div class="edit-actions">
              <button class="act save" @mousedown.prevent="saveSubEdit(s)">保存</button>
              <button class="act cancel" @mousedown.prevent="cancelSubEdit">取消</button>
            </div>
          </div>
        </div>
        <button class="add-sub" @click="emit('addSubtask', note.id)">+ 子任务</button>
      </template>
    </div>
    <!-- Add subtask when no subtasks exist -->
    <div v-else class="subs">
      <button class="add-sub" @click="emit('addSubtask', note.id)">+ 子任务</button>
    </div>
  </div>
</template>

<style scoped>
.card {
  background: var(--surface-1);
  border-left: 3px solid #666;
  border-radius: 8px;
  padding: var(--sp-md);
  transition: all var(--duration) var(--ease);
  position: relative;
  cursor: grab;
}
.card:active { cursor: grabbing; }
.card:hover { background: var(--surface-2); }
.card.drag-over {
  border-left-color: var(--accent) !important;
  background: var(--accent-dim);
}
.card.dragging { opacity: 0.4; }
.card.editing { background: rgba(30, 30, 30, 0.95); }

.row {
  display: flex;
  align-items: flex-start;
  gap: var(--sp-sm);
  cursor: pointer;
}

.check {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  border: 1.5px solid var(--border-medium);
  background: transparent;
  color: var(--accent);
  cursor: pointer;
  font-size: var(--font-sm);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 2px;
  transition: all var(--duration) var(--ease);
}
.check.done {
  background: var(--accent-dim);
  border-color: var(--accent);
}

.info {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--sp-xs);
  flex-wrap: wrap;
}

.title-text {
  font-size: var(--font-md);
  color: var(--text-primary);
  word-break: break-word;
  line-height: 1.4;
}
.title-text.done {
  text-decoration: line-through;
  color: var(--text-tertiary);
}

/* Due date badges — always visible in info area */
.due-badge, .overdue-badge {
  font-size: var(--font-xs);
  padding: 1px 6px;
  border-radius: 4px;
  flex-shrink: 0;
}
.due-badge { background: var(--surface-2); color: var(--text-tertiary); }
.overdue-badge { background: var(--danger-dim); color: var(--danger); }

/* Pin — always visible but subtle */
.pin-btn {
  background: none;
  border: none;
  cursor: pointer;
  flex-shrink: 0;
  font-size: var(--font-base);
  padding: 0 2px;
  line-height: 1;
  opacity: 0.2;
  transition: all var(--duration) var(--ease);
}
.pin-btn:hover { opacity: 0.7; }
.pin-btn.pinned { opacity: 1; filter: drop-shadow(0 0 2px var(--warning-dim)); }

/* Hover actions — hidden by default, shown on row hover */
.hover-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0;
  pointer-events: none;
  transition: opacity var(--duration) var(--ease);
}
.row:hover .hover-actions { opacity: 1; pointer-events: auto; }

.icon-btn {
  background: none;
  border: none;
  color: var(--text-disabled);
  cursor: pointer;
  font-size: var(--font-md);
  flex-shrink: 0;
  padding: 0 2px;
  transition: color var(--duration) var(--ease);
}
.icon-btn:hover { color: var(--text-secondary); }

.del {
  background: none;
  border: none;
  color: var(--text-disabled);
  cursor: pointer;
  font-size: 16px;
  flex-shrink: 0;
  line-height: 1;
  transition: color var(--duration) var(--ease);
}
.del:hover { color: var(--danger); }

.conflict-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: var(--font-md);
  padding: 0 2px;
  animation: conflict-pulse 2s ease-in-out infinite;
}
@keyframes conflict-pulse {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}

.edit-form { display: flex; flex-direction: column; gap: var(--sp-sm); }

.edit-title {
  width: 100%;
  background: var(--surface-2);
  border: 1px solid var(--border-medium);
  border-radius: 5px;
  color: var(--text-primary);
  font-size: var(--font-md);
  padding: 7px 10px;
  outline: none;
  transition: border-color var(--duration) var(--ease);
}
.edit-title:focus { border-color: var(--accent); }

.edit-actions { display: flex; gap: var(--sp-sm); justify-content: flex-end; }

.act {
  padding: 4px 14px;
  border: none;
  border-radius: 5px;
  font-size: var(--font-sm);
  cursor: pointer;
  transition: all var(--duration) var(--ease);
}
.act.save { background: var(--accent-dim); color: var(--accent); }
.act.cancel { background: var(--surface-1); color: var(--text-secondary); }
.act.save:hover { background: var(--accent-glow); }
.act.cancel:hover { background: var(--surface-3); }

/* ---- Subtasks ---- */
.subs {
  margin-top: var(--sp-sm);
  margin-left: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sub-row {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  padding: 5px 8px;
  border-radius: 5px;
  font-size: var(--font-base);
  transition: background var(--duration) var(--ease);
}
.sub-row:hover { background: var(--surface-1); }
.sub-row.editing { background: rgba(40, 40, 40, 0.9); }

.sub-info {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--sp-xs);
  cursor: pointer;
}

.sub-title {
  color: var(--text-secondary);
  word-break: break-word;
  font-size: var(--font-base);
}
.sub-title.done {
  text-decoration: line-through;
  color: var(--text-tertiary);
}

/* Subtask hover actions */
.sub-hover-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0;
  pointer-events: none;
  transition: opacity var(--duration) var(--ease);
}
.sub-row:hover .sub-hover-actions { opacity: 1; pointer-events: auto; }

.sub-edit { display: flex; flex-direction: column; gap: var(--sp-xs); width: 100%; }
.sub-row .check { width: 16px; height: 16px; font-size: 10px; }
.sub-row .del { font-size: 14px; }

.due-btn-sm {
  background: none;
  border: none;
  cursor: pointer;
  font-size: var(--font-sm);
  flex-shrink: 0;
  padding: 0 2px;
  line-height: 1;
  opacity: 0.4;
  transition: opacity var(--duration) var(--ease);
}
.due-btn-sm:hover { opacity: 0.8; }

.add-sub {
  background: none;
  border: none;
  color: var(--text-tertiary);
  font-size: var(--font-sm);
  cursor: pointer;
  padding: 3px 8px;
  border-radius: 5px;
  text-align: left;
  transition: all var(--duration) var(--ease);
}
.add-sub:hover {
  color: var(--text-secondary);
  background: var(--surface-0);
}

/* ---- Collapse toggle ---- */
.collapse-icon-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 16px;
  line-height: 1.2;
  color: var(--text-secondary);
  padding: 0;
  flex-shrink: 0;
  width: 16px;
  text-align: center;
  transition: color var(--duration) var(--ease);
}
.collapse-icon-btn:hover { color: var(--text-primary); }

.collapse-spacer {
  width: 16px;
  flex-shrink: 0;
}

.collapse-summary-row {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px;
  border-radius: 5px;
  transition: background var(--duration) var(--ease);
}
.collapse-summary-row:hover { background: var(--surface-1); }
.collapse-summary-row .collapse-icon {
  font-size: 14px;
  line-height: 1;
  color: var(--text-tertiary);
}
.collapse-text {
  font-size: var(--font-sm);
  color: var(--text-tertiary);
}
</style>
