<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

const props = defineProps<{ onAdd: () => void; onPen: () => void; onTop: () => void; onSettings: () => void; onExport: (format: string) => void; ontop: boolean; penetrating: boolean; onClose: () => void }>();
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
    <div class="item" :class="{ active: props.penetrating }" @click="props.onPen(); hide()">{{ props.penetrating ? '✓ 穿透' : '穿透' }}</div>
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
  position: fixed; z-index: 9999;
  background: rgba(20,20,20,0.95); backdrop-filter: blur(20px);
  border: 1px solid rgba(255,255,255,0.1); border-radius: 8px;
  padding: 4px; min-width: 140px;
}
.item {
  padding: 8px 12px; font-size: 12px; color: rgba(255,255,255,0.8);
  border-radius: 4px; cursor: pointer;
}
.item:hover { background: rgba(255,255,255,0.1); }
.item.close:hover { background: rgba(255,80,80,0.3); color: #ff6b6b; }
.sep { height: 1px; background: rgba(255,255,255,0.08); margin: 4px 8px; }
.item.active { color: #4ade80; }
</style>
