use std::ptr;
use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, EndPaint, FillRect, InvalidateRect,
    SelectObject, SetBkMode, SetTextColor, TextOutW, UpdateWindow, DEFAULT_CHARSET, HBRUSH,
    OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, SetLayeredWindowAttributes, ShowWindow,
    CS_HREDRAW, CS_VREDRAW, LAYERED_WINDOW_ATTRIBUTES_FLAGS, LWA_ALPHA, SW_HIDE, SW_SHOW,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_PAINT, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};

use crate::grid::{Grid, TilePos};

const OVERLAY_CLASS_NAME: PCWSTR = w!("TactileWinOverlay");
const OVERLAY_ALPHA: u8 = 220;

static OVERLAY_GRID: Mutex<Option<Grid>> = Mutex::new(None);
static HIGHLIGHT_TILE: Mutex<Option<TilePos>> = Mutex::new(None);

pub struct Overlay {
    hwnd: HWND,
}

fn draw_grid(hwnd: HWND) {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);

        // Dark background
        let bg_brush = CreateSolidBrush(COLORREF(0x00302020)); // Dark gray-brown
        FillRect(hdc, &ps.rcPaint, bg_brush);
        let _ = DeleteObject(bg_brush.into());

        // Get grid
        let grid_guard = OVERLAY_GRID.lock().ok();
        let highlight_guard = HIGHLIGHT_TILE.lock().ok();

        if let Some(Some(ref grid)) = grid_guard.as_ref().map(|g| g.as_ref()) {
            // Create font for labels
            let font = CreateFontW(
                48,
                0,
                0,
                0,
                700, // Bold
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
            let old_font = SelectObject(hdc, font.into());
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLORREF(0x00FFFFFF)); // White text

            // Tile colors
            let tile_brush = CreateSolidBrush(COLORREF(0x00805030)); // Teal-ish
            let highlight_brush = CreateSolidBrush(COLORREF(0x0000A0FF)); // Orange highlight

            let keys = [
                ['Q', 'W', 'E', 'R'],
                ['A', 'S', 'D', 'F'],
            ];

            let highlight = highlight_guard.as_ref().and_then(|h| h.as_ref());

            for row in 0..grid.rows.min(2) {
                for col in 0..grid.cols.min(4) {
                    let pos = TilePos { col, row };
                    let rect = grid.tile_rect(pos);

                    // Adjust rect to be relative to overlay window (0,0 based)
                    let draw_rect = RECT {
                        left: rect.left - grid.work_area.left,
                        top: rect.top - grid.work_area.top,
                        right: rect.right - grid.work_area.left,
                        bottom: rect.bottom - grid.work_area.top,
                    };

                    // Choose brush based on highlight
                    let brush = if highlight == Some(&pos) {
                        highlight_brush
                    } else {
                        tile_brush
                    };

                    FillRect(hdc, &draw_rect, HBRUSH(brush.0));

                    // Draw key label centered
                    let key = keys[row as usize][col as usize];
                    let key_str: Vec<u16> = format!("{}", key).encode_utf16().collect();

                    let center_x = (draw_rect.left + draw_rect.right) / 2 - 15;
                    let center_y = (draw_rect.top + draw_rect.bottom) / 2 - 24;

                    TextOutW(hdc, center_x, center_y, &key_str);
                }
            }

            let _ = DeleteObject(tile_brush.into());
            let _ = DeleteObject(highlight_brush.into());
            SelectObject(hdc, old_font);
            let _ = DeleteObject(font.into());
        }

        EndPaint(hwnd, &ps);
    }
}

unsafe extern "system" fn overlay_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                draw_grid(hwnd);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

impl Overlay {
    pub fn new(work_area: RECT) -> windows::core::Result<Self> {
        unsafe {
            let hinstance = GetModuleHandleW(None)?;

            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(overlay_window_proc),
                hInstance: hinstance.into(),
                lpszClassName: OVERLAY_CLASS_NAME,
                ..Default::default()
            };

            let _ = RegisterClassW(&wc);

            let width = work_area.right - work_area.left;
            let height = work_area.bottom - work_area.top;

            let ex_style = WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW;
            let style = WS_POPUP;

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(ex_style.0),
                OVERLAY_CLASS_NAME,
                w!("Tactile-Win Overlay"),
                WINDOW_STYLE(style.0),
                work_area.left,
                work_area.top,
                width,
                height,
                None,
                None,
                Some(hinstance.into()),
                Some(ptr::null()),
            )?;

            SetLayeredWindowAttributes(
                hwnd,
                COLORREF(0),
                OVERLAY_ALPHA,
                LAYERED_WINDOW_ATTRIBUTES_FLAGS(LWA_ALPHA.0),
            )?;

            // Create and store grid
            let grid = Grid::new(4, 2, 10, work_area);
            if let Ok(mut guard) = OVERLAY_GRID.lock() {
                *guard = Some(grid);
            }

            Ok(Self { hwnd })
        }
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            let _ = UpdateWindow(self.hwnd);
        }
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    pub fn set_highlight(&self, pos: Option<TilePos>) {
        if let Ok(mut guard) = HIGHLIGHT_TILE.lock() {
            *guard = pos;
        }
        unsafe {
            let _ = InvalidateRect(Some(self.hwnd), None, true);
            let _ = UpdateWindow(self.hwnd);
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn grid(&self) -> Option<Grid> {
        OVERLAY_GRID.lock().ok().and_then(|g| g.clone())
    }
}
