<script setup lang="ts">
import { ref, computed, watch } from "vue";
import type { Note } from "../types/note";
import { COLORS } from "../types/note";

const props = defineProps<{ note: Note; allNotes: Note[] }>();
const emit = defineEmits<{
  toggle: [note: Note];
  update: [note: Note];
  remove: [id: string];
  addSubtask: [parentId: string];
  editing: [v: boolean];
  reorder: [draggedId: string, beforeId: string | null];
}>();

// Self editing
const editing = ref(false);
const editTitle = ref("");
const editContent = ref("");

watch(editing, (v) => emit("editing", v));

function startEdit() {
  editTitle.value = props.note.title;
  editContent.value = props.note.content;
  editing.value = true;
}

function save() {
  editing.value = false;
  if (editTitle.value !== props.note.title || editContent.value !== props.note.content) {
    emit("update", { ...props.note, title: editTitle.value, content: editContent.value });
  }
}

function onBlur() {
  save();
}

function cancel() {
  editing.value = false;
}

// Subtask editing (independent from main editing state)
const subEditingId = ref<string | null>(null);
const subEditTitle = ref("");
const subEditContent = ref("");

function startSubEdit(s: Note) {
  subEditingId.value = s.id;
  subEditTitle.value = s.title;
  subEditContent.value = s.content;
}

function saveSubEdit(s: Note) {
  if (subEditingId.value !== s.id) return;
  subEditingId.value = null;
  if (subEditTitle.value !== s.title || subEditContent.value !== s.content) {
    emit("update", { ...s, title: subEditTitle.value, content: subEditContent.value });
  }
}

function cancelSubEdit() {
  subEditingId.value = null;
}

const subtasks = computed(() =>
  props.allNotes.filter((n) => n.parent_id === props.note.id)
);

const displayTitle = computed(() => props.note.title || "新便签");
const showColors = ref(false);

const subDueId = ref<string | null>(null);
const showDuePicker = ref(false);
const editDueDate = ref("");
const editRemindAt = ref("");

const dueDateStr = computed(() => {
  if (!props.note.due_date) return "";
  return new Date(props.note.due_date).toLocaleDateString("zh-CN", { month: "2-digit", day: "2-digit" });
});
const isOverdue = computed(() => {
  if (!props.note.due_date || props.note.completed) return false;
  return props.note.due_date < Date.now();
});

function initDueEdit(t?: Note) {
  const target = t || props.note;
  editDueDate.value = target.due_date ? new Date(target.due_date).toISOString().slice(0, 16) : "";
  editRemindAt.value = target.remind_at ? new Date(target.remind_at).toISOString().slice(0, 16) : "";
}

function saveDue() {
  const due = editDueDate.value ? new Date(editDueDate.value).getTime() : null;
  const remind = editRemindAt.value ? new Date(editRemindAt.value).getTime() : null;
  if (subDueId.value) {
    const s = props.allNotes.find((n) => n.id === subDueId.value);
    if (s) emit("update", { ...s, due_date: due, remind_at: remind });
    subDueId.value = null;
  } else {
    emit("update", { ...props.note, due_date: due, remind_at: remind });
  }
  showDuePicker.value = false;
}

function clearDue() {
  if (subDueId.value) {
    const s = props.allNotes.find((n) => n.id === subDueId.value);
    if (s) emit("update", { ...s, due_date: null, remind_at: null });
    subDueId.value = null;
  } else {
    emit("update", { ...props.note, due_date: null, remind_at: null });
  }
  showDuePicker.value = false;
}

// Drag state
const dragOver = ref(false);

function onDragStart(e: DragEvent) {
  const el = e.currentTarget as HTMLElement;
  el.style.opacity = "0.4";
  e.dataTransfer?.setData("text/plain", props.note.id);
  if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
}

function onDragEnd(e: DragEvent) {
  (e.currentTarget as HTMLElement).style.opacity = "";
}

function onDragOver(e: DragEvent) {
  dragOver.value = true;
  e.dataTransfer!.dropEffect = "move";
}

function onDragLeave() {
  dragOver.value = false;
}

