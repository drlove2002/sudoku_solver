use crate::types::{masks::Masks, Board};

/// Applies the "Hidden Single" deduction rule iteratively until no more cells can be filled.
/// Returns the total number of cells filled.
pub fn apply_hidden_singles<const N: usize, const K: usize>(
    board: &mut Board<N>,
    masks: &mut Masks<N>,
) -> usize {
    let mut total_filled = 0;

    loop {
        let mut changed = false;

        for d in 1..=N {
            // 1. Check rows
            for r in 0..N {
                let mut possible = None;
                let mut count = 0;
                for c in 0..N {
                    if board.cells[r][c] == 0 && !masks.conflict[r][c].is_dirty(d) {
                        possible = Some((r, c));
                        count += 1;
                    }
                }
                if count == 1 {
                    let (pr, pc) = possible.unwrap();
                    if board.cells[pr][pc] == 0 {
                        board.cells[pr][pc] = d as u8;
                        update_masks::<N, K>(masks, pr, pc, d);
                        total_filled += 1;
                        changed = true;
                    }
                }
            }

            // 2. Check columns
            for c in 0..N {
                let mut possible = None;
                let mut count = 0;
                for r in 0..N {
                    if board.cells[r][c] == 0 && !masks.conflict[r][c].is_dirty(d) {
                        possible = Some((r, c));
                        count += 1;
                    }
                }
                if count == 1 {
                    let (pr, pc) = possible.unwrap();
                    if board.cells[pr][pc] == 0 {
                        board.cells[pr][pc] = d as u8;
                        update_masks::<N, K>(masks, pr, pc, d);
                        total_filled += 1;
                        changed = true;
                    }
                }
            }

            // 3. Check 3x3 minigrids (boxes)
            for b in 0..N {
                let mut possible = None;
                let mut count = 0;
                let base_r = (b / K) * K;
                let base_c = (b % K) * K;

                for i in 0..N {
                    let r = base_r + i / K;
                    let c = base_c + i % K;
                    if board.cells[r][c] == 0 && !masks.conflict[r][c].is_dirty(d) {
                        possible = Some((r, c));
                        count += 1;
                    }
                }
                if count == 1 {
                    let (pr, pc) = possible.unwrap();
                    if board.cells[pr][pc] == 0 {
                        board.cells[pr][pc] = d as u8;
                        update_masks::<N, K>(masks, pr, pc, d);
                        total_filled += 1;
                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    total_filled
}

/// Helper to update the `Masks` struct efficiently after a single digit is placed
fn update_masks<const N: usize, const K: usize>(masks: &mut Masks<N>, r: usize, c: usize, d: usize) {
    let b = Board::<N>::box_idx(r, c);
    masks.rows[r].dirty_set(d);
    masks.cols[c].dirty_set(d);
    masks.boxs[b].dirty_set(d);

    // Recompute conflict masks for the entire board
    for i in 0..N {
        for j in 0..N {
            let bx = Board::<N>::box_idx(i, j);
            masks.conflict[i][j] = masks.rows[i] | masks.cols[j] | masks.boxs[bx];
        }
    }
}
