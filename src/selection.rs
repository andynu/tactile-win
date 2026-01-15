use crate::grid::{Grid, TilePos};
use windows::Win32::Foundation::RECT;

#[derive(Debug, Clone, Copy)]
pub enum SelectionState {
    Idle,
    FirstKeyPressed(TilePos),
    Complete(RECT),
    Cancelled,
}

impl PartialEq for SelectionState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SelectionState::Idle, SelectionState::Idle) => true,
            (SelectionState::FirstKeyPressed(a), SelectionState::FirstKeyPressed(b)) => a == b,
            (SelectionState::Complete(a), SelectionState::Complete(b)) => {
                a.left == b.left && a.top == b.top && a.right == b.right && a.bottom == b.bottom
            }
            (SelectionState::Cancelled, SelectionState::Cancelled) => true,
            _ => false,
        }
    }
}

impl Eq for SelectionState {}

pub struct TileSelector {
    state: SelectionState,
    grid: Grid,
}

impl TileSelector {
    pub fn new(grid: Grid) -> Self {
        Self {
            state: SelectionState::Idle,
            grid,
        }
    }

    pub fn handle_key(&mut self, key: char) -> SelectionState {
        match self.state {
            SelectionState::Idle => {
                if let Some(pos) = self.grid.key_to_tile(key) {
                    self.state = SelectionState::FirstKeyPressed(pos);
                }
                self.state
            }
            SelectionState::FirstKeyPressed(first_pos) => {
                if let Some(second_pos) = self.grid.key_to_tile(key) {
                    let rect = if first_pos == second_pos {
                        // Same key twice = single tile
                        self.grid.tile_rect(first_pos)
                    } else {
                        // Different keys = combined rectangle
                        self.grid.combine_tiles(first_pos, second_pos)
                    };
                    self.state = SelectionState::Complete(rect);
                }
                self.state
            }
            SelectionState::Complete(_) | SelectionState::Cancelled => {
                // Already complete or cancelled, ignore further input
                self.state
            }
        }
    }

    pub fn cancel(&mut self) {
        self.state = SelectionState::Cancelled;
    }

    pub fn reset(&mut self) {
        self.state = SelectionState::Idle;
    }

    pub fn state(&self) -> SelectionState {
        self.state
    }

    pub fn first_tile(&self) -> Option<TilePos> {
        match self.state {
            SelectionState::FirstKeyPressed(pos) => Some(pos),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_grid() -> Grid {
        Grid::new(
            4,
            2,
            10,
            RECT {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            },
        )
    }

    #[test]
    fn test_single_tile_selection() {
        let mut selector = TileSelector::new(test_grid());

        // First Q
        let state = selector.handle_key('Q');
        assert!(matches!(state, SelectionState::FirstKeyPressed(_)));

        // Second Q (same key)
        let state = selector.handle_key('Q');
        assert!(matches!(state, SelectionState::Complete(_)));
    }

    #[test]
    fn test_two_tile_selection() {
        let mut selector = TileSelector::new(test_grid());

        // First Q
        selector.handle_key('Q');

        // Second F (different key)
        let state = selector.handle_key('F');
        assert!(matches!(state, SelectionState::Complete(_)));
    }

    #[test]
    fn test_cancel() {
        let mut selector = TileSelector::new(test_grid());

        selector.handle_key('Q');
        selector.cancel();

        assert_eq!(selector.state(), SelectionState::Cancelled);
    }

    #[test]
    fn test_reset() {
        let mut selector = TileSelector::new(test_grid());

        selector.handle_key('Q');
        selector.handle_key('F');
        selector.reset();

        assert_eq!(selector.state(), SelectionState::Idle);
    }
}
