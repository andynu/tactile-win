use std::ptr;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, GetStockObject, HBRUSH, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
    CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, WHITE_BRUSH,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_STANDARD_CLASSES, ICC_UPDOWN_CLASS, INITCOMMONCONTROLSEX,
    UDM_SETPOS, UDM_SETRANGE, UDS_ALIGNRIGHT, UDS_ARROWKEYS, UDS_AUTOBUDDY, UDS_SETBUDDYINT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, EnumChildWindows, GetDlgItemInt,
    GetSystemMetrics, GetWindowRect, RegisterClassW, SendMessageW, SetWindowPos, ShowWindow,
    CS_HREDRAW, CS_VREDRAW, HWND_TOP, SM_CXSCREEN, SM_CYSCREEN, SWP_NOSIZE, SWP_NOZORDER,
    SW_SHOW, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_SETFONT, WNDCLASSW, WS_CAPTION,
    WS_CHILD, WS_EX_DLGMODALFRAME, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

use crate::config::Config;

const SETTINGS_CLASS_NAME: PCWSTR = w!("TactileWinSettings");

// Control IDs
const ID_COLS_EDIT: i32 = 101;
const ID_ROWS_EDIT: i32 = 103;
const ID_GAP_EDIT: i32 = 105;
const ID_SAVE_BTN: i32 = 110;
const ID_CANCEL_BTN: i32 = 111;

// Use atomic for HWND tracking since HWND is not Send
static SETTINGS_HWND: AtomicIsize = AtomicIsize::new(0);
static CURRENT_CONFIG: Mutex<Option<Config>> = Mutex::new(None);
static ON_SAVE_CALLBACK: Mutex<Option<fn(Config)>> = Mutex::new(None);

pub fn show_settings(config: Config, on_save: fn(Config)) {
    // Store config and callback
    if let Ok(mut guard) = CURRENT_CONFIG.lock() {
        *guard = Some(config);
    }
    if let Ok(mut guard) = ON_SAVE_CALLBACK.lock() {
        *guard = Some(on_save);
    }

    // Check if already open
    if SETTINGS_HWND.load(Ordering::SeqCst) != 0 {
        return; // Already open
    }

    unsafe {
        // Initialize common controls for modern visual styles
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_STANDARD_CLASSES | ICC_UPDOWN_CLASS,
        };
        let _ = InitCommonControlsEx(&icc);

        let hinstance = GetModuleHandleW(None).unwrap();

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(settings_window_proc),
            hInstance: hinstance.into(),
            lpszClassName: SETTINGS_CLASS_NAME,
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            ..Default::default()
        };

        let _ = RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME,
            SETTINGS_CLASS_NAME,
            w!("Tactile-Win Settings"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            100,
            100,
            320,
            220,
            None,
            None,
            Some(hinstance.into()),
            Some(ptr::null()),
        );

        if let Ok(hwnd) = hwnd {
            SETTINGS_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
    }
}

