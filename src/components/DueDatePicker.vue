<script setup lang="ts">
import { ref, computed } from "vue";

const props = defineProps<{
  dueDate: number | null;
  remindAt: number | null;
  show: boolean;
  isOverdue: boolean;
}>();

const emit = defineEmits<{
  save: [due: number | null, remind: number | null];
  clear: [];
  toggle: [];
}>();

const editDueDate = ref("");
const editRemindAt = ref("");

const dueDateStr = computed(() => {
  if (!props.dueDate) return "";
  return new Date(props.dueDate).toLocaleDateString("zh-CN", { month: "2-digit", day: "2-digit" });
});

function initEdit() {
  editDueDate.value = props.dueDate ? new Date(props.dueDate).toISOString().slice(0, 16) : "";
  editRemindAt.value = props.remindAt ? new Date(props.remindAt).toISOString().slice(0, 16) : "";
}

function save() {
  const due = editDueDate.value ? new Date(editDueDate.value).getTime() : null;
  const remind = editRemindAt.value ? new Date(editRemindAt.value).getTime() : null;
  emit("save", due, remind);
}
</script>

<template>
  <div class="due-wrap">
    <span v-if="props.isOverdue" class="overdue-badge">逾期</span>
    <span v-else-if="dueDateStr" class="due-badge">{{ dueDateStr }}</span>
    <button class="due-btn" :class="{ overdue: props.isOverdue }" @click.stop="initEdit(); emit('toggle')" title="截止日期">📅</button>
    <div v-if="props.show" class="due-popup" @click.stop @mouseleave="emit('toggle')">
      <label class="due-field">截止 <input v-model="editDueDate" type="datetime-local" class="due-input" /></label>
      <label class="due-field">提醒 <input v-model="editRemindAt" type="datetime-local" class="due-input" /></label>
      <div class="due-actions">
        <button class="act save" @click="save">保存</button>
        <button class="act cancel" @click="emit('clear')">清除</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.due-wrap { display: flex; align-items: center; gap: 4px; position: relative; flex-shrink: 0; }
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
.act { padding: 3px 12px; border: none; border-radius: 4px; font-size: 11px; cursor: pointer; }
.act.save { background: rgba(74,222,128,0.3); color: #4ade80; }
.act.cancel { background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.5); }
.act.save:hover { background: rgba(74,222,128,0.5); }
.act.cancel:hover { background: rgba(255,255,255,0.15); }
</style>
