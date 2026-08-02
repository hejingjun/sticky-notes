<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Note } from "../types/note";

const props = defineProps<{ noteId: string | null }>();
const emit = defineEmits<{ close: []; resolved: [] }>();

const remoteNote = ref<Note | null>(null);
const localNote = ref<Note | null>(null);
const loading = ref(false);

watch(() => props.noteId, async (id) => {
  if (!id) return;
  loading.value = true;
  try {
    const conflict = await invoke<Note | null>("get_conflict", { noteId: id });
    remoteNote.value = conflict;
    // Get local note from the notes list (passed via parent or fetch)
    localNote.value = null; // Will be populated by parent
  } catch (e) {
    console.error("获取冲突版本失败:", e);
  } finally {
    loading.value = false;
  }
}, { immediate: true });

async function resolve(useRemote: boolean) {
  if (!props.noteId) return;
  loading.value = true;
  try {
    await invoke("resolve_conflict", { noteId: props.noteId, useRemote });
    emit("resolved");
    emit("close");
  } catch (e) {
    console.error("解决冲突失败:", e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="noteId" class="overlay" @click.self="emit('close')">
      <div class="panel">
        <div class="panel-title">⚠️ 同步冲突</div>
        <p class="desc">此便签在两台设备上同时被修改，请选择保留哪个版本：</p>

        <div v-if="loading" class="loading">加载中...</div>

        <template v-else-if="remoteNote">
          <div class="versions">
            <div class="version local">
              <div class="version-label">📱 本地版本</div>
              <div class="version-title">{{ localNote?.title || '(当前显示)' }}</div>
              <div class="version-meta" v-if="localNote">
                修改时间: {{ new Date(localNote.updated_at).toLocaleString("zh-CN") }}
              </div>
            </div>
            <div class="version remote">
              <div class="version-label">☁️ 远程版本</div>
              <div class="version-title">{{ remoteNote.title || '(无标题)' }}</div>
              <div class="version-meta">
                修改时间: {{ new Date(remoteNote.updated_at).toLocaleString("zh-CN") }}
              </div>
            </div>
          </div>

          <div class="actions">
            <button class="btn" @click="resolve(false)" :disabled="loading">保留本地</button>
            <button class="btn primary" @click="resolve(true)" :disabled="loading">使用远程</button>
          </div>
        </template>

        <div v-else class="no-conflict">未找到冲突数据</div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 10001;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}
.panel {
  background: rgba(24, 24, 24, 0.96);
  backdrop-filter: blur(24px);
  border: 1px solid var(--border-light);
  border-radius: var(--glass-radius);
  padding: var(--sp-xl);
  width: 380px;
  display: flex;
  flex-direction: column;
  gap: var(--sp-md);
}
.panel-title {
  font-size: var(--font-lg);
  font-weight: 600;
  color: var(--warning);
}
.desc {
  font-size: var(--font-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}
.versions {
  display: flex;
  flex-direction: column;
  gap: var(--sp-sm);
}
.version {
  padding: var(--sp-md);
  border-radius: 8px;
  border: 1px solid var(--border-light);
}
.version.local { border-color: var(--accent); background: var(--accent-dim); }
.version.remote { border-color: var(--warning); background: var(--warning-dim); }
.version-label {
  font-size: var(--font-xs);
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: var(--sp-xs);
}
.version-title {
  font-size: var(--font-md);
  color: var(--text-primary);
  font-weight: 500;
}
.version-meta {
  font-size: var(--font-xs);
  color: var(--text-tertiary);
  margin-top: var(--sp-xs);
}
.loading, .no-conflict {
  text-align: center;
  color: var(--text-tertiary);
  font-size: var(--font-sm);
  padding: var(--sp-lg) 0;
}
.actions {
  display: flex;
  gap: var(--sp-sm);
  justify-content: flex-end;
}
.btn {
  padding: 7px 18px;
  border: none;
  border-radius: 6px;
  font-size: var(--font-base);
  cursor: pointer;
  background: var(--surface-2);
  color: var(--text-secondary);
  transition: all var(--duration) var(--ease);
}
.btn:hover { background: var(--surface-3); color: var(--text-primary); }
.btn.primary { background: var(--warning-dim); color: var(--warning); }
.btn.primary:hover { background: rgba(255, 200, 0, 0.3); }
.btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