unsafe extern "system" fn settings_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => {
                create_controls(hwnd);
                set_dialog_font(hwnd);
                center_window(hwnd);
                LRESULT(0)
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as i32;
                match id {
                    ID_SAVE_BTN => {
                        save_settings(hwnd);
                        let _ = DestroyWindow(hwnd);
                    }
                    ID_CANCEL_BTN => {
                        let _ = DestroyWindow(hwnd);
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                SETTINGS_HWND.store(0, Ordering::SeqCst);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn create_controls(hwnd: HWND) {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap();

        // Get current config values
        let config = CURRENT_CONFIG
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_default();

        // Create labels and edit controls
        let label_style = WS_CHILD | WS_VISIBLE;
        let edit_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP;
        let button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP;

        // Columns
        let _ = CreateWindowExW(
            Default::default(),
            w!("STATIC"),
            w!("Columns (1-8):"),
            label_style,
            20,
            20,
            100,
            20,
            Some(hwnd),
            None,
            Some(hinstance.into()),
            Some(ptr::null()),
        );

        let cols_edit = CreateWindowExW(
            Default::default(),
            w!("EDIT"),
            PCWSTR::null(),
            edit_style,
            130,
            18,
            50,
            22,
            Some(hwnd),
            Some(std::mem::transmute::<isize, _>(ID_COLS_EDIT as isize)),
            Some(hinstance.into()),
            Some(ptr::null()),
        );

        if cols_edit.is_ok() {
            // Create up-down control
            let updown_style = WS_CHILD
                | WS_VISIBLE
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                    UDS_SETBUDDYINT | UDS_ALIGNRIGHT | UDS_ARROWKEYS | UDS_AUTOBUDDY,
                );

            let updown = CreateWindowExW(
                Default::default(),
                w!("msctls_updown32"),
                PCWSTR::null(),
                updown_style,
                0,
                0,
                0,
                0,
                Some(hwnd),
                None,
                Some(hinstance.into()),
                Some(ptr::null()),
            );

            if let Ok(ud) = updown {
                // Range: LOWORD = max, HIWORD = min (for UDM_SETRANGE)
                let _ = SendMessageW(
                    ud,
                    UDM_SETRANGE,
                    Some(WPARAM(0)),
                    Some(LPARAM(((1 << 16) | 8) as isize)),
                );
                let _ = SendMessageW(
                    ud,
                    UDM_SETPOS,
                    Some(WPARAM(0)),
                    Some(LPARAM(config.grid.cols as isize)),
                );
            }
        }

        // Rows
        let _ = CreateWindowExW(
            Default::default(),
            w!("STATIC"),
            w!("Rows (1-4):"),
            label_style,
            20,
            50,
            100,
            20,
            Some(hwnd),
            None,
            Some(hinstance.into()),
            Some(ptr::null()),
        );

        let rows_edit = CreateWindowExW(
            Default::default(),
            w!("EDIT"),
            PCWSTR::null(),
            edit_style,
            130,
            48,
            50,
            22,
            Some(hwnd),
            Some(std::mem::transmute::<isize, _>(ID_ROWS_EDIT as isize)),
            Some(hinstance.into()),
            Some(ptr::null()),
        );

        if rows_edit.is_ok() {
            let updown_style = WS_CHILD
                | WS_VISIBLE
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                    UDS_SETBUDDYINT | UDS_ALIGNRIGHT | UDS_ARROWKEYS | UDS_AUTOBUDDY,
                );

            let updown = CreateWindowExW(
                Default::default(),
                w!("msctls_updown32"),
                PCWSTR::null(),
                updown_style,
                0,
                0,
                0,
                0,
                Some(hwnd),
                None,
                Some(hinstance.into()),
                Some(ptr::null()),
            );

            if let Ok(ud) = updown {
                let _ = SendMessageW(
                    ud,
                    UDM_SETRANGE,
                    Some(WPARAM(0)),
                    Some(LPARAM(((1 << 16) | 4) as isize)),
                );
                let _ = SendMessageW(
                    ud,
                    UDM_SETPOS,
                    Some(WPARAM(0)),
                    Some(LPARAM(config.grid.rows as isize)),
                );
            }
        }

        // Gap
        let _ = CreateWindowExW(
            Default::default(),
            w!("STATIC"),
            w!("Gap (pixels):"),
            label_style,
            20,
            80,
            100,
            20,
            Some(hwnd),
            None,
            Some(hinstance.into()),
            Some(ptr::null()),
        );

        let gap_edit = CreateWindowExW(
            Default::default(),
            w!("EDIT"),
            PCWSTR::null(),
            edit_style,
            130,
            78,
            50,
            22,
            Some(hwnd),
            Some(std::mem::transmute::<isize, _>(ID_GAP_EDIT as isize)),
            Some(hinstance.into()),
            Some(ptr::null()),
        );

        if gap_edit.is_ok() {
            let updown_style = WS_CHILD
                | WS_VISIBLE
                | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(
                    UDS_SETBUDDYINT | UDS_ALIGNRIGHT | UDS_ARROWKEYS | UDS_AUTOBUDDY,
                );

            let updown = CreateWindowExW(
                Default::default(),
                w!("msctls_updown32"),
                PCWSTR::null(),
                updown_style,
                0,
                0,
                0,
                0,
                Some(hwnd),
                None,
                Some(hinstance.into()),
                Some(ptr::null()),
            );

            if let Ok(ud) = updown {
                let _ = SendMessageW(
                    ud,
                    UDM_SETRANGE,
                    Some(WPARAM(0)),
                    Some(LPARAM(((0 << 16) | 50) as isize)),
                );
                let _ = SendMessageW(
                    ud,
                    UDM_SETPOS,
                    Some(WPARAM(0)),
                    Some(LPARAM(config.grid.gap as isize)),
                );
            }
        }

        // Save button
        let _ = CreateWindowExW(
            Default::default(),
            w!("BUTTON"),
            w!("Save"),
            button_style,
            60,
            130,
            80,
            30,
            Some(hwnd),
            Some(std::mem::transmute::<isize, _>(ID_SAVE_BTN as isize)),
            Some(hinstance.into()),
            Some(ptr::null()),
        );

        // Cancel button
        let _ = CreateWindowExW(
            Default::default(),
            w!("BUTTON"),
            w!("Cancel"),
            button_style,
            160,
            130,
            80,
            30,
            Some(hwnd),
            Some(std::mem::transmute::<isize, _>(ID_CANCEL_BTN as isize)),
            Some(hinstance.into()),
            Some(ptr::null()),
        );
    }
}

