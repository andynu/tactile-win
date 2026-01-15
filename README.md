# Tactile-Win

A keyboard-driven window tiler for Windows, inspired by [Tactile](https://extensions.gnome.org/extension/4548/tactile/) for GNOME.

## Usage

1. Press **Ctrl+Shift+G** to show the grid overlay
2. Press two keys to select a tile region (e.g., `Q` then `F` for top-left half)
3. The active window snaps to the selected region
4. Press **Escape** to cancel

### Key Layout

For 1-3 row grids:
```
Row 0: Q W E R T Y U I
Row 1: A S D F G H J K
Row 2: Z X C V B N M ,
```

For 4 row grids (number row added at top):
```
Row 0: 1 2 3 4 5 6 7 8
Row 1: Q W E R T Y U I
Row 2: A S D F G H J K
Row 3: Z X C V B N M ,
```

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/tactile-win.exe`.

## Configuration

Create `~/.tactile-win.toml` (i.e., `C:\Users\<username>\.tactile-win.toml`):

```toml
[grid]
cols = 4    # 1-8 columns
rows = 2    # 1-4 rows
gap = 10    # pixels between tiles

[appearance]
tile_color = 0x00805030       # BGR format
highlight_color = 0x0000A0FF  # BGR format (orange)
background_color = 0x00302020
text_color = 0x00FFFFFF
alpha = 220                   # 0-255 transparency
```

## Using Win+T Instead of Ctrl+Shift+G

By default, Win+T is reserved by Windows for cycling taskbar items. To use Win+T with Tactile-Win (matching Linux Tactile's Super+T):

1. Run `disable-win-t.reg` to disable the system shortcut
2. Log off and back on (or restart Explorer)
3. Modify the hotkey in the source code to use `MOD_WIN` + `VK_T`

To restore the default Windows behavior, run `enable-win-t.reg`.

## System Tray

Tactile-Win runs in the system tray. Right-click the icon for:
- **About** - Version info
- **Quit** - Exit the application

## Building from Source

Requires Rust 1.70+:

```bash
cargo build --release
```

## License

MIT
