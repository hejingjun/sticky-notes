// ============================================================
// Electron Spike v2 — 预加载脚本
// ============================================================

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("spike", {
  startDrag: () => ipcRenderer.send("start-drag"),
  setIgnoreMouse: (ignore) => ipcRenderer.send("set-ignore-mouse", ignore),
  setAlwaysOnTop: (flag) => ipcRenderer.send("set-always-on-top", flag),
  togglePenetrate: () => ipcRenderer.send("toggle-penetrate"),
  minimize: () => ipcRenderer.send("minimize-window"),
  close: () => ipcRenderer.send("close-window"),
  // 监听主进程推送的穿透状态变化
  onPenetrateChanged: (callback) => {
    ipcRenderer.on("penetrate-changed", (_event, value) => callback(value));
  },
});
