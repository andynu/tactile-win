use std::ptr;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, GetCursorPos,
    LoadImageW, PostQuitMessage, RegisterClassW, SetForegroundWindow, TrackPopupMenu,
    IMAGE_ICON, LR_DEFAULTSIZE, LR_SHARED, MF_STRING, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSW,
};

const TRAY_CLASS_NAME: PCWSTR = w!("TactileWinTrayClass");
const WM_TRAYICON: u32 = 0x8000; // WM_APP

const IDM_QUIT: u16 = 1001;
const IDM_ABOUT: u16 = 1002;
const IDM_SETTINGS: u16 = 1003;

static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);
static SHOW_SETTINGS_CALLBACK: Mutex<Option<fn()>> = Mutex::new(None);

pub struct TrayIcon {
    hwnd: HWND,
}

unsafe extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_TRAYICON => {
                let event = lparam.0 as u32;
                match event {
                    x if x == WM_RBUTTONUP || x == WM_LBUTTONUP => {
                        show_context_menu(hwnd);
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let cmd = (wparam.0 & 0xFFFF) as u16;
                match cmd {
                    IDM_QUIT => {
                        PostQuitMessage(0);
                    }
                    IDM_ABOUT => {
                        // TODO: Show about dialog
                    }
                    IDM_SETTINGS => {
                        if let Ok(guard) = SHOW_SETTINGS_CALLBACK.lock() {
                            if let Some(callback) = *guard {
                                callback();
                            }
                        }
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn show_context_menu(hwnd: HWND) {
    unsafe {
        let menu = CreatePopupMenu().unwrap();

        let _ = AppendMenuW(menu, MF_STRING, IDM_SETTINGS as usize, w!("Settings..."));
        let _ = AppendMenuW(menu, MF_STRING, IDM_ABOUT as usize, w!("About Tactile-Win"));
        let _ = AppendMenuW(menu, MF_STRING, IDM_QUIT as usize, w!("Quit"));

        let mut pt = windows::Win32::Foundation::POINT::default();
        let _ = GetCursorPos(&mut pt);

        // Required for the menu to close when clicking elsewhere
        let _ = SetForegroundWindow(hwnd);

        TrackPopupMenu(menu, TPM_LEFTALIGN | TPM_BOTTOMALIGN, pt.x, pt.y, Some(0), hwnd, None);

        let _ = DestroyMenu(menu);
    }
}

impl TrayIcon {
    pub fn new() -> windows::core::Result<Self> {
        unsafe {
            let hinstance = GetModuleHandleW(None)?;

            let wc = WNDCLASSW {
                lpfnWndProc: Some(tray_window_proc),
                hInstance: hinstance.into(),
                lpszClassName: TRAY_CLASS_NAME,
                ..Default::default()
            };

            let _ = RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                TRAY_CLASS_NAME,
                w!("Tactile-Win Tray"),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                None,
                None,
                Some(hinstance.into()),
                Some(ptr::null()),
            )?;

            TRAY_HWND.store(hwnd.0 as isize, Ordering::SeqCst);

            // Load custom icon from resources (ID 1)
            let icon_handle = LoadImageW(
                Some(hinstance.into()),
                PCWSTR(1 as *const u16),  // Resource ID 1
                IMAGE_ICON,
                0,
                0,
                LR_DEFAULTSIZE | LR_SHARED,
            )?;
            let icon = HICON(icon_handle.0);

            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: icon,
                ..Default::default()
            };

            // Set tooltip
            let tip = "Tactile-Win (Ctrl+Shift+G)";
            let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();
            nid.szTip[..tip_wide.len().min(128)]
                .copy_from_slice(&tip_wide[..tip_wide.len().min(128)]);

            if !Shell_NotifyIconW(NIM_ADD, &nid).as_bool() {
                return Err(windows::core::Error::from_win32());
            }

            Ok(Self { hwnd })
        }
    }

    pub fn remove(&self) {
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: 1,
                ..Default::default()
            };

            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        self.remove();
    }
}

pub fn set_settings_callback(callback: fn()) {
    if let Ok(mut guard) = SHOW_SETTINGS_CALLBACK.lock() {
        *guard = Some(callback);
    }
}
