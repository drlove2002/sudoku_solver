use log::debug;

pub trait MaskGenerator<const N: usize> {
    fn generate_masks(&self) -> Vec<Vec<u32>>;
}

impl<const N: usize> MaskGenerator<N> for super::SudokuSolver<N> {
    fn generate_masks(&self) -> Vec<Vec<u32>> {
        debug!("Board size: {}x{}, Box size: {}x{}", N, N, Self::K, Self::K);

        // Optimized approach: O(N^2)
        // 1. Calculate used masks for rows, cols, boxes
        let mut row_used = vec![0u32; N];
        let mut col_used = vec![0u32; N];
        let mut box_used = vec![0u32; N];

        for r in 0..N {
            for c in 0..N {
                let val = self.board.cells[r][c];
                if val != 0 {
                    let b = (r / Self::K) * Self::K + (c / Self::K);
                    let mask = 1 << val;

                    if (row_used[r] & mask) != 0
                        || (col_used[c] & mask) != 0
                        || (box_used[b] & mask) != 0
                    {
                        panic!("Invalid board: duplicate value found");
                    }

                    row_used[r] |= mask;
                    col_used[c] |= mask;
                    box_used[b] |= mask;
                }
            }
        }

        // 2. Calculate allowed masks for each cell
        let valid_digits_mask = (1 << (N + 1)) - 2;
        let mut conflict_masks = vec![vec![0u32; N]; N];

        for r in 0..N {
            for c in 0..N {
                if self.board.cells[r][c] == 0 {
                    let b = (r / Self::K) * Self::K + (c / Self::K);
                    let used = row_used[r] | col_used[c] | box_used[b];
                    conflict_masks[r][c] = (!used) & valid_digits_mask;
                }
            }
        }
        conflict_masks
    }
}
