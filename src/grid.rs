use std::time::{Duration, Instant};

use crate::config::GridConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub col: usize,
    pub row: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub struct Grid {
    pub cols: usize,
    pub rows: usize,
    keycode_map: Vec<(u16, usize, usize)>,
    pub selection_timeout: Duration,
}

#[derive(Debug, PartialEq)]
pub enum SelectionAction {
    FirstSelected(Cell),
    Tile(Cell, Cell),
    Ignored,
}

pub enum SelectionState {
    Empty,
    FirstSelected { cell: Cell, selected_at: Instant },
}

// default keycode map for 4x3 QWERTY grid (used by default_4x3)
const DEFAULT_KEYCODE_MAP: &[(u16, usize, usize)] = &[
    (12, 0, 0), (13, 1, 0), (14, 2, 0), (15, 3, 0), // Q W E R
    (0, 0, 1),  (1, 1, 1),  (2, 2, 1),  (3, 3, 1),  // A S D F
    (6, 0, 2),  (7, 1, 2),  (8, 2, 2),  (9, 3, 2),  // Z X C V
];

impl Grid {
    pub fn default_4x3() -> Self {
        Self {
            cols: 4,
            rows: 3,
            keycode_map: DEFAULT_KEYCODE_MAP.to_vec(),
            selection_timeout: Duration::from_secs(1),
        }
    }

    /// Build a Grid from a validated GridConfig.
    pub fn from_config(config: &GridConfig) -> Result<Self, String> {
        Ok(Self {
            cols: config.cols,
            rows: config.rows,
            keycode_map: config.build_keycode_map()?,
            selection_timeout: config.selection_timeout(),
        })
    }

    pub fn cell_for_keycode(&self, keycode: u16) -> Option<Cell> {
        self.keycode_map
            .iter()
            .find(|(kc, col, row)| *kc == keycode && *col < self.cols && *row < self.rows)
            .map(|(_, col, row)| Cell { col: *col, row: *row })
    }

    /// bounding box of two cells in screen coordinates. order doesn't matter
    pub fn bounding_rect(&self, a: Cell, b: Cell, screen: Rect) -> Rect {
        let min_col = a.col.min(b.col);
        let max_col = a.col.max(b.col);
        let min_row = a.row.min(b.row);
        let max_row = a.row.max(b.row);

        let cell_width = screen.width / self.cols as f64;
        let cell_height = screen.height / self.rows as f64;

        Rect {
            x: screen.x + min_col as f64 * cell_width,
            y: screen.y + min_row as f64 * cell_height,
            width: (max_col - min_col + 1) as f64 * cell_width,
            height: (max_row - min_row + 1) as f64 * cell_height,
        }
    }
}

impl SelectionState {
    pub fn new() -> Self {
        Self::Empty
    }

    pub fn advance(&mut self, keycode: u16, grid: &Grid, now: Instant) -> SelectionAction {
        let cell = match grid.cell_for_keycode(keycode) {
            Some(c) => c,
            None => return SelectionAction::Ignored,
        };

        match self {
            Self::Empty => {
                *self = Self::FirstSelected { cell, selected_at: now };
                SelectionAction::FirstSelected(cell)
            }
            Self::FirstSelected { cell: first, selected_at } => {
                if now.duration_since(*selected_at) >= grid.selection_timeout {
                    // timed out, start over
                    *self = Self::FirstSelected { cell, selected_at: now };
                    SelectionAction::FirstSelected(cell)
                } else {
                    let first_cell = *first;
                    *self = Self::Empty;
                    SelectionAction::Tile(first_cell, cell)
                }
            }
        }
    }

    pub fn reset(&mut self) {
        *self = Self::Empty;
    }