function onDrop(e: DragEvent) {
  dragOver.value = false;
  const draggedId = e.dataTransfer?.getData("text/plain");
  if (draggedId && draggedId !== props.note.id) {
    // Emit up: reorder(draggedId, beforeId)
    emit("reorder" as any, draggedId, props.note.id);
  }
}
</script>

<template>
  <div
    class="card"
    :class="{ editing, 'drag-over': dragOver }"
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
    <div v-if="!editing" class="row" @click="startEdit">
      <button class="check" :class="{ done: note.completed }" @click.stop="emit('toggle', note)">
        {{ note.completed ? '✓' : '' }}
      </button>
      <div class="info">
        <span class="title-text" :class="{ done: note.completed }">{{ displayTitle }}</span>
        <div class="meta-row">
          <span v-if="note.content" class="preview">{{ note.content.slice(0, 40) }}</span>
          <span v-if="isOverdue" class="overdue-badge">逾期</span>
          <span v-else-if="dueDateStr" class="due-badge">{{ dueDateStr }}</span>
        </div>
      </div>
      <button class="due-btn" :class="{ overdue: isOverdue }" @click.stop="initDueEdit(); showDuePicker = !showDuePicker" title="截止日期">{{ note.due_date ? '📅' : '➕' }}</button>
      <!-- Due picker popup -->
      <div v-if="showDuePicker" class="due-popup" @click.stop @mouseleave="showDuePicker = false">
        <label class="due-field">截止 <input v-model="editDueDate" type="datetime-local" class="due-input" /></label>
        <label class="due-field">提醒 <input v-model="editRemindAt" type="datetime-local" class="due-input" /></label>
        <div class="due-actions">
          <button class="act save" @click="saveDue">保存</button>
          <button class="act cancel" @click="clearDue">清除</button>
        </div>
      </div>
      <button class="pin-btn" :class="{ pinned: note.pinned }" @click.stop="emit('update', { ...note, pinned: !note.pinned })" title="置顶">📌</button>
      <div class="color-wrap">
        <div class="color-dot" :style="{ background: note.color }" @click.stop="showColors = !showColors" title="颜色"></div>
        <div v-if="showColors" class="color-picker" @mouseleave="showColors = false">
          <div v-for="c in COLORS" :key="c" class="color-opt" :class="{ active: note.color === c }" :style="{ background: c }" @click.stop="emit('update', { ...note, color: c }); showColors = false"></div>
        </div>
      </div>
      <button class="icon-btn" @click.stop="startEdit" title="编辑">✎</button>
      <button class="del" @click.stop="emit('remove', note.id)" title="删除">&times;</button>
    </div>

    <!-- Editing view -->
    <div v-else class="edit-form" @blur="onBlur">
      <input
        v-model="editTitle"
        class="edit-title"
        placeholder="标题"
        @keydown.escape="cancel"
      />
      <textarea
        v-model="editContent"
        class="edit-body"
        placeholder="内容..."
        rows="4"
        @keydown.escape="cancel"
      />
      <div class="edit-actions">
        <button class="act save" @mousedown.prevent="save">保存</button>
        <button class="act cancel" @mousedown.prevent="cancel">取消</button>
      </div>
    </div>

    <!-- Subtasks -->
    <div class="subs">
      <div v-for="s in subtasks" :key="s.id" class="sub-row" :class="{ editing: subEditingId === s.id }">
        <button class="check" :class="{ done: s.completed }" @click="emit('toggle', s)">
          {{ s.completed ? '✓' : '' }}
        </button>

        <!-- Sub collapsed -->
        <template v-if="subEditingId !== s.id">
          <div class="sub-info" @click="startSubEdit(s)">
            <span class="sub-title" :class="{ done: s.completed }">{{ s.title || '子任务' }}</span>
            <div class="sub-meta-row">
              <span v-if="s.content" class="sub-preview">{{ s.content.slice(0, 30) }}</span>
              <span v-if="s.due_date && s.due_date < Date.now() && !s.completed" class="overdue-badge">逾期</span>
              <span v-else-if="s.due_date" class="due-badge">{{ new Date(s.due_date).toLocaleDateString("zh-CN", { month: "2-digit", day: "2-digit" }) }}</span>
            </div>
          </div>
          <button class="due-btn" :class="{ overdue: s.due_date && s.due_date < Date.now() && !s.completed }" @click.stop="initDueEdit(s); subDueId = s.id" title="截止日期">{{ s.due_date ? '📅' : '➕' }}</button>
          <button class="icon-btn" @click="startSubEdit(s)" title="编辑">✎</button>
          <button class="del" @click="emit('remove', s.id)" title="删除">&times;</button>
        </template>

        <!-- Sub editing -->
        <div v-else class="sub-edit" @blur="saveSubEdit(s)">
          <input v-model="subEditTitle" class="edit-title" placeholder="子任务标题" @keydown.escape="cancelSubEdit" />
          <textarea v-model="subEditContent" class="edit-body" placeholder="内容..." rows="2" @keydown.escape="cancelSubEdit" />
          <div class="edit-actions">
            <button class="act save" @mousedown.prevent="saveSubEdit(s)">保存</button>
            <button class="act cancel" @mousedown.prevent="cancelSubEdit">取消</button>
          </div>
        </div>
      </div>

      <!-- Add subtask -->
      <button class="add-sub" @click="emit('addSubtask', note.id)">+ 子任务</button>
    </div>
  </div>