unsafe fn center_window(hwnd: HWND) {
    unsafe {
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        // Get screen dimensions
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        let x = (screen_width - width) / 2;
        let y = (screen_height - height) / 2;

        let _ = SetWindowPos(hwnd, Some(HWND_TOP), x, y, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
    }
}

// Callback for EnumChildWindows to set font on all children
unsafe extern "system" fn set_font_callback(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    unsafe {
        let font = lparam.0 as *mut std::ffi::c_void;
        let _ = SendMessageW(hwnd, WM_SETFONT, Some(WPARAM(font as usize)), Some(LPARAM(1)));
    }
    windows::core::BOOL(1) // Continue enumeration
}

unsafe fn set_dialog_font(hwnd: HWND) {
    unsafe {
        // Create Segoe UI font (Windows default UI font)
        let font = CreateFontW(
            -12, // Height (negative for character height)
            0,
            0,
            0,
            400, // Normal weight
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY,
            0,
            w!("Segoe UI"),
        );

        // Set font on all child windows
        let _ = EnumChildWindows(
            Some(hwnd),
            Some(set_font_callback),
            LPARAM(font.0 as isize),
        );
    }
}

unsafe fn save_settings(hwnd: HWND) {
    unsafe {
        let cols = GetDlgItemInt(hwnd, ID_COLS_EDIT, None, false);
        let rows = GetDlgItemInt(hwnd, ID_ROWS_EDIT, None, false);
        let gap = GetDlgItemInt(hwnd, ID_GAP_EDIT, None, false) as i32;

        // Get current config and update it
        let mut config = CURRENT_CONFIG
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_default();
        config.grid.cols = cols.clamp(1, 8);
        config.grid.rows = rows.clamp(1, 4);
        config.grid.gap = gap.clamp(0, 50);

        // Save to file
        if let Err(e) = config.save() {
            eprintln!("Failed to save config: {}", e);
        }

        // Call the callback
        if let Ok(guard) = ON_SAVE_CALLBACK.lock() {
            if let Some(callback) = *guard {
                callback(config);
            }
        }
    }
}
