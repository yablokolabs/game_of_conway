use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Grid {
    cells: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridError {
    Empty,
    InvalidSize(usize),
    NotSquare {
        row_index: usize,
        expected: usize,
        got: usize,
    },
    InvalidCellValue {
        row: usize,
        col: usize,
        value: u8,
    },
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "grid cannot be empty"),
            Self::InvalidSize(s) => {
                write!(f, "grid size must be between 3 and 1000, got {s}")
            }
            Self::NotSquare {
                row_index,
                expected,
                got,
            } => write!(
                f,
                "grid must be square: row {row_index} has {got} columns, expected {expected}"
            ),
            Self::InvalidCellValue { row, col, value } => {
                write!(
                    f,
                    "cell values must be 0 or 1, got {value} at ({row}, {col})"
                )
            }
        }
    }
}

impl std::error::Error for GridError {}

impl Grid {
    pub const MIN_SIZE: usize = 3;
    pub const MAX_SIZE: usize = 1000;

    pub fn new(cells: Vec<Vec<u8>>) -> Result<Self, GridError> {
        if cells.is_empty() {
            return Err(GridError::Empty);
        }

        let size = cells.len();
        if size < Self::MIN_SIZE || size > Self::MAX_SIZE {
            return Err(GridError::InvalidSize(size));
        }

        for (row_idx, row) in cells.iter().enumerate() {
            if row.len() != size {
                return Err(GridError::NotSquare {
                    row_index: row_idx,
                    expected: size,
                    got: row.len(),
                });
            }
            for (col_idx, &cell) in row.iter().enumerate() {
                if cell > 1 {
                    return Err(GridError::InvalidCellValue {
                        row: row_idx,
                        col: col_idx,
                        value: cell,
                    });
                }
            }
        }

        Ok(Self { cells })
    }

    pub fn size(&self) -> usize {
        self.cells.len()
    }

    pub fn cells(&self) -> &[Vec<u8>] {
        &self.cells
    }

    pub fn into_cells(self) -> Vec<Vec<u8>> {
        self.cells
    }

    pub fn next_state(&self) -> Self {
        let size = self.size();
        let mut next_cells = vec![vec![0u8; size]; size];

        for row in 0..size {
            for col in 0..size {
                let neighbors = self.count_neighbors(row, col);
                let alive = self.cells[row][col] == 1;
                next_cells[row][col] = match (alive, neighbors) {
                    (true, 2 | 3) => 1,
                    (false, 3) => 1,
                    _ => 0,
                };
            }
        }

        Self { cells: next_cells }
    }