    /// returns true if we just cleared a timed-out selection (caller should update UI)
    pub fn check_timeout(&mut self, now: Instant, grid: &Grid) -> bool {
        if let Self::FirstSelected { selected_at, .. } = self {
            if now.duration_since(*selected_at) >= grid.selection_timeout {
                *self = Self::Empty;
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> Grid {
        Grid::default_4x3()
    }

    fn screen() -> Rect {
        Rect { x: 0.0, y: 25.0, width: 1920.0, height: 1055.0 }
    }

    #[test]
    fn keycode_q_maps_to_top_left() {
        assert_eq!(grid().cell_for_keycode(12), Some(Cell { col: 0, row: 0 }));
    }

    #[test]
    fn keycode_v_maps_to_bottom_right() {
        assert_eq!(grid().cell_for_keycode(9), Some(Cell { col: 3, row: 2 }));
    }

    #[test]
    fn keycode_s_maps_to_middle() {
        assert_eq!(grid().cell_for_keycode(1), Some(Cell { col: 1, row: 1 }));
    }

    #[test]
    fn invalid_keycode_returns_none() {
        assert_eq!(grid().cell_for_keycode(5), None);  // G
        assert_eq!(grid().cell_for_keycode(53), None); // Escape
    }

    #[test]
    fn all_twelve_keys_map_correctly() {
        let g = grid();
        let expected = vec![
            (12, 0, 0), (13, 1, 0), (14, 2, 0), (15, 3, 0),
            (0, 0, 1),  (1, 1, 1),  (2, 2, 1),  (3, 3, 1),
            (6, 0, 2),  (7, 1, 2),  (8, 2, 2),  (9, 3, 2),
        ];
        for (kc, col, row) in expected {
            assert_eq!(
                g.cell_for_keycode(kc),
                Some(Cell { col, row }),
                "keycode {kc} should map to ({col}, {row})"
            );
        }
    }

    #[test]
    fn single_cell_rect() {
        let r = grid().bounding_rect(
            Cell { col: 0, row: 0 },
            Cell { col: 0, row: 0 },
            screen(),
        );
        assert_eq!(r.x, 0.0);
        assert_eq!(r.y, 25.0);
        assert_eq!(r.width, 480.0);
        assert!((r.height - 351.666).abs() < 0.01);
    }

    #[test]
    fn full_screen_rect() {
        let s = screen();
        let r = grid().bounding_rect(
            Cell { col: 0, row: 0 },
            Cell { col: 3, row: 2 },
            s,
        );
        assert_eq!(r.x, s.x);
        assert_eq!(r.y, s.y);
        assert_eq!(r.width, s.width);
        assert_eq!(r.height, s.height);
    }

    #[test]
    fn bounding_rect_order_independent() {
        let s = screen();
        let a = Cell { col: 0, row: 0 };
        let b = Cell { col: 3, row: 1 };
        let r1 = grid().bounding_rect(a, b, s);
        let r2 = grid().bounding_rect(b, a, s);
        assert_eq!(r1.x, r2.x);
        assert_eq!(r1.y, r2.y);
        assert_eq!(r1.width, r2.width);
        assert_eq!(r1.height, r2.height);
    }

    #[test]
    fn top_row_full_width() {
        let s = screen();
        let r = grid().bounding_rect(
            Cell { col: 0, row: 0 },
            Cell { col: 3, row: 0 },
            s,
        );
        assert_eq!(r.x, s.x);
        assert_eq!(r.y, s.y);
        assert_eq!(r.width, s.width);
        assert!((r.height - 351.666).abs() < 0.01);
    }

    #[test]
    fn left_column_full_height() {
        let s = screen();
        let r = grid().bounding_rect(
            Cell { col: 0, row: 0 },
            Cell { col: 0, row: 2 },
            s,
        );
        assert_eq!(r.x, s.x);
        assert_eq!(r.y, s.y);
        assert_eq!(r.width, 480.0);
        assert_eq!(r.height, s.height);
    }

    #[test]
    fn first_key_enters_first_selected() {
        let g = grid();
        let mut state = SelectionState::new();
        let now = Instant::now();
        let action = state.advance(12, &g, now);
        assert_eq!(action, SelectionAction::FirstSelected(Cell { col: 0, row: 0 }));
    }

    #[test]
    fn two_keys_within_timeout_produces_tile() {
        let g = grid();
        let mut state = SelectionState::new();
        let now = Instant::now();
        state.advance(12, &g, now);
        let action = state.advance(3, &g, now + std::time::Duration::from_millis(500));
        assert_eq!(
            action,
            SelectionAction::Tile(Cell { col: 0, row: 0 }, Cell { col: 3, row: 1 })
        );
    }

    #[test]
    fn same_key_twice_produces_single_cell_tile() {
        let g = grid();
        let mut state = SelectionState::new();
        let now = Instant::now();
        state.advance(12, &g, now);
        let action = state.advance(12, &g, now + std::time::Duration::from_millis(100));
        assert_eq!(
            action,
            SelectionAction::Tile(Cell { col: 0, row: 0 }, Cell { col: 0, row: 0 })
        );
    }

    #[test]
    fn timeout_resets_to_new_first_selection() {
        let g = grid();
        let mut state = SelectionState::new();
        let now = Instant::now();
        state.advance(12, &g, now);
        let action = state.advance(3, &g, now + std::time::Duration::from_millis(1001));
        assert_eq!(action, SelectionAction::FirstSelected(Cell { col: 3, row: 1 }));
    }

    #[test]
    fn timeout_at_exact_boundary_resets() {
        let g = grid();
        let mut state = SelectionState::new();
        let now = Instant::now();
        state.advance(12, &g, now);
        let action = state.advance(3, &g, now + std::time::Duration::from_secs(1));
        assert_eq!(action, SelectionAction::FirstSelected(Cell { col: 3, row: 1 }));
    }

    #[test]
    fn invalid_key_is_ignored() {
        let g = grid();
        let mut state = SelectionState::new();
        let action = state.advance(5, &g, Instant::now());
        assert_eq!(action, SelectionAction::Ignored);
    }

    #[test]
    fn invalid_key_does_not_affect_first_selection() {
        let g = grid();
        let mut state = SelectionState::new();
        let now = Instant::now();
        state.advance(12, &g, now);
        state.advance(5, &g, now + std::time::Duration::from_millis(200)); // garbage key, ignored
        let action = state.advance(3, &g, now + std::time::Duration::from_millis(400));
        assert_eq!(
            action,
            SelectionAction::Tile(Cell { col: 0, row: 0 }, Cell { col: 3, row: 1 })
        );
    }

    #[test]
    fn check_timeout_clears_state() {
        let g = Grid::default_4x3();
        let mut state = SelectionState::new();
        let now = Instant::now();
        state.advance(12, &g, now);
        assert!(!state.check_timeout(now + std::time::Duration::from_millis(500), &g));
        assert!(state.check_timeout(now + std::time::Duration::from_millis(1001), &g));
    }

    #[test]
    fn reset_clears_state() {
        let mut state = SelectionState::new();
        let now = Instant::now();
        let g = Grid::default_4x3();
        state.advance(12, &g, now);
        state.reset();
        let action = state.advance(3, &g, now + std::time::Duration::from_millis(100));
        assert_eq!(action, SelectionAction::FirstSelected(Cell { col: 3, row: 1 }));
    }
}
