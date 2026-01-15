use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Mutex;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VIRTUAL_KEY};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN,
};

static HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);
static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInput {
    GridKey(char),
    Escape,
    Other,
}

pub type KeyCallback = fn(KeyInput);

static KEY_CALLBACK: Mutex<Option<KeyCallback>> = Mutex::new(None);

fn vk_to_char(vk: u32) -> Option<char> {
    match vk {
        0x51 => Some('Q'), // VK_Q
        0x57 => Some('W'), // VK_W
        0x45 => Some('E'), // VK_E
        0x52 => Some('R'), // VK_R
        0x41 => Some('A'), // VK_A
        0x53 => Some('S'), // VK_S
        0x44 => Some('D'), // VK_D
        0x46 => Some('F'), // VK_F
        _ => None,
    }
}

unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if code >= 0 && wparam.0 as u32 == WM_KEYDOWN {
            let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk = kb_struct.vkCode;

            let input = if VIRTUAL_KEY(vk as u16) == VK_ESCAPE {
                KeyInput::Escape
            } else if let Some(c) = vk_to_char(vk) {
                KeyInput::GridKey(c)
            } else {
                KeyInput::Other
            };

            if let Ok(callback_guard) = KEY_CALLBACK.lock() {
                if let Some(callback) = *callback_guard {
                    callback(input);
                }
            }

            // Block the key from reaching other applications when hook is active
            if HOOK_ACTIVE.load(Ordering::SeqCst) && input != KeyInput::Other {
                return LRESULT(1);
            }
        }

        CallNextHookEx(None, code, wparam, lparam)
    }
}

pub fn install_keyboard_hook(callback: KeyCallback) -> windows::core::Result<()> {
    unsafe {
        if let Ok(mut callback_guard) = KEY_CALLBACK.lock() {
            *callback_guard = Some(callback);
        }

        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0)?;

        HOOK_HANDLE.store(hook.0 as isize, Ordering::SeqCst);
        HOOK_ACTIVE.store(true, Ordering::SeqCst);
        Ok(())
    }
}

pub fn uninstall_keyboard_hook() {
    unsafe {
        HOOK_ACTIVE.store(false, Ordering::SeqCst);

        let handle = HOOK_HANDLE.swap(0, Ordering::SeqCst);
        if handle != 0 {
            let _ = UnhookWindowsHookEx(HHOOK(handle as *mut _));
        }

        if let Ok(mut callback_guard) = KEY_CALLBACK.lock() {
            *callback_guard = None;
        }
    }
}

pub fn is_hook_active() -> bool {
    HOOK_ACTIVE.load(Ordering::SeqCst)
}

pub fn set_hook_active(active: bool) {
    HOOK_ACTIVE.store(active, Ordering::SeqCst);
}
