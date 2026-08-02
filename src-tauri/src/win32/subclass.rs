use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::UI::Shell::*;

const SUBCLASS_ID: usize = 1;
const SW_SHOWNOACTIVATE: i32 = 4;

const HTTRANSPARENT: isize = -1;
/// TECH DEBT: PEN_PTR is a raw pointer to lib.rs's PEN AtomicBool.
/// This is necessary because SetWindowSubclass callbacks cannot capture closures.
/// The pointer is set exactly once in install_guard() before the window message
/// loop starts, and `PEN` in lib.rs is a `static AtomicBool` — so the pointer
/// remains valid for the entire lifetime of the application.
///
/// Potential improvement: pass a heap-allocated `Arc<AtomicBool>` via
/// SetWindowSubclass's dwRefData parameter instead of using static mut.
/// This is documented in docs/code-review.md §1.1 and deferred to a future
/// iteration due to the risk of touching the Win32 subclassing hot path.
static mut PEN_PTR: *const AtomicBool = std::ptr::null();

unsafe fn is_penetrating() -> bool {
    if PEN_PTR.is_null() { return false; }
    (*PEN_PTR).load(Ordering::SeqCst)
}

pub unsafe fn install_guard(h: *mut c_void, pen: &AtomicBool) -> bool {
    PEN_PTR = pen as *const AtomicBool;
    SetWindowSubclass(h, Some(subclass_proc), SUBCLASS_ID, 0) != 0
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
    _id: usize, _data: usize,
) -> LRESULT {
    // Win+D guard
    if msg == WM_SHOWWINDOW && wparam == 0 && lparam == 0 {
        let h = hwnd as isize;
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(1));
            // Safety: verify the window still exists before calling ShowWindow.
            // Without this check, a race between app exit and the guard thread
            // would cause use-after-free on the HWND.
            unsafe {
                if IsWindow(h as *mut c_void) != 0 {
                    ShowWindow(h as *mut c_void, SW_SHOWNOACTIVATE);
                }
            }
        });
        return 0;
    }

    // WM_NCHITTEST: reclaim handle zone when penetrating
    // set_ignore_cursor_events sets WS_EX_TRANSPARENT on the main window,
    // which normally makes ALL clicks pass through. This handler overrides
    // that behavior for the top 30px handle zone.
    if msg == WM_NCHITTEST && is_penetrating() {
        let screen_y = ((lparam as u32 >> 16) & 0xFFFF) as i16 as i32;

        let mut wr: RECT = std::mem::zeroed();
        if GetWindowRect(hwnd, &mut wr) != 0 {
            let client_y = screen_y - wr.top;
            if client_y >= 0 && client_y <= 30 {
                // Handle zone: return default (HTCLIENT) so DOM handles clicks
                return DefSubclassProc(hwnd, msg, wparam, lparam);
            }
        }
        // Content area: return HTTRANSPARENT, combined with WS_EX_TRANSPARENT
        // this makes the click pass through to the desktop
        return HTTRANSPARENT;
    }

    DefSubclassProc(hwnd, msg, wparam, lparam)
}
