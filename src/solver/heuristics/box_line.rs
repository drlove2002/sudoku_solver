use super::remove_digit::remove_digit;
use crate::types::Board;
use std::collections::VecDeque;

/// Pointing pair (box → row/col elimination):
/// If all candidates for digit d in a box lie within a single row of
/// that box, d can be eliminated from the rest of that row outside
/// the box. Same for columns.
///
/// Example: digit 7 in box 3 only appears in the middle row of the
/// box. Those cells span columns 6-8. Eliminate 7 from columns 0-5
/// and 9-N in that row.
///
/// Uses the box_pos bitmask: if all local positions share the same
/// row index (i/K), the pointing detection succeeds.
#[inline(never)]
pub(crate) fn pointing_pairs<const N: usize, const K: usize>(
    board: &Board<N>,
    allowed: &mut [[u32; N]; N],
    row_count: &mut [[u16; N]; N],
    col_count: &mut [[u16; N]; N],
    box_count: &mut [[u16; N]; N],
    row_pos: &mut [[u32; N]; N],
    col_pos: &mut [[u32; N]; N],
    box_pos: &mut [[u32; N]; N],
    queue: &mut VecDeque<(usize, usize, usize)>,
) -> bool {
    let mut found = false;

    for b in 0..N {
        let base_r = (b / K) * K;
        let base_c = (b % K) * K;

        for d in 1..=N {
            let d_mask = 1u32 << (d - 1);
            let positions = box_pos[b][d - 1];
            if positions == 0 { continue; }

            // Check if all candidates are in the same local row
            let mut local_rows = 0u32;
            let mut bits = positions;
            while bits != 0 {
                let i = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                local_rows |= 1u32 << (i / K);
            }
            if local_rows.count_ones() == 1 {
                // Pointing: d only appears in one row-section of this box
                let local_row = local_rows.trailing_zeros() as usize;
                let global_row = base_r + local_row;
                let box_c_start = base_c;
                let box_c_end = base_c + K;
                for c in 0..N {
                    if c >= box_c_start && c < box_c_end { continue; }
                    if board.cells[global_row][c] != 0 { continue; }
                    if allowed[global_row][c] & d_mask != 0 {
                        remove_digit::<N, K>(global_row, c, d, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                }
            }

            // Check if all candidates are in the same local column
            let mut local_cols = 0u32;
            let mut bits = positions;
            while bits != 0 {
                let i = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                local_cols |= 1u32 << (i % K);
            }
            if local_cols.count_ones() == 1 {
                let local_col = local_cols.trailing_zeros() as usize;
                let global_col = base_c + local_col;
                let box_r_start = base_r;
                let box_r_end = base_r + K;
                for r in 0..N {
                    if r >= box_r_start && r < box_r_end { continue; }
                    if board.cells[r][global_col] != 0 { continue; }
                    if allowed[r][global_col] & d_mask != 0 {
                        remove_digit::<N, K>(r, global_col, d, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                }
            }
        }
    }

    found
}

/// Claiming pair (row/col → box elimination):
/// If all candidates for digit d in a row lie within a single box,
/// d can be eliminated from the rest of that box.
/// Same for columns.
#[inline(never)]
pub(crate) fn claiming_pairs<const N: usize, const K: usize>(
    board: &Board<N>,
    allowed: &mut [[u32; N]; N],
    row_count: &mut [[u16; N]; N],
    col_count: &mut [[u16; N]; N],
    box_count: &mut [[u16; N]; N],
    row_pos: &mut [[u32; N]; N],
    col_pos: &mut [[u32; N]; N],
    box_pos: &mut [[u32; N]; N],
    queue: &mut VecDeque<(usize, usize, usize)>,
) -> bool {
    let mut found = false;

    // Row → box
    for r in 0..N {
        let box_row = r / K;
        for d in 1..=N {
            let d_mask = 1u32 << (d - 1);
            let cols = row_pos[r][d - 1];
            if cols == 0 { continue; }
            // Check if all columns are in the same box-column group
            let mut box_cols = 0u32;
            let mut bits = cols;
            while bits != 0 {
                let c = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                box_cols |= 1u32 << (c / K);
            }
            if box_cols.count_ones() == 1 {
                let bc = box_cols.trailing_zeros() as usize;
                let _b = box_row * K + bc;
                let base_r = box_row * K;
                let base_c = bc * K;
                for i in 0..N {
                    let rr = base_r + i / K;
                    let cc = base_c + i % K;
                    if rr == r { continue; } // skip the row itself
                    if board.cells[rr][cc] != 0 { continue; }
                    if allowed[rr][cc] & d_mask != 0 {
                        remove_digit::<N, K>(rr, cc, d, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                }
            }
        }
    }

    // Col → box
    for c in 0..N {
        let box_col = c / K;
        for d in 1..=N {
            let d_mask = 1u32 << (d - 1);
            let rows = col_pos[c][d - 1];
            if rows == 0 { continue; }
            let mut box_rows = 0u32;
            let mut bits = rows;
            while bits != 0 {
                let r = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                box_rows |= 1u32 << (r / K);
            }
            if box_rows.count_ones() == 1 {
                let br = box_rows.trailing_zeros() as usize;
                let _b = br * K + box_col;
                let base_r = br * K;
                let base_c = box_col * K;
                for i in 0..N {
                    let rr = base_r + i / K;
                    let cc = base_c + i % K;
                    if cc == c { continue; }
                    if board.cells[rr][cc] != 0 { continue; }
                    if allowed[rr][cc] & d_mask != 0 {
                        remove_digit::<N, K>(rr, cc, d, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                }
            }
        }
    }

    found
}
