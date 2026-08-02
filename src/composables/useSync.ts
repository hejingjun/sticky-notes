import { ref, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const SYNC_DEBOUNCE_MS = 3000;  // Wait 3s after last change before pushing
const POLL_INTERVAL_MS = 60000; // Pull every 60s as fallback

const isSyncing = ref(false);
const lastSyncTime = ref<number>(0);
const syncError = ref<string>("");
const webdavConfigured = ref(false);

let pushTimer: ReturnType<typeof setTimeout> | null = null;
let pollTimer: ReturnType<typeof setInterval> | null = null;
let focusHandler: (() => void) | null = null;
let syncing = false; // Lock to prevent concurrent syncs

async function checkConfig() {
  try {
    const cfg: { webdav_url: string } = await invoke("get_settings");
    webdavConfigured.value = !!cfg.webdav_url;
  } catch {
    webdavConfigured.value = false;
  }
}

async function doPush() {
  if (!webdavConfigured.value || syncing) return;
  syncing = true;
  isSyncing.value = true;
  syncError.value = "";
  try {
    await invoke("sync_push");
    lastSyncTime.value = Date.now();
  } catch (e) {
    syncError.value = String(e);
    console.warn("[sync] push failed:", e);
  } finally {
    syncing = false;
    isSyncing.value = false;
  }
}

async function doPull() {
  if (!webdavConfigured.value || syncing) return;
  syncing = true;
  isSyncing.value = true;
  syncError.value = "";
  try {
    await invoke("sync_pull");
    lastSyncTime.value = Date.now();
  } catch (e) {
    syncError.value = String(e);
    console.warn("[sync] pull failed:", e);
  } finally {
    syncing = false;
    isSyncing.value = false;
  }
}

/**
 * Called after each local mutation. Debounces the push:
 * waits SYNC_DEBOUNCE_MS after the LAST call before actually pushing.
 */
function notifyChanged() {
  if (!webdavConfigured.value) return;
  if (pushTimer) clearTimeout(pushTimer);
  pushTimer = setTimeout(doPush, SYNC_DEBOUNCE_MS);
}

function onFocus() {
  // Pull when window gains focus (user may have edited on another device)
  doPull();
}

/**
 * Start the sync lifecycle: focus listener + poll timer.
 * Call this once in App.vue's onMounted.
 */
async function startSync() {
  await checkConfig();
  if (!webdavConfigured.value) return;

  // Trigger 2: pull on window focus
  focusHandler = onFocus;
  window.addEventListener("focus", focusHandler);

  // Trigger 3: periodic poll
  pollTimer = setInterval(doPull, POLL_INTERVAL_MS);
}

/**
 * Stop the sync lifecycle. Call on unmount.
 */
function stopSync() {
  if (pushTimer) { clearTimeout(pushTimer); pushTimer = null; }
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }
  if (focusHandler) { window.removeEventListener("focus", focusHandler); focusHandler = null; }
}

export function useSync() {
  onUnmounted(stopSync);

  return {
    startSync,
    stopSync,
    notifyChanged,
    isSyncing,
    lastSyncTime,
    syncError,
    webdavConfigured,
    checkConfig,
  };
}
