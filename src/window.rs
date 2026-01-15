use std::cell::RefCell;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, HDC, HMONITOR, MONITORINFO,
    MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowPlacement, SetWindowPlacement, SetWindowPos, HWND_TOP,
    SET_WINDOW_POS_FLAGS, SHOW_WINDOW_CMD, SWP_NOACTIVATE, SWP_NOZORDER, SW_MAXIMIZE, SW_MINIMIZE,
    SW_RESTORE, WINDOWPLACEMENT,
};

pub fn get_foreground_window() -> Option<HWND> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            None
        } else {
            Some(hwnd)
        }
    }
}

pub fn get_work_area(hwnd: HWND) -> Option<RECT> {
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if monitor.is_invalid() {
            return None;
        }

        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            Some(info.rcWork)
        } else {
            None
        }
    }
}

/// Get work area for a specific monitor by HMONITOR
pub fn get_monitor_work_area(monitor: HMONITOR) -> Option<RECT> {
    unsafe {
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            Some(info.rcWork)
        } else {
            None
        }
    }
}

// Thread-local storage for monitor enumeration callback
thread_local! {
    static MONITOR_LIST: RefCell<Vec<HMONITOR>> = const { RefCell::new(Vec::new()) };
}

unsafe extern "system" fn monitor_enum_proc(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    _lparam: LPARAM,
) -> windows::core::BOOL {
    MONITOR_LIST.with(|list| {
        list.borrow_mut().push(monitor);
    });
    windows::core::BOOL(1) // Continue enumeration
}

/// Get all monitors in the system
pub fn get_all_monitors() -> Vec<HMONITOR> {
    unsafe {
        // Clear the list
        MONITOR_LIST.with(|list| {
            list.borrow_mut().clear();
        });

        // Enumerate all monitors
        let _ = EnumDisplayMonitors(None, None, Some(monitor_enum_proc), LPARAM(0));

        // Return a copy of the list
        MONITOR_LIST.with(|list| list.borrow().clone())
    }
}

/// Get the monitor that contains the given window
pub fn get_window_monitor(hwnd: HWND) -> HMONITOR {
    unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) }
}

pub fn restore_if_maximized(hwnd: HWND) {
    unsafe {
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };

        if GetWindowPlacement(hwnd, &mut placement).is_ok() {
            // Check if window is maximized or minimized
            let show_cmd = SHOW_WINDOW_CMD(placement.showCmd as i32);
            if show_cmd == SW_MAXIMIZE || show_cmd == SW_MINIMIZE {
                placement.showCmd = SW_RESTORE.0 as u32;
                let _ = SetWindowPlacement(hwnd, &placement);
            }
        }
    }
}

pub fn move_window(hwnd: HWND, rect: &RECT) -> windows::core::Result<()> {
    unsafe {
        // First restore if maximized
        restore_if_maximized(hwnd);

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            rect.left,
            rect.top,
            width,
            height,
            SET_WINDOW_POS_FLAGS(SWP_NOZORDER.0 | SWP_NOACTIVATE.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_foreground_window() {
        // This test may fail in headless environments
        let hwnd = get_foreground_window();
        // Just verify it doesn't crash - may or may not return a window
        println!("Foreground window: {:?}", hwnd);
    }
}
