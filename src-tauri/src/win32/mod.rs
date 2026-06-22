pub mod subclass;

use std::ffi::c_void;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

type HWND = *mut c_void;
type BOOL = i32;

fn encode_wide(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

const GWL_EXSTYLE: i32 = -20;
const GWL_HWNDPARENT: i32 = -8;
const WS_EX_LAYERED: u32 = 0x80000;
const LWA_ALPHA: u32 = 0x2;

static mut ORIGINAL_PARENT: isize = 0;

/// Embed window behind desktop icons (like a desktop gadget).
/// Win+D will NOT hide it — it's part of the desktop.
pub unsafe fn embed_desktop(h: *mut c_void) -> bool {
    // Save original parent (NULL = desktop) so we can restore later
    ORIGINAL_PARENT = GetWindowLongPtrW(h, GWL_HWNDPARENT);

    // Find Progman
    let progman = FindWindowW(encode_wide("Progman").as_ptr(), std::ptr::null());
    if progman.is_null() { return false; }

    // Send 0x052C to Progman to trigger WorkerW creation
    let _ = SendMessageW(progman, 0x052C, 0, 0);

    // Find the WorkerW that has SHELLDLL_DefView as child
    let mut worker_w: *mut c_void = std::ptr::null_mut();
    let _ = EnumWindows(Some(enum_proc), &mut worker_w as *mut *mut c_void as isize);

    if worker_w.is_null() {
        // Fallback: embed directly to Progman
        SetWindowLongPtrW(h, GWL_HWNDPARENT, progman as isize);
    } else {
        SetWindowLongPtrW(h, GWL_HWNDPARENT, worker_w as isize);
    }
    true
}

/// Restore window to normal top-level state (detach from desktop).
pub unsafe fn unembed_desktop(h: *mut c_void) {
    SetWindowLongPtrW(h, GWL_HWNDPARENT, ORIGINAL_PARENT);
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: isize) -> BOOL {
    let ptr = lparam as *mut *mut c_void;
    // Find WorkerW
    let mut buf = [0u16; 256];
    let len = GetClassNameW(hwnd, buf.as_mut_ptr(), 256);
    if len > 0 {
        let name = String::from_utf16_lossy(&buf[..len as usize]);
        if name == "WorkerW" {
            // Check if it has SHELLDLL_DefView as child
            let child = FindWindowExW(hwnd, std::ptr::null_mut(), encode_wide("SHELLDLL_DefView").as_ptr(), std::ptr::null());
            if !child.is_null() {
                *ptr = hwnd as *mut c_void;
                return 0; // found, stop
            }
        }
    }
    1 // continue
}

pub unsafe fn apply_styles(h: *mut c_void) {
    let ex = GetWindowLongPtrW(h, GWL_EXSTYLE) as u32;
    SetWindowLongPtrW(h, GWL_EXSTYLE, (ex | WS_EX_LAYERED) as isize);
    SetLayeredWindowAttributes(h, 0, 240, LWA_ALPHA);
}
