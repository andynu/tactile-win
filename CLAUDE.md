# Claude Code Guidelines for Tactile-Win

## Commit Early and Often
- Commit after completing each feature or logical unit of work
- Don't let changes accumulate across multiple features
- Each commit should be atomic and self-contained

## Hotkey Selection
- Don't use Ctrl+Alt+Space - it's used by Claude Code itself
- Current hotkey: Ctrl+Shift+G (for "Grid")

## Grid Key Layout
- For 1-3 rows: Start with Q row (QWER/ASDF/ZXCV)
- For 4 rows: Include number row (1234/QWER/ASDF/ZXCV)
- Columns map to keys left-to-right within each row

## Multi-Monitor Behavior
- Show overlay on the monitor where the target window currently is
- Tab key cycles to next monitor
- Window gets moved to selected region on current overlay's monitor

## Windows API Notes
- Use windows-rs crate (currently v0.61)
- BOOL type is in `windows::core::BOOL`, not `Win32::Foundation`
- HWND is not Send-safe - use AtomicIsize for static storage
- Enable visual styles via manifest for modern controls appearance