</template>

<style scoped>
.card {
  background: rgba(255,255,255,0.05);
  border-left: 3px solid #666;
  border-radius: 6px;
  padding: 8px 10px;
  transition: all 0.15s;
}
.card:hover { background: rgba(255,255,255,0.08); }
.card.drag-over { border-left-color: #4ade80 !important; background: rgba(74,222,128,0.08); }
.card.editing { background: rgba(30,30,30,0.95); }
.row { display: flex; align-items: flex-start; gap: 8px; cursor: pointer; }
.check {
  width: 18px; height: 18px; border-radius: 4px;
  border: 1.5px solid rgba(255,255,255,0.3);
  background: transparent; color: #4ade80;
  cursor: pointer; font-size: 11px; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  margin-top: 1px;
}
.check.done { background: rgba(74,222,128,0.2); border-color: #4ade80; }
.info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
.title-text { font-size: 13px; color: rgba(255,255,255,0.9); word-break: break-word; }
.title-text.done { text-decoration: line-through; color: rgba(255,255,255,0.35); }
.meta-row { display: flex; align-items: center; gap: 6px; }
.preview {
  font-size: 11px; color: rgba(255,255,255,0.4);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex: 1; min-width: 0;
}
.due-badge, .overdue-badge { font-size: 10px; padding: 1px 6px; border-radius: 4px; flex-shrink: 0; }
.due-badge { background: rgba(255,255,255,0.06); color: rgba(255,255,255,0.4); }
.overdue-badge { background: rgba(255,80,80,0.25); color: #ff6b6b; }
.due-btn {
  background: none; border: none; cursor: pointer; font-size: 12px; flex-shrink: 0; padding: 0 2px; line-height: 1;
  opacity: 0.3; transition: all 0.15s;
}
.due-btn:hover { opacity: 0.8; }
.due-btn.overdue { opacity: 1; }
.due-popup {
  position: absolute; top: 100%; right: 0; z-index: 100; margin-top: 2px;
  display: flex; flex-direction: column; gap: 6px; padding: 10px; border-radius: 8px;
  background: rgba(24,24,24,0.96); backdrop-filter: blur(16px);
  border: 1px solid rgba(255,255,255,0.1); min-width: 200px;
}
.due-field { display: flex; align-items: center; gap: 6px; font-size: 11px; color: rgba(255,255,255,0.5); }
.due-input {
  flex: 1; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.12);
  border-radius: 4px; color: #fff; font-size: 11px; padding: 4px 6px; outline: none;
}
.due-actions { display: flex; gap: 6px; justify-content: flex-end; }
.card { position: relative; }
.icon-btn {
  background: none; border: none; color: rgba(255,255,255,0.2);
  cursor: pointer; font-size: 13px; flex-shrink: 0; padding: 0 2px;
}
.icon-btn:hover { color: rgba(255,255,255,0.7); }
.pin-btn {
  background: none; border: none; cursor: pointer; flex-shrink: 0;
  font-size: 12px; padding: 0 2px; line-height: 1; opacity: 0.3;
  transition: all 0.15s;
}
.pin-btn:hover { opacity: 0.8; }
.pin-btn.pinned { opacity: 1; filter: drop-shadow(0 0 2px rgba(255,200,0,0.3)); }
.color-wrap { position: relative; flex-shrink: 0; display: flex; align-items: center; }
.color-dot {
  width: 14px; height: 14px; border-radius: 50%; cursor: pointer;
  border: 1.5px solid rgba(255,255,255,0.15); transition: border-color 0.15s;
}
.color-dot:hover { border-color: rgba(255,255,255,0.4); }
.color-picker {
  position: absolute; top: 20px; right: 0; z-index: 100;
  display: flex; gap: 3px; padding: 4px; border-radius: 6px;
  background: rgba(24,24,24,0.96); backdrop-filter: blur(16px);
  border: 1px solid rgba(255,255,255,0.1);
}
.color-opt {
  width: 16px; height: 16px; border-radius: 50%; cursor: pointer;
  border: 2px solid transparent; transition: border-color 0.1s;
}
.color-opt:hover { border-color: rgba(255,255,255,0.5); }
.color-opt.active { border-color: #fff; }
.del {
  background: none; border: none; color: rgba(255,255,255,0.2);
  cursor: pointer; font-size: 16px; flex-shrink: 0; line-height: 1;
}
.del:hover { color: rgba(255,100,100,0.8); }
.edit-form { display: flex; flex-direction: column; gap: 6px; }
.edit-title, .sub-edit .edit-title {
  width: 100%; background: rgba(255,255,255,0.08);
  border: 1px solid rgba(255,255,255,0.2); border-radius: 4px;
  color: #fff; font-size: 13px; padding: 6px 8px; outline: none;
}
.edit-body, .sub-edit .edit-body {
  width: 100%; background: rgba(255,255,255,0.06);
  border: 1px solid rgba(255,255,255,0.15); border-radius: 4px;
  color: #fff; font-size: 12px; padding: 8px; outline: none;
  resize: vertical; font-family: inherit;
}
.edit-actions { display: flex; gap: 6px; justify-content: flex-end; }
.act { padding: 3px 12px; border: none; border-radius: 4px; font-size: 11px; cursor: pointer; }
.act.save { background: rgba(74,222,128,0.3); color: #4ade80; }
.act.cancel { background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.5); }
.act.save:hover { background: rgba(74,222,128,0.5); }
.act.cancel:hover { background: rgba(255,255,255,0.15); }

/* Subtask styles */
.subs { margin-top: 6px; margin-left: 12px; display: flex; flex-direction: column; gap: 3px; }
.sub-row {
  display: flex; align-items: flex-start; gap: 6px;
  padding: 4px 6px; border-radius: 4px;
  font-size: 12px;
}
.sub-row:hover { background: rgba(255,255,255,0.06); }
.sub-row.editing { background: rgba(40,40,40,0.9); }
.sub-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; cursor: pointer; }
.sub-title { color: rgba(255,255,255,0.75); word-break: break-word; font-size: 12px; }
.sub-title.done { text-decoration: line-through; color: rgba(255,255,255,0.3); }
.sub-preview { font-size: 10px; color: rgba(255,255,255,0.3); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.sub-edit { display: flex; flex-direction: column; gap: 4px; width: 100%; }
.sub-row .check { width: 16px; height: 16px; font-size: 10px; }
.sub-row .del { font-size: 14px; }
.sub-row .icon-btn { font-size: 11px; }

.add-sub {
  background: none; border: none;
  color: rgba(255,255,255,0.25); font-size: 11px;
  cursor: pointer; padding: 2px 6px; border-radius: 4px;
  text-align: left; transition: all 0.15s;
}
.add-sub:hover { color: rgba(255,255,255,0.6); background: rgba(255,255,255,0.05); }
</style>
