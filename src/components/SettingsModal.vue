<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ close: []; 'shortcut-updated': [shortcut: string] }>();

const penetrate = ref("加载中...");
const autostart = ref(false);
const autoPurge = ref(true);
const opacity = ref(0.6);
const theme = ref("glass");
const saving = ref(false);
const error = ref("");
const listening = ref(false);

const webdavUrl = ref("");
const webdavUser = ref("");
const webdavPassword = ref("");
const showPassword = ref(false);
const syncing = ref(false);
const syncStatus = ref("");

function formatShortcut(e: KeyboardEvent): string | null {
  if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) return null;
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");
  let mainKey = e.key;
  if (mainKey === " ") mainKey = "Space";
  if (mainKey.length === 1) mainKey = mainKey.toUpperCase();
  parts.push(mainKey);
  return parts.join("+");
}

function startCapture() { listening.value = true; error.value = ""; }

function onCaptureKey(e: KeyboardEvent) {
  e.preventDefault();
  if (e.key === "Escape") { listening.value = false; return; }
  const combo = formatShortcut(e);
  if (!combo) return;
  penetrate.value = combo;
  listening.value = false;
}

onMounted(async () => {
  try {
    const cfg: { penetrate: string; auto_purge: boolean; opacity: number; theme: string; webdav_url: string; webdav_user: string; webdav_password: string } = await invoke("get_settings");
    penetrate.value = cfg.penetrate;
    autoPurge.value = cfg.auto_purge;
    opacity.value = cfg.opacity ?? 0.6;
    theme.value = cfg.theme || "green";
    webdavUrl.value = cfg.webdav_url;
    webdavUser.value = cfg.webdav_user;
    webdavPassword.value = cfg.webdav_password;
    document.documentElement.style.setProperty("--glass-opacity", String(opacity.value));
    autostart.value = await invoke<boolean>("is_autostart");
  } catch (e) {
    error.value = String(e);
  }
});

watch(opacity, (v) => {
  document.documentElement.style.setProperty("--glass-opacity", String(v));
});

async function toggleAuto() {
  try { autostart.value = await invoke<boolean>("toggle_autostart"); }
  catch (e) { error.value = String(e); }
}

async function togglePurge() {
  try { autoPurge.value = await invoke<boolean>("toggle_auto_purge"); }
  catch (e) { error.value = String(e); }
}

function onOpacityInput(e: Event) {
  const v = parseFloat((e.target as HTMLInputElement).value);
  opacity.value = v;
  document.documentElement.style.setProperty("--glass-opacity", String(v));
}

async function saveOpacity() {
  try { await invoke("set_opacity", { value: opacity.value }); }
  catch (e) { error.value = String(e); }
}

function setTheme(t: string) {
  theme.value = t;
  document.body.className = document.body.className.replace(/theme-\S+/g, "").trim() + " theme-" + t;
  invoke("set_theme", { theme: t }).catch((e) => error.value = String(e));
}

async function saveWebDAV() {
  try {
    await invoke("save_webdav", { url: webdavUrl.value, user: webdavUser.value, password: webdavPassword.value });
    syncStatus.value = "已保存";
  } catch (e) {
    error.value = String(e);
  }
}

async function doPush() {
  syncing.value = true;
  syncStatus.value = "上传中...";
  error.value = "";
  try {
    const msg = await invoke<string>("sync_push");
    syncStatus.value = msg;
  } catch (e) {
    const errStr = String(e);
    if (errStr.includes("404") || errStr.includes("远程文件不存在")) {
      syncStatus.value = "首次上传（远程无数据）";
      return;
    }
    error.value = errStr;
    syncStatus.value = "上传失败";
  } finally {
    syncing.value = false;
  }
}

async function doPull() {
  syncing.value = true;
  syncStatus.value = "下载中...";
  error.value = "";
  try {
    const msg = await invoke<string>("sync_pull");
    syncStatus.value = msg;
  } catch (e) {
    const errStr = String(e);
    if (errStr.includes("404") || errStr.includes("远程文件不存在")) {
      syncStatus.value = "远程无数据";
      return;
    }
    error.value = errStr;
    syncStatus.value = "下载失败";
  } finally {
    syncing.value = false;
  }
}