    fn count_neighbors(&self, row: usize, col: usize) -> u8 {
        let size = self.size() as i32;
        let mut count = 0u8;

        for dr in [-1i32, 0, 1] {
            for dc in [-1i32, 0, 1] {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let nr = row as i32 + dr;
                let nc = col as i32 + dc;
                if nr >= 0 && nr < size && nc >= 0 && nc < size {
                    count += self.cells[nr as usize][nc as usize];
                }
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(cells: Vec<Vec<u8>>) -> Grid {
        Grid::new(cells).expect("test grid should be valid")
    }

    // --- Conway's Rules ---

    #[test]
    fn blinker_oscillates() {
        let vertical = grid(vec![vec![0, 1, 0], vec![0, 1, 0], vec![0, 1, 0]]);
        let horizontal = grid(vec![vec![0, 0, 0], vec![1, 1, 1], vec![0, 0, 0]]);
        assert_eq!(vertical.next_state(), horizontal);
        assert_eq!(horizontal.next_state(), vertical);
    }

    #[test]
    fn block_is_still_life() {
        let block = grid(vec![
            vec![0, 0, 0, 0],
            vec![0, 1, 1, 0],
            vec![0, 1, 1, 0],
            vec![0, 0, 0, 0],
        ]);
        assert_eq!(block.next_state(), block);
    }

    #[test]
    fn empty_grid_stays_empty() {
        let empty = grid(vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]]);
        assert_eq!(empty.next_state(), empty);
    }

    #[test]
    fn dead_cell_with_three_neighbors_is_born() {
        let before = grid(vec![vec![1, 1, 0], vec![1, 0, 0], vec![0, 0, 0]]);
        let after = before.next_state();
        assert_eq!(after.cells()[1][1], 1, "center cell should be born");
    }

    #[test]
    fn alive_cell_with_two_neighbors_survives() {
        // Cell (0,0) has neighbors (0,1)=1 and (1,0)=1 → 2 → survives
        let before = grid(vec![vec![1, 1, 0], vec![1, 0, 0], vec![0, 0, 0]]);
        let after = before.next_state();
        assert_eq!(after.cells()[0][0], 1, "cell with 2 neighbors survives");
    }

    #[test]
    fn alive_cell_with_three_neighbors_survives() {
        let before = grid(vec![vec![1, 1, 0], vec![1, 1, 0], vec![0, 0, 0]]);
        let after = before.next_state();
        // Each cell in the 2x2 block has exactly 3 neighbors → all survive
        assert_eq!(after.cells()[0][0], 1);
        assert_eq!(after.cells()[0][1], 1);
        assert_eq!(after.cells()[1][0], 1);
        assert_eq!(after.cells()[1][1], 1);
    }

    #[test]
    fn alive_cell_with_zero_neighbors_dies() {
        let before = grid(vec![vec![1, 0, 0], vec![0, 0, 0], vec![0, 0, 0]]);
        let after = before.next_state();
        assert_eq!(after.cells()[0][0], 0, "lonely cell dies");
    }

    #[test]
    fn alive_cell_with_four_neighbors_dies() {
        // Cell (0,1): neighbors (0,0)=1, (0,2)=1, (1,0)=1, (1,1)=1 → 4 → dies
        let before = grid(vec![vec![1, 1, 1], vec![1, 1, 0], vec![0, 0, 0]]);
        let after = before.next_state();
        assert_eq!(after.cells()[0][1], 0, "overpopulated cell dies");
    }

    #[test]
    fn full_grid_applies_overpopulation() {
        let all_alive = grid(vec![vec![1, 1, 1], vec![1, 1, 1], vec![1, 1, 1]]);
        let after = all_alive.next_state();
        // Corners have 3 neighbors → survive; edges have 5 → die; center has 8 → dies
        let expected = grid(vec![vec![1, 0, 1], vec![0, 0, 0], vec![1, 0, 1]]);
        assert_eq!(after, expected);
    }

    #[test]
    fn glider_moves_diagonally_after_four_generations() {
        let gen0 = grid(vec![
            vec![0, 1, 0, 0, 0, 0],
            vec![0, 0, 1, 0, 0, 0],
            vec![1, 1, 1, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0],
        ]);
        let gen4 = gen0.next_state().next_state().next_state().next_state();
        let expected = grid(vec![
            vec![0, 0, 0, 0, 0, 0],
            vec![0, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0],
            vec![0, 1, 1, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0],
        ]);
        assert_eq!(gen4, expected);
    }

    #[test]
    fn next_state_is_deterministic() {
        let g = grid(vec![vec![0, 1, 0], vec![0, 1, 0], vec![0, 1, 0]]);
        assert_eq!(g.next_state(), g.next_state());
    }

    // --- Grid Validation ---

    #[test]
    fn rejects_empty_grid() {
        assert!(matches!(Grid::new(vec![]), Err(GridError::Empty)));
    }

    #[test]
    fn rejects_grid_smaller_than_minimum() {
        let result = Grid::new(vec![vec![0, 0], vec![0, 0]]);
        assert!(matches!(result, Err(GridError::InvalidSize(2))));
    }

    #[test]
    fn rejects_grid_larger_than_maximum() {
        let cells = vec![vec![0u8; 1001]; 1001];
        let result = Grid::new(cells);
        assert!(matches!(result, Err(GridError::InvalidSize(1001))));
    }

    #[test]
    fn rejects_non_square_grid() {
        let result = Grid::new(vec![vec![0, 1, 0, 0], vec![0, 1, 0], vec![0, 1, 0]]);
        assert!(matches!(result, Err(GridError::NotSquare { .. })));
    }

    #[test]
    fn rejects_invalid_cell_values() {
        let result = Grid::new(vec![vec![0, 1, 0], vec![0, 2, 0], vec![0, 1, 0]]);
        assert!(matches!(
            result,
            Err(GridError::InvalidCellValue { value: 2, .. })
        ));
    }

    #[test]
    fn accepts_minimum_size_grid() {
        assert!(Grid::new(vec![vec![0; 3]; 3]).is_ok());
    }

    #[test]
    fn accepts_maximum_size_grid() {
        assert!(Grid::new(vec![vec![0; 1000]; 1000]).is_ok());
    }
}
