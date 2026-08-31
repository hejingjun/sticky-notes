import { ref, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const SYNC_DEBOUNCE_MS = 5000;  // Wait 5s after last change
const POLL_INTERVAL_MS = 300000; // Poll every 5 minutes

const isSyncing = ref(false);
const lastSyncTime = ref<number>(0);
const syncError = ref<string>("");
const webdavConfigured = ref(false);

let syncTimer: ReturnType<typeof setTimeout> | null = null;
let pollTimer: ReturnType<typeof setInterval> | null = null;
let focusHandler: (() => void) | null = null;
let syncing = false;

async function checkConfig() {
  try {
    const cfg: { webdav_url: string } = await invoke("get_settings");
    webdavConfigured.value = !!cfg.webdav_url;
  } catch {
    webdavConfigured.value = false;
  }
}

/**
 * Single sync function: fetch remote → merge with local → push back.
 * Uses the existing sync_notes command which handles LWW merge correctly.
 * Local edits always win because their updated_at is newer.
 */
async function doSync() {
  if (!webdavConfigured.value || syncing) return;
  syncing = true;
  isSyncing.value = true;
  syncError.value = "";
  try {
    await invoke("sync_notes");
    lastSyncTime.value = Date.now();
  } catch (e) {
    syncError.value = String(e);
    console.warn("[sync] failed:", e);
  } finally {
    syncing = false;
    isSyncing.value = false;
  }
}

/**
 * Called after each local mutation. Debounces the sync.
 */
function notifyChanged() {
  if (!webdavConfigured.value) return;
  if (syncTimer) clearTimeout(syncTimer);
  syncTimer = setTimeout(doSync, SYNC_DEBOUNCE_MS);
}

/**
 * Sync immediately (e.g. Ctrl+S). Cancels any pending debounced sync.
 */
function syncNow() {
  if (syncTimer) { clearTimeout(syncTimer); syncTimer = null; }
  doSync();
}

function onFocus() {
  doSync();
}

async function startSync() {
  await checkConfig();
  if (!webdavConfigured.value) return;
  focusHandler = onFocus;
  window.addEventListener("focus", focusHandler);
  pollTimer = setInterval(doSync, POLL_INTERVAL_MS);
}

function stopSync() {
  if (syncTimer) { clearTimeout(syncTimer); syncTimer = null; }
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }
  if (focusHandler) { window.removeEventListener("focus", focusHandler); focusHandler = null; }
}

export function useSync() {
  onUnmounted(stopSync);
  return {
    startSync,
    stopSync,
    notifyChanged,
    syncNow,
    isSyncing,
    lastSyncTime,
    syncError,
    webdavConfigured,
    checkConfig,
  };
}
