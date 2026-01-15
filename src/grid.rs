use windows::Win32::Foundation::RECT;

#[derive(Clone)]
pub struct Grid {
    pub cols: u32,
    pub rows: u32,
    pub gap: i32,
    pub work_area: RECT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilePos {
    pub col: u32,
    pub row: u32,
}

impl Grid {
    pub fn new(cols: u32, rows: u32, gap: i32, work_area: RECT) -> Self {
        Self {
            cols,
            rows,
            gap,
            work_area,
        }
    }

    /// Key layout for up to 4 rows × 8 columns:
    ///
    /// For 1-3 rows (top-left starts with Q):
    ///   Row 0: Q W E R T Y U I
    ///   Row 1: A S D F G H J K
    ///   Row 2: Z X C V B N M ,
    ///
    /// For 4 rows (includes number row at top):
    ///   Row 0: 1 2 3 4 5 6 7 8
    ///   Row 1: Q W E R T Y U I
    ///   Row 2: A S D F G H J K
    ///   Row 3: Z X C V B N M ,
    ///
    /// Default 2×4 grid uses just QWER/ASDF (rows 0-1, cols 0-3)
    pub fn key_to_tile(&self, key: char) -> Option<TilePos> {
        let key = key.to_ascii_uppercase();

        // Determine row offset: for 4 rows, number keys are row 0
        let use_number_row = self.rows == 4;

        let (keyboard_row, col) = match key {
            // Number keys row
            '1' => (0, 0),
            '2' => (0, 1),
            '3' => (0, 2),
            '4' => (0, 3),
            '5' => (0, 4),
            '6' => (0, 5),
            '7' => (0, 6),
            '8' => (0, 7),
            // QWERTY row
            'Q' => (1, 0),
            'W' => (1, 1),
            'E' => (1, 2),
            'R' => (1, 3),
            'T' => (1, 4),
            'Y' => (1, 5),
            'U' => (1, 6),
            'I' => (1, 7),
            // ASDF row
            'A' => (2, 0),
            'S' => (2, 1),
            'D' => (2, 2),
            'F' => (2, 3),
            'G' => (2, 4),
            'H' => (2, 5),
            'J' => (2, 6),
            'K' => (2, 7),
            // ZXCV row
            'Z' => (3, 0),
            'X' => (3, 1),
            'C' => (3, 2),
            'V' => (3, 3),
            'B' => (3, 4),
            'N' => (3, 5),
            'M' => (3, 6),
            ',' => (3, 7),
            _ => return None,
        };

        // Map keyboard row to grid row
        let row = if use_number_row {
            // 4 rows: keyboard rows 0-3 map directly to grid rows 0-3
            keyboard_row
        } else {
            // 1-3 rows: keyboard rows 1-3 (QWER/ASDF/ZXCV) map to grid rows 0-2
            if keyboard_row == 0 {
                return None; // Number keys not used for <4 rows
            }
            keyboard_row - 1
        };

        if col < self.cols && row < self.rows {
            Some(TilePos { col, row })
        } else {
            None
        }
    }

    /// Returns the key character for a given tile position
    pub fn tile_to_key(&self, pos: TilePos) -> Option<char> {
        const KEYS_3ROW: [[char; 8]; 3] = [
            ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I'],
            ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K'],
            ['Z', 'X', 'C', 'V', 'B', 'N', 'M', ','],
        ];
        const KEYS_4ROW: [[char; 8]; 4] = [
            ['1', '2', '3', '4', '5', '6', '7', '8'],
            ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I'],
            ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K'],
            ['Z', 'X', 'C', 'V', 'B', 'N', 'M', ','],
        ];

        if pos.row >= self.rows || pos.col >= self.cols {
            return None;
        }

        if self.rows == 4 {
            Some(KEYS_4ROW[pos.row as usize][pos.col as usize])
        } else {
            Some(KEYS_3ROW[pos.row as usize][pos.col as usize])
        }
    }

    pub fn tile_rect(&self, pos: TilePos) -> RECT {
        let work_width = self.work_area.right - self.work_area.left;
        let work_height = self.work_area.bottom - self.work_area.top;

        let total_gap_x = self.gap * (self.cols as i32 + 1);
        let total_gap_y = self.gap * (self.rows as i32 + 1);

        let tile_width = (work_width - total_gap_x) / self.cols as i32;
        let tile_height = (work_height - total_gap_y) / self.rows as i32;

        let left = self.work_area.left + self.gap + (pos.col as i32 * (tile_width + self.gap));
        let top = self.work_area.top + self.gap + (pos.row as i32 * (tile_height + self.gap));

        RECT {
            left,
            top,
            right: left + tile_width,
            bottom: top + tile_height,
        }
    }

