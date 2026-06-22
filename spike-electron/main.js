// ============================================================
// Electron Spike v3 — 主进程
// 修复：Win+D 轮询恢复 + GWL_HWNDPARENT
// ============================================================

const { app, BrowserWindow, ipcMain, globalShortcut } = require("electron");
const path = require("path");

// ─── koffi ───
const koffi = require("koffi");
koffi.alias("HWND", "size_t");
koffi.alias("LONG_PTR", "int64_t");
koffi.alias("LRESULT", "int64_t");
koffi.alias("WPARAM", "uint64_t");
koffi.alias("LPARAM", "int64_t");

const user32 = koffi.load("user32.dll");
const FindWindowW = user32.func("HWND FindWindowW(const char16_t* lpClassName, const char16_t* lpWindowName)");
const SetWindowLongPtrW = user32.func("LONG_PTR SetWindowLongPtrW(HWND hWnd, int nIndex, LONG_PTR dwNewLong)");
const GetWindowLongPtrW = user32.func("LONG_PTR GetWindowLongPtrW(HWND hWnd, int nIndex)");
const SetWindowPos = user32.func("int32_t SetWindowPos(HWND hWnd, HWND hWndInsertAfter, int X, int Y, int cx, int cy, uint32_t uFlags)");
const SetLayeredWindowAttributes = user32.func("int32_t SetLayeredWindowAttributes(HWND hWnd, uint32_t crKey, uint8_t bAlpha, uint32_t dwFlags)");
const ReleaseCapture = user32.func("int32_t ReleaseCapture()");
const SendMessageW = user32.func("LRESULT SendMessageW(HWND hWnd, uint32_t Msg, WPARAM wParam, LPARAM lParam)");
const IsWindowVisible = user32.func("int32_t IsWindowVisible(HWND hWnd)");
const ShowWindow = user32.func("int32_t ShowWindow(HWND hWnd, int nCmdShow)");
const IsWindow = user32.func("int32_t IsWindow(HWND hWnd)");

// 常量
const GWL_EXSTYLE = -20;
const GWL_HWNDPARENT = -8;
const WS_EX_TOOLWINDOW = 0x00000080;
const WS_EX_LAYERED = 0x00080000;
const WS_EX_APPWINDOW = 0x00040000;
const WS_EX_NOACTIVATE = 0x08000000;
const SWP_FRAMECHANGED = 0x0020;
const SWP_NOMOVE = 0x0002;
const SWP_NOSIZE = 0x0001;
const LWA_ALPHA = 0x00000002;
const SW_SHOWNOACTIVATE = 4;

let mainWindow = null;
let visibilityTimer = null;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 360,
    height: 500,
    x: 100,
    y: 100,
    frame: false,
    transparent: true,
    alwaysOnTop: true,
    skipTaskbar: true,
    resizable: true,
    hasShadow: false,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  mainWindow.on("minimize", (event) => {
    event.preventDefault();
    mainWindow.restore();
  });

  mainWindow.on("hide", () => {
    setTimeout(() => {
      if (mainWindow && !mainWindow.isDestroyed() && !mainWindow.isVisible()) {
        mainWindow.restore();
        mainWindow.showInactive();
      }
    }, 50);
  });

  mainWindow.loadFile(path.join(__dirname, "renderer", "index.html"));
  mainWindow.webContents.on("did-finish-load", () => {
    applyWin32Styles();
    startVisibilityPolling();
  });
}

function applyWin32Styles() {
  const hwnd = Number(mainWindow.getNativeWindowHandle().readBigUInt64LE(0));
  console.log(`[Spike v3] HWND: 0x${hwnd.toString(16)}`);

  // 1. 扩展样式
  let exStyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
  exStyle = (exStyle | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_NOACTIVATE) & ~WS_EX_APPWINDOW;
  SetWindowLongPtrW(hwnd, GWL_EXSTYLE, exStyle);

  // 2. 尝试将桌面设为逻辑父窗口（不嵌入，只设所有权）
  //    这能阻止 Win+D 把此窗口当作普通窗口最小化
  const progman = FindWindowW("Progman", null);
  if (progman !== 0) {
    SetWindowLongPtrW(hwnd, GWL_HWNDPARENT, progman);
    console.log(`[Spike v3] GWL_HWNDPARENT → Progman (0x${progman.toString(16)})`);
  }

  // 3. 应用
  SetWindowPos(hwnd, -1, 0, 0, 0, 0, SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE);
  SetLayeredWindowAttributes(hwnd, 0, 240, LWA_ALPHA);
}

// ═══ Win+D 轮询兜底 ═══
function startVisibilityPolling() {
  if (visibilityTimer) clearInterval(visibilityTimer);
  const hwnd = Number(mainWindow.getNativeWindowHandle().readBigUInt64LE(0));

  visibilityTimer = setInterval(() => {
    if (!mainWindow || mainWindow.isDestroyed()) {
      clearInterval(visibilityTimer);
      return;
    }
    // 用 Win32 IsWindowVisible 检查（比 Electron 的 isVisible 更可靠）
    if (IsWindow(hwnd) && !IsWindowVisible(hwnd)) {
      console.log("[Spike v3] 检测到窗口不可见，尝试恢复...");
      ShowWindow(hwnd, SW_SHOWNOACTIVATE);
      mainWindow.restore();
      mainWindow.showInactive();
    }
  }, 500);
}

// ─── IPC ───
ipcMain.on("start-drag", () => {
  const hwnd = Number(mainWindow.getNativeWindowHandle().readBigUInt64LE(0));
  ReleaseCapture();
  SendMessageW(hwnd, 0x00A1, 2, 0);
});

let isPenetrating = false;
function togglePenetrate() {
  isPenetrating = !isPenetrating;
  mainWindow.setIgnoreMouseEvents(isPenetrating, { forward: true });
  mainWindow.webContents.send("penetrate-changed", isPenetrating);
  console.log(`[Spike v3] 穿透: ${isPenetrating ? "开" : "关"}`);
}

ipcMain.on("toggle-penetrate", togglePenetrate);
ipcMain.on("set-ignore-mouse", (_e, ignore) => {
  isPenetrating = ignore;
  mainWindow.setIgnoreMouseEvents(ignore, { forward: true });
});
ipcMain.on("set-always-on-top", (_e, flag) => mainWindow.setAlwaysOnTop(flag));

// 全局快捷键
app.whenReady().then(() => {
  createWindow();
  globalShortcut.register("Ctrl+Shift+P", togglePenetrate);
});

app.on("will-quit", () => {
  globalShortcut.unregisterAll();
  if (visibilityTimer) clearInterval(visibilityTimer);
});
app.on("window-all-closed", () => app.quit());
app.on("activate", () => { if (BrowserWindow.getAllWindows().length === 0) createWindow(); });
