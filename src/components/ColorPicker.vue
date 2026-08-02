<script setup lang="ts">
import { COLORS } from "../types/note";

const props = defineProps<{ color: string; show: boolean }>();
const emit = defineEmits<{
  select: [color: string];
  close: [];
}>();
</script>

<template>
  <div class="color-wrap">
    <div class="color-dot" :style="{ background: props.color }" @click.stop="emit('close')" title="颜色"></div>
    <div v-if="props.show" class="color-picker" @mouseleave="emit('close')">
      <div
        v-for="c in COLORS"
        :key="c"
        class="color-opt"
        :class="{ active: props.color === c }"
        :style="{ background: c }"
        @click.stop="emit('select', c)"
      ></div>
    </div>
  </div>
</template>

<style scoped>
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
</style>