    pub fn combine_tiles(&self, pos1: TilePos, pos2: TilePos) -> RECT {
        let rect1 = self.tile_rect(pos1);
        let rect2 = self.tile_rect(pos2);

        RECT {
            left: rect1.left.min(rect2.left),
            top: rect1.top.min(rect2.top),
            right: rect1.right.max(rect2.right),
            bottom: rect1.bottom.max(rect2.bottom),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_work_area() -> RECT {
        RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        }
    }

    #[test]
    fn test_key_to_tile_2x4() {
        // Default 2 rows x 4 cols uses QWER/ASDF (rows 0-1, cols 0-3)
        let grid = Grid::new(4, 2, 10, test_work_area());

        assert_eq!(grid.key_to_tile('q'), Some(TilePos { col: 0, row: 0 }));
        assert_eq!(grid.key_to_tile('W'), Some(TilePos { col: 1, row: 0 }));
        assert_eq!(grid.key_to_tile('a'), Some(TilePos { col: 0, row: 1 }));
        assert_eq!(grid.key_to_tile('F'), Some(TilePos { col: 3, row: 1 }));
        // Row 2 (ZXCV) out of bounds for 2-row grid
        assert_eq!(grid.key_to_tile('z'), None);
        // Col 4+ out of bounds
        assert_eq!(grid.key_to_tile('t'), None);
        // Number keys not used for <4 rows
        assert_eq!(grid.key_to_tile('1'), None);
    }

    #[test]
    fn test_key_to_tile_3x6() {
        // 3 rows uses QWER/ASDF/ZXCV
        let grid = Grid::new(6, 3, 10, test_work_area());

        assert_eq!(grid.key_to_tile('q'), Some(TilePos { col: 0, row: 0 }));
        assert_eq!(grid.key_to_tile('y'), Some(TilePos { col: 5, row: 0 }));
        assert_eq!(grid.key_to_tile('a'), Some(TilePos { col: 0, row: 1 }));
        assert_eq!(grid.key_to_tile('z'), Some(TilePos { col: 0, row: 2 }));
        // Number keys not used for 3 rows
        assert_eq!(grid.key_to_tile('1'), None);
    }

    #[test]
    fn test_key_to_tile_4x8() {
        // 4 rows: number keys at top
        let grid = Grid::new(8, 4, 10, test_work_area());

        assert_eq!(grid.key_to_tile('1'), Some(TilePos { col: 0, row: 0 }));
        assert_eq!(grid.key_to_tile('8'), Some(TilePos { col: 7, row: 0 }));
        assert_eq!(grid.key_to_tile('q'), Some(TilePos { col: 0, row: 1 }));
        assert_eq!(grid.key_to_tile('i'), Some(TilePos { col: 7, row: 1 }));
        assert_eq!(grid.key_to_tile('a'), Some(TilePos { col: 0, row: 2 }));
        assert_eq!(grid.key_to_tile('k'), Some(TilePos { col: 7, row: 2 }));
        assert_eq!(grid.key_to_tile('z'), Some(TilePos { col: 0, row: 3 }));
        assert_eq!(grid.key_to_tile(','), Some(TilePos { col: 7, row: 3 }));
    }

    #[test]
    fn test_tile_to_key_2row() {
        let grid = Grid::new(4, 2, 10, test_work_area());

        assert_eq!(grid.tile_to_key(TilePos { col: 0, row: 0 }), Some('Q'));
        assert_eq!(grid.tile_to_key(TilePos { col: 0, row: 1 }), Some('A'));
        assert_eq!(grid.tile_to_key(TilePos { col: 3, row: 1 }), Some('F'));
    }

    #[test]
    fn test_tile_to_key_4row() {
        let grid = Grid::new(8, 4, 10, test_work_area());

        assert_eq!(grid.tile_to_key(TilePos { col: 0, row: 0 }), Some('1'));
        assert_eq!(grid.tile_to_key(TilePos { col: 0, row: 1 }), Some('Q'));
        assert_eq!(grid.tile_to_key(TilePos { col: 3, row: 2 }), Some('F'));
        assert_eq!(grid.tile_to_key(TilePos { col: 7, row: 3 }), Some(','));
    }

    #[test]
    fn test_tile_rect() {
        let grid = Grid::new(4, 2, 10, test_work_area());

        let rect = grid.tile_rect(TilePos { col: 0, row: 0 });
        assert_eq!(rect.left, 10);
        assert_eq!(rect.top, 10);
    }

    #[test]
    fn test_combine_tiles() {
        let grid = Grid::new(4, 2, 10, test_work_area());

        let combined = grid.combine_tiles(
            TilePos { col: 0, row: 0 },
            TilePos { col: 1, row: 1 },
        );

        let rect1 = grid.tile_rect(TilePos { col: 0, row: 0 });
        let rect2 = grid.tile_rect(TilePos { col: 1, row: 1 });

        assert_eq!(combined.left, rect1.left);
        assert_eq!(combined.top, rect1.top);
        assert_eq!(combined.right, rect2.right);
        assert_eq!(combined.bottom, rect2.bottom);
    }
}
