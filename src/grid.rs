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

    pub fn key_to_tile(&self, key: char) -> Option<TilePos> {
        let key = key.to_ascii_uppercase();

        // Row 0: Q W E R (cols 0-3, but limited by grid cols)
        // Row 1: A S D F (cols 0-3, but limited by grid cols)
        let (row, col) = match key {
            'Q' => (0, 0),
            'W' => (0, 1),
            'E' => (0, 2),
            'R' => (0, 3),
            'A' => (1, 0),
            'S' => (1, 1),
            'D' => (1, 2),
            'F' => (1, 3),
            _ => return None,
        };

        if col < self.cols && row < self.rows {
            Some(TilePos { col, row })
        } else {
            None
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
    fn test_key_to_tile() {
        let grid = Grid::new(4, 2, 10, test_work_area());

        assert_eq!(grid.key_to_tile('q'), Some(TilePos { col: 0, row: 0 }));
        assert_eq!(grid.key_to_tile('W'), Some(TilePos { col: 1, row: 0 }));
        assert_eq!(grid.key_to_tile('a'), Some(TilePos { col: 0, row: 1 }));
        assert_eq!(grid.key_to_tile('F'), Some(TilePos { col: 3, row: 1 }));
        assert_eq!(grid.key_to_tile('x'), None);
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
