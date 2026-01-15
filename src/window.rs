use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
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