async function save() {
  saving.value = true;
  error.value = "";
  try {
    await invoke("set_shortcut", { accelerator: penetrate.value });
    emit("shortcut-updated", penetrate.value);
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="overlay" @click.self="emit('close')">
      <div class="panel">
        <div class="panel-title">设置</div>

        <label class="field">
          <span class="label">穿透快捷键</span>
          <div class="capture" :class="{ listening }" tabindex="0" @click="startCapture" @keydown="onCaptureKey" @blur="listening = false">
            <template v-if="listening">请按下快捷键...</template>
            <template v-else>{{ penetrate }}</template>
          </div>
          <span class="hint">点击上方区域，然后按下组合键</span>
        </label>

        <label class="field">
          <span class="label">背景透明度 {{ Math.round(opacity * 100) }}%</span>
          <input type="range" min="0.1" max="0.9" step="0.05" :value="opacity" class="slider" @input="onOpacityInput" @change="saveOpacity" />
          <span class="hint">编辑便签时自动降低透明度</span>
        </label>

        <label class="field">
          <span class="label">主题风格</span>
          <div class="theme-options">
            <button class="theme-btn" :class="{ active: theme === 'green' }" @click="setTheme('green')">🌲 松绿</button>
            <button class="theme-btn" :class="{ active: theme === 'blue' }" @click="setTheme('blue')">🌊 雾蓝</button>
            <button class="theme-btn" :class="{ active: theme === 'gray' }" @click="setTheme('gray')">🪨 暖灰</button>
          </div>
        </label>

        <!-- WebDAV 同步 -->
        <div class="section-title">WebDAV 同步</div>
        <label class="field">
          <span class="label">WebDAV 地址</span>
          <input v-model="webdavUrl" class="input" placeholder="https://dav.example.com/dav/" />
          <span class="hint">WebDAV 根地址，必须以 / 结尾</span>
        </label>
        <label class="field">
          <span class="label">账号</span>
          <input v-model="webdavUser" class="input" placeholder="your@email.com" />
        </label>
        <label class="field">
          <span class="label">应用密码</span>
          <div class="password-row">
            <input v-model="webdavPassword" class="input" :type="showPassword ? 'text' : 'password'" placeholder="密码" />
            <button class="eye-btn" @click="showPassword = !showPassword" :title="showPassword ? '隐藏密码' : '显示密码'">
              {{ showPassword ? '🙈' : '👁' }}
            </button>
          </div>
        </label>
        <div class="webdav-actions">
          <button class="btn" @click="saveWebDAV">保存配置</button>
          <button class="btn primary" :disabled="syncing" @click="doPush">{{ syncing ? '...' : '上传' }}</button>
          <button class="btn primary" :disabled="syncing" @click="doPull">{{ syncing ? '...' : '下载' }}</button>
        </div>
        <div v-if="syncStatus" class="sync-status">{{ syncStatus }}</div>

        <label class="field row-field">
          <span class="label">开机自启</span>
          <button class="toggle" :class="{ on: autostart }" @click="toggleAuto">{{ autostart ? '开启' : '关闭' }}</button>
        </label>

        <label class="field row-field">
          <span class="label">30天自动清理</span>
          <button class="toggle" :class="{ on: autoPurge }" @click="togglePurge">{{ autoPurge ? '开启' : '关闭' }}</button>
        </label>

        <div v-if="error" class="error">{{ error }}</div>

        <div class="actions">
          <button class="btn primary" :disabled="saving" @click="save">{{ saving ? '保存中...' : '保存' }}</button>
          <button class="btn" @click="emit('close')">取消</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: rgba(0, 0, 0, 0.4);
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
  width: 360px;
  display: flex;
  flex-direction: column;
  gap: var(--sp-md);
  max-height: 90vh;
  overflow-y: auto;
}

.panel-title {
  font-size: var(--font-lg);
  font-weight: 600;
  color: var(--text-primary);
}

.section-title {
  font-size: var(--font-base);
  font-weight: 600;
  color: var(--text-secondary);
  margin-top: var(--sp-xs);
}

.field { display: flex; flex-direction: column; gap: 3px; }
.row-field { flex-direction: row; align-items: center; justify-content: space-between; }

.label {
  font-size: var(--font-sm);
  color: var(--text-secondary);
}

.capture, .input {
  background: var(--surface-1);
  border: 1px solid var(--border-medium);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: var(--font-base);
  padding: 7px 10px;
  outline: none;
  transition: border-color var(--duration) var(--ease);
}
.capture { cursor: pointer; user-select: none; min-height: 18px; }
.capture:hover, .capture:focus, .input:focus { border-color: var(--accent); }
.capture.listening {
  border-color: var(--accent);
  background: var(--accent-dim);
  animation: pulse 1.5s ease-in-out infinite;
}
@keyframes pulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(74, 222, 128, 0.2); }
  50% { box-shadow: 0 0 0 4px rgba(74, 222, 128, 0.05); }
}

.slider {
  -webkit-appearance: none;
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: var(--surface-3);
  outline: none;
  cursor: pointer;
}
.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--accent);
  border: none;
  cursor: pointer;
}

.webdav-actions { display: flex; gap: 6px; }

.theme-options { display: flex; gap: 6px; }

.theme-btn {
  flex: 1;
  padding: 7px 0;
  border: 1px solid var(--border-medium);
  border-radius: 6px;
  font-size: var(--font-sm);
  cursor: pointer;
  background: var(--surface-0);
  color: var(--text-secondary);
  transition: all var(--duration) var(--ease);
}
.theme-btn:hover { background: var(--surface-2); }
.theme-btn.active {
  border-color: var(--accent);
  background: var(--accent-dim);
  color: var(--accent);
}

.sync-status {
  font-size: var(--font-sm);
  color: var(--accent);
  text-align: center;
}

.toggle {
  padding: 4px 14px;
  border: 1px solid var(--border-medium);
  border-radius: 6px;
  font-size: var(--font-sm);
  cursor: pointer;
  background: var(--surface-1);
  color: var(--text-secondary);
  transition: all var(--duration) var(--ease);
}
.toggle.on {
  background: var(--accent-dim);
  border-color: rgba(74, 222, 128, 0.3);
  color: var(--accent);
}

.hint {
  font-size: var(--font-xs);
  color: var(--text-tertiary);
}

.password-row { display: flex; gap: var(--sp-xs); }
.password-row .input { flex: 1; }

.eye-btn {
  width: 36px;
  border: 1px solid var(--border-medium);
  border-radius: 6px;
  background: var(--surface-1);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: var(--font-lg);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--duration) var(--ease);
}
.eye-btn:hover { background: var(--surface-3); color: var(--text-primary); }

.error {
  font-size: var(--font-sm);
  color: var(--danger);
  padding: 7px 10px;
  background: var(--danger-dim);
  border-radius: 5px;
}

.actions { display: flex; gap: var(--sp-sm); justify-content: flex-end; }

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
.btn.primary { background: var(--accent-dim); color: var(--accent); }
.btn.primary:hover { background: var(--accent-glow); }
.btn:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
