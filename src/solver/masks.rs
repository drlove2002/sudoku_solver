use log::{debug, trace};

pub trait MaskGenerator<const N: usize> {
    fn generate_masks(&self) -> [[u32; N]; N];
}

impl<const N: usize> MaskGenerator<N> for super::SudokuSolver<N> {
    fn generate_masks(&self) -> [[u32; N]; N] {
        debug!("Board size: {}x{}, Box size: {}x{}", N, N, Self::K, Self::K);

        // Optimized approach: O(N^2)
        // 1. Calculate used masks for rows, cols, boxes
        let mut row_used = vec![0u32; N];
        let mut col_used = vec![0u32; N];
        let mut box_used = vec![0u32; N];

        for (r, row) in self.board.cells.iter().enumerate() {
            for (c, &val) in row.iter().enumerate() {
                if val != 0 {
                    let b = (r / Self::K) * Self::K + (c / Self::K);
                    let mask = 1 << (val - 1);
                    trace!("Cell ({}, {}): val={}, mask={:032b}", r, c, val, mask);

                    if (row_used[r] & mask) != 0
                        || (col_used[c] & mask) != 0
                        || (box_used[b] & mask) != 0
                    {
                        trace!(
                            "Conflict detected! row_used={:032b}, col_used={:032b}, box_used={:032b}",
                            row_used[r], col_used[c], box_used[b]
                        );
                        panic!("Invalid board: duplicate value found");
                    }

                    row_used[r] |= mask;
                    col_used[c] |= mask;
                    box_used[b] |= mask;
                    trace!(
                        "Updated masks: row_used[{}]={:032b}, col_used[{}]={:032b}, box_used[{}]={:032b}",
                        r, row_used[r], c, col_used[c], b, box_used[b]
                    );
                }
            }
        }

        // 2. Calculate allowed masks for each cell
        let mut conflict_masks = [[0u32; N]; N];

        for (r, row) in self.board.cells.iter().enumerate() {
            for (c, &val) in row.iter().enumerate() {
                if val == 0 {
                    let b = (r / Self::K) * Self::K + (c / Self::K);
                    let used = row_used[r] | col_used[c] | box_used[b];
                    trace!(
                        "Cell ({}, {}): Empty. Combined used mask={:032b} (row={:032b}, col={:032b}, box={:032b})",
                        r, c, used, row_used[r], col_used[c], box_used[b]
                    );
                    conflict_masks[r][c] = used;
                } else {
                    // Set all bits to 1 for known values (to exclude them from permutations)
                    conflict_masks[r][c] = !0;
                    trace!(
                        "Cell ({}, {}): Pre-filled. Conflict mask set to all ones.",
                        r, c
                    );
                }
            }
        }
        conflict_masks
    }
}

#[allow(dead_code)]
pub trait MaskGeneratorV2<const N: usize> {
    fn generate_masks_v2(&self) -> [[u32; N]; N];
}

impl<const N: usize> MaskGeneratorV2<N> for super::SudokuSolver<N> {
    fn generate_masks_v2(&self) -> [[u32; N]; N] {
        // TODO: Implement v2 mask generation logic here
        // This is intended to be benchmarked against the original implementation
        todo!("Implement v2 mask generation")
    }
}
