<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

const props = defineProps<{ onAdd: () => void; onTop: () => void; onSettings: () => void; onExport: (format: string) => void; ontop: boolean; onClose: () => void }>();
const menu = ref<{ show: boolean; x: number; y: number }>({ show: false, x: 0, y: 0 });

function onCtx(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) return;
  e.preventDefault();
  // Clamp to viewport so menu doesn't overflow
  const mx = Math.min(e.clientX, window.innerWidth - 150);
  const my = Math.min(e.clientY, window.innerHeight - 200);
  menu.value = { show: true, x: mx, y: my };
}
function hide() { menu.value.show = false; }

onMounted(() => {
  document.addEventListener("contextmenu", onCtx);
  document.addEventListener("click", hide);
});
onUnmounted(() => {
  document.removeEventListener("contextmenu", onCtx);
  document.removeEventListener("click", hide);
});
</script>

<template>
  <div v-if="menu.show" class="ctx" :style="{ left: menu.x + 'px', top: menu.y + 'px' }">
    <div class="item" @click="props.onAdd(); hide()">新建便签</div>
    <div class="item" :class="{ active: props.ontop }" @click="props.onTop(); hide()">{{ props.ontop ? '✓ 置顶' : '置顶' }}</div>
    <div class="sep"></div>
    <div class="item" @click="props.onSettings(); hide()">设置</div>
    <div class="sep"></div>
    <div class="item" @click="props.onExport('csv'); hide()">导出 CSV</div>
    <div class="sep"></div>
    <div class="item close" @click="props.onClose(); hide()">隐藏到托盘</div>
  </div>
</template>

<style scoped>
.ctx {
  position: fixed;
  z-index: 9999;
  background: rgba(20, 20, 20, 0.95);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border-light);
  border-radius: 8px;
  padding: var(--sp-xs);
  min-width: 140px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.item {
  padding: 7px 12px;
  font-size: var(--font-base);
  color: var(--text-primary);
  border-radius: 5px;
  cursor: pointer;
  transition: all var(--duration) var(--ease);
}
.item:hover { background: var(--surface-3); }
.item.close:hover { background: var(--danger-dim); color: var(--danger); }
.item.active { color: var(--accent); }

.sep {
  height: 1px;
  background: var(--border-subtle);
  margin: var(--sp-xs) var(--sp-sm);
}
</style>
