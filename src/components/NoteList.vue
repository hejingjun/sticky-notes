<script setup lang="ts">
import { ref, computed } from "vue";
import type { Note } from "../types/note";
import { COLORS } from "../types/note";
import NoteCard from "./NoteCard.vue";

const props = defineProps<{ notes: Note[]; tab: "todo" | "done" }>();
const emit = defineEmits<{
  toggle: [note: Note];
  update: [note: Note];
  remove: [id: string];
  add: [];
  addSubtask: [parentId: string];
  editing: [v: boolean];
  reorder: [draggedId: string, beforeId: string | null];
  'resolve-conflict': [noteId: string];
}>();

const search = ref("");
const filterColor = ref("");

const filteredNotes = computed(() => {
  let list = props.notes.filter((n) => !n.parent_id);
  // Tab filter
  if (props.tab === "todo") {
    list = list.filter((n) => !n.completed);
  } else {
    list = list.filter((n) => n.completed);
  }
  if (filterColor.value) {
    list = list.filter((n) => n.color === filterColor.value);
  }
  if (search.value.trim()) {
    const q = search.value.trim().toLowerCase();
    list = list.filter((n) => {
      if (n.title.toLowerCase().includes(q)) return true;
      return props.notes.some((s) => s.parent_id === n.id && s.title.toLowerCase().includes(q));
    });
  }
  return list;
});

function dateKey(ts: number): string {
  const d = new Date(ts);
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const yesterday = today - 86400000;
  if (ts >= today) return "今天";
  if (ts >= yesterday) return "昨天";
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  const label = `${m}/${day}`;
  const year = d.getFullYear();
  if (year === now.getFullYear()) return label;
  return `${year}/${label}`;
}

const groupedDone = computed(() => {
  const map = new Map<string, Note[]>();
  const sorted = [...filteredNotes.value].sort((a, b) => {
    const ta = a.completed_at || a.updated_at;
    const tb = b.completed_at || b.updated_at;
    return tb - ta; // newest first
  });
  for (const n of sorted) {
    const ts = n.completed_at || n.updated_at;
    const key = dateKey(ts);
    if (!map.has(key)) map.set(key, []);
    map.get(key)!.push(n);
  }
  return map;
});

function onEditing(v: boolean) {
  emit("editing", v);
}
</script>

<template>
  <div class="list">
    <!-- Todo view: search + filter + add -->
    <template v-if="props.tab === 'todo'">
      <div class="search-row">
        <input v-model="search" class="search-input" placeholder="搜索便签..." @keydown.escape="search = ''" />
        <button v-if="filterColor" class="clear-filter" @click="filterColor = ''" title="清除筛选">✕</button>
      </div>
      <div class="color-chips">
        <button class="chip" :class="{ active: !filterColor }" @click="filterColor = ''">全部</button>
        <button v-for="c in COLORS" :key="c" class="chip" :class="{ active: filterColor === c }" :style="{ background: c }" @click="filterColor = c === filterColor ? '' : c"></button>
      </div>
      <button class="add-btn" @click="emit('add')">+ New Note</button>
      <NoteCard
        v-for="note in filteredNotes"
        :key="note.id"
        :note="note"
        :all-notes="props.notes"
        @toggle="emit('toggle', $event)"
        @update="emit('update', $event)"
        @remove="emit('remove', $event)"
        @add-subtask="emit('addSubtask', $event)"
        @editing="onEditing"
        @reorder="(a, b) => emit('reorder', a, b)"
        @resolve-conflict="(id) => emit('resolve-conflict', id)"
      />
      <div v-if="filteredNotes.length === 0" class="empty">暂无待办</div>
    </template>

    <!-- Done view: grouped by date -->
    <template v-else>
      <template v-for="[dateLabel, notes] in groupedDone" :key="dateLabel">
        <div class="date-header">{{ dateLabel }}</div>
        <NoteCard
          v-for="note in notes"
          :key="note.id"
          :note="note"
          :all-notes="props.notes"
          @toggle="emit('toggle', $event)"
          @update="emit('update', $event)"
          @remove="emit('remove', $event)"
          @add-subtask="emit('addSubtask', $event)"
          @editing="onEditing"
          @reorder="(a, b) => emit('reorder', a, b)"
        @resolve-conflict="(id) => emit('resolve-conflict', id)"
        />
      </template>
      <div v-if="filteredNotes.length === 0" class="empty">暂无已完成任务</div>
    </template>
  </div>
</template>

<style scoped>
.list {
  padding: var(--sp-sm);
  display: flex;
  flex-direction: column;
  gap: var(--sp-sm);
  height: 100%;
  overflow-y: auto;
}

.search-row { display: flex; gap: var(--sp-xs); }

.search-input {
  flex: 1;
  background: var(--surface-1);
  border: 1px solid var(--border-light);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: var(--font-base);
  padding: 7px 10px;
  outline: none;
  transition: border-color var(--duration) var(--ease);
}
.search-input:focus { border-color: var(--border-medium); }
.search-input::placeholder { color: var(--text-tertiary); }

.clear-filter {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: var(--font-lg);
  padding: 0 var(--sp-xs);
}

.color-chips { display: flex; gap: var(--sp-xs); flex-wrap: wrap; }

.chip {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  transition: border-color var(--duration) var(--ease);
  font-size: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.chip.active { border-color: var(--accent); }

.chip:first-child {
  width: auto;
  padding: 0 10px;
  background: var(--surface-1);
  border-radius: 10px;
  color: var(--text-secondary);
  font-size: var(--font-xs);
}
.chip:first-child.active {
  background: var(--accent-dim);
  color: var(--accent);
}

.add-btn {
  background: var(--surface-0);
  border: 1px dashed var(--border-light);
  border-radius: 8px;
  color: var(--text-secondary);
  padding: 12px;
  cursor: pointer;
  font-size: var(--font-base);
  transition: all var(--duration) var(--ease);
}
.add-btn:hover {
  background: var(--surface-2);
  color: var(--text-primary);
  border-color: var(--border-medium);
}

.empty {
  text-align: center;
  color: var(--text-disabled);
  font-size: var(--font-sm);
  padding: var(--sp-xl) 0;
}

.date-header {
  font-size: var(--font-xs);
  font-weight: 600;
  color: var(--text-tertiary);
  padding: var(--sp-sm) 0 2px;
  border-bottom: 1px solid var(--border-subtle);
  margin-bottom: 2px;
  letter-spacing: 0.5px;
}
</style>
