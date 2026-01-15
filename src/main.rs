mod grid;
mod keyboard;
mod overlay;
mod selection;
mod window;

use std::cell::RefCell;
use std::ptr;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_CONTROL, MOD_SHIFT, VK_G,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW,
    TranslateMessage, HWND_MESSAGE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_HOTKEY, WNDCLASSW,
};

use crate::grid::Grid;
use crate::keyboard::{install_keyboard_hook, set_hook_active, uninstall_keyboard_hook, KeyInput};
use crate::overlay::Overlay;
use crate::selection::{SelectionState, TileSelector};
use crate::window::{get_foreground_window, get_work_area, move_window};

const CLASS_NAME: PCWSTR = w!("TactileWinClass");
const HOTKEY_ID: i32 = 1;

thread_local! {
    static APP_STATE: RefCell<Option<AppState>> = const { RefCell::new(None) };
}

struct AppState {
    overlay: Option<Overlay>,
    selector: Option<TileSelector>,
    target_hwnd: Option<HWND>,
}

fn handle_hotkey() {
    APP_STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(ref mut app) = *state {
            // Get the foreground window before showing overlay
            app.target_hwnd = get_foreground_window();

            if let Some(target) = app.target_hwnd {
                if let Some(work_area) = get_work_area(target) {
                    // Create overlay if needed
                    if app.overlay.is_none() {
                        app.overlay = Overlay::new(work_area).ok();
                    }

                    // Create selector
                    let grid = Grid::new(4, 2, 10, work_area);
                    app.selector = Some(TileSelector::new(grid));

                    // Show overlay and activate keyboard hook
                    if let Some(ref overlay) = app.overlay {
                        overlay.show();
                        set_hook_active(true);
                    }
                }
            }
        }
    });
}

fn handle_key_input(input: KeyInput) {
    APP_STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(ref mut app) = *state {
            match input {
                KeyInput::Escape => {
                    // Cancel and hide overlay
                    if let Some(ref mut selector) = app.selector {
                        selector.cancel();
                    }
                    if let Some(ref overlay) = app.overlay {
                        overlay.hide();
                        overlay.set_highlight(None);
                    }
                    set_hook_active(false);
                }
                KeyInput::GridKey(key) => {
                    if let Some(ref mut selector) = app.selector {
                        let new_state = selector.handle_key(key);

                        match new_state {
                            SelectionState::FirstKeyPressed(pos) => {
                                // Highlight the first tile
                                if let Some(ref overlay) = app.overlay {
                                    overlay.set_highlight(Some(pos));
                                }
                            }
                            SelectionState::Complete(rect) => {
                                // Move the window and hide overlay
                                if let Some(target) = app.target_hwnd {
                                    let _ = move_window(target, &rect);
                                }
                                if let Some(ref overlay) = app.overlay {
                                    overlay.hide();
                                    overlay.set_highlight(None);
                                }
                                set_hook_active(false);
                            }
                            _ => {}
                        }
                    }
                }
                KeyInput::Other => {
                    // Ignore other keys
                }
            }
        }
    });
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_HOTKEY => {
                if wparam.0 as i32 == HOTKEY_ID {
                    handle_hotkey();
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn register_hotkey(hwnd: HWND) -> windows::core::Result<()> {
    unsafe {
        // Use Ctrl+Shift+G for Grid
        RegisterHotKey(
            Some(hwnd),
            HOTKEY_ID,
            HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_SHIFT.0),
            VK_G.0 as u32,
        )?;
        println!("Registered Ctrl+Shift+G hotkey - press it to show grid overlay");
        Ok(())
    }
}

fn unregister_hotkey(hwnd: HWND) {
    unsafe {
        let _ = UnregisterHotKey(Some(hwnd), HOTKEY_ID);
    }
}

fn create_message_window() -> windows::core::Result<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;

        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: hinstance.into(),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(windows::core::Error::from_win32());
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CLASS_NAME,
            w!("Tactile-Win"),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(hinstance.into()),
            Some(ptr::null()),
        )?;

        Ok(hwnd)
    }
}

fn run_message_loop() {
    unsafe {
        let mut msg = MSG::default();

        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn main() {
    println!("Tactile-Win starting...");
    println!("Press Ctrl+Shift+G to show the tiling grid overlay");
    println!("Then press two keys (Q/W/E/R/A/S/D/F) to select a tile region");
    println!("Press Escape to cancel");

    match create_message_window() {
        Ok(hwnd) => {
            // Initialize app state
            APP_STATE.with(|state| {
                *state.borrow_mut() = Some(AppState {
                    overlay: None,
                    selector: None,
                    target_hwnd: None,
                });
            });

            // Install keyboard hook with direct callback
            if let Err(e) = install_keyboard_hook(handle_key_input) {
                eprintln!("Failed to install keyboard hook: {}", e);
                return;
            }
            set_hook_active(false); // Start with hook inactive

            if let Err(e) = register_hotkey(hwnd) {
                eprintln!("Failed to register hotkey: {}", e);
                return;
            }

            run_message_loop();

            uninstall_keyboard_hook();
            unregister_hotkey(hwnd);
        }
        Err(e) => {
            eprintln!("Failed to create message window: {}", e);
        }
    }
}
