use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::Mutex;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_ESCAPE, VK_TAB, VIRTUAL_KEY};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN,
};

static HOOK_ACTIVE: AtomicBool = AtomicBool::new(false);
static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInput {
    GridKey(char),
    Tab,
    Escape,
    Other,
}

pub type KeyCallback = fn(KeyInput);

static KEY_CALLBACK: Mutex<Option<KeyCallback>> = Mutex::new(None);

fn vk_to_char(vk: u32) -> Option<char> {
    match vk {
        // Number keys (top row for 4-row grids)
        0x31 => Some('1'), // VK_1
        0x32 => Some('2'), // VK_2
        0x33 => Some('3'), // VK_3
        0x34 => Some('4'), // VK_4
        0x35 => Some('5'), // VK_5
        0x36 => Some('6'), // VK_6
        0x37 => Some('7'), // VK_7
        0x38 => Some('8'), // VK_8
        // QWERTY row
        0x51 => Some('Q'), // VK_Q
        0x57 => Some('W'), // VK_W
        0x45 => Some('E'), // VK_E
        0x52 => Some('R'), // VK_R
        0x54 => Some('T'), // VK_T
        0x59 => Some('Y'), // VK_Y
        0x55 => Some('U'), // VK_U
        0x49 => Some('I'), // VK_I
        // ASDF row
        0x41 => Some('A'), // VK_A
        0x53 => Some('S'), // VK_S
        0x44 => Some('D'), // VK_D
        0x46 => Some('F'), // VK_F
        0x47 => Some('G'), // VK_G
        0x48 => Some('H'), // VK_H
        0x4A => Some('J'), // VK_J
        0x4B => Some('K'), // VK_K
        // ZXCV row
        0x5A => Some('Z'), // VK_Z
        0x58 => Some('X'), // VK_X
        0x43 => Some('C'), // VK_C
        0x56 => Some('V'), // VK_V
        0x42 => Some('B'), // VK_B
        0x4E => Some('N'), // VK_N
        0x4D => Some('M'), // VK_M
        0xBC => Some(','), // VK_OEM_COMMA
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
            } else if VIRTUAL_KEY(vk as u16) == VK_TAB {
                KeyInput::Tab
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
