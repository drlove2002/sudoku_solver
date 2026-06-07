mod remove_digit;
mod pairs;

use crate::types::{Board, masks::Masks};
use remove_digit::remove_digit;
use pairs::{
    hidden_pairs_boxes, hidden_pairs_cols, hidden_pairs_rows,
    naked_pairs_boxes, naked_pairs_cols, naked_pairs_rows,
};
use std::collections::VecDeque;

/// Constraint propagation using incremental bitmask counters.
///
/// Detects naked singles, hidden singles, naked pairs, and hidden pairs.
/// Runs in a loop: drain singles → scan pairs → repeat until quiescence.
///
/// === DATA STRUCTURES ===
///
/// `allowed[r][c]`: u32 bitmask. Bit (d-1) = 1 → digit d is still legal.
/// `row_count[r][d_idx]`: how many EMPTY cells in row r still allow digit d.
/// `col_count`, `box_count`: same for columns/boxes.
/// `row_pos[r][d_idx]`: bitmask of columns in row r where d is still allowed.
/// `col_pos[c][d_idx]`: bitmask of rows in column c where d is still allowed.
/// `box_pos[b][d_idx]`: bitmask of box-local positions where d is still allowed.
///
/// Queue: pending forced placements as (row, col, digit).
pub fn propagate_constraints<const N: usize, const K: usize>(
    board: &mut Board<N>,
    masks: &mut Masks<N>,
) -> usize {
    assert!(N <= 32, "Constraint propagation requires N <= 32 (u32 bitmask limit)");
    assert_eq!(K * K, N, "K must be the square root of N");

    let mut total_filled = 0usize;
    let all_mask: u32 = (1u32 << N) - 1;

    let mut allowed = [[0u32; N]; N];
    let mut row_count = [[0u16; N]; N];
    let mut col_count = [[0u16; N]; N];
    let mut box_count = [[0u16; N]; N];
    let mut row_pos = [[0u32; N]; N];
    let mut col_pos = [[0u32; N]; N];
    let mut box_pos = [[0u32; N]; N];
    let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::with_capacity(N * N);

    // Phase 1: initialize allowed masks and counters from conflict masks
    for r in 0..N {
        for c in 0..N {
            if board.cells[r][c] != 0 { continue; }
            let a = all_mask & !masks.conflict[r][c].raw();
            allowed[r][c] = a;
            let b = Board::<N>::box_idx(r, c);
            let bl = (r % K) * K + (c % K);
            let mut bits = a;
            while bits != 0 {
                let d_idx = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                row_count[r][d_idx] += 1;
                row_pos[r][d_idx] |= 1u32 << c;
                col_count[c][d_idx] += 1;
                col_pos[c][d_idx] |= 1u32 << r;
                box_count[b][d_idx] += 1;
                box_pos[b][d_idx] |= 1u32 << bl;
            }
            if a.count_ones() == 1 {
                let d = a.trailing_zeros() as usize + 1;
                queue.push_back((r, c, d));
            }
        }
    }

    // Phase 2: seed hidden singles
    for r in 0..N {
        for d_idx in 0..N {
            if row_count[r][d_idx] == 1 {
                let c = row_pos[r][d_idx].trailing_zeros() as usize;
                queue.push_back((r, c, d_idx + 1));
            }
        }
    }
    for c in 0..N {
        for d_idx in 0..N {
            if col_count[c][d_idx] == 1 {
                let r = col_pos[c][d_idx].trailing_zeros() as usize;
                queue.push_back((r, c, d_idx + 1));
            }
        }
    }
    for b in 0..N {
        for d_idx in 0..N {
            if box_count[b][d_idx] == 1 {
                let local = box_pos[b][d_idx].trailing_zeros() as usize;
                let r = (b / K) * K + local / K;
                let c = (b % K) * K + local % K;
                queue.push_back((r, c, d_idx + 1));
            }
        }
    }

    // Phase 3: propagate to quiescence
    loop {
        while let Some((r, c, d)) = queue.pop_front() {
            if board.cells[r][c] != 0 { continue; }
            if allowed[r][c] & (1u32 << (d - 1)) == 0 { continue; }

            let saved = allowed[r][c];
            let mut bits = saved;
            while bits != 0 {
                let d_idx = bits.trailing_zeros() as usize;
                let od = d_idx + 1;
                bits &= bits - 1;
                remove_digit::<N, K>(r, c, od, board, &mut allowed, &mut row_count, &mut col_count, &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos, &mut queue);
            }

            board.cells[r][c] = d as u8;
            let b = Board::<N>::box_idx(r, c);
            masks.rows[r].dirty_set(d);
            masks.cols[c].dirty_set(d);
            masks.boxs[b].dirty_set(d);
            total_filled += 1;

            for pc in 0..N {
                if pc == c { continue; }
                remove_digit::<N, K>(r, pc, d, board, &mut allowed, &mut row_count, &mut col_count, &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos, &mut queue);
            }
            for pr in 0..N {
                if pr == r { continue; }
                remove_digit::<N, K>(pr, c, d, board, &mut allowed, &mut row_count, &mut col_count, &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos, &mut queue);
            }
            let br = (b / K) * K;
            let bc = (b % K) * K;
            for bi in 0..K {
                for bj in 0..K {
                    let pr = br + bi;
                    let pc = bc + bj;
                    if pr == r && pc == c { continue; }
                    remove_digit::<N, K>(pr, pc, d, board, &mut allowed, &mut row_count, &mut col_count, &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos, &mut queue);
                }
            }
        }

        let mut pair_found = false;
        pair_found |= naked_pairs_rows::<N, K>(board, &mut allowed, &mut row_count, &mut col_count, &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos, &mut queue);
        pair_found |= naked_pairs_cols::<N, K>(board, &mut allowed, &mut row_count, &mut col_count, &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos, &mut queue);
        pair_found |= naked_pairs_boxes::<N, K>(board, &mut allowed, &mut row_count, &mut col_count, &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos, &mut queue);
        pair_found |= hidden_pairs_rows::<N, K>(board, &mut allowed, &mut row_count, &mut row_pos, &mut col_count, &mut box_count, &mut col_pos, &mut box_pos, &mut queue);
        pair_found |= hidden_pairs_cols::<N, K>(board, &mut allowed, &mut col_count, &mut col_pos, &mut row_count, &mut box_count, &mut row_pos, &mut box_pos, &mut queue);
        pair_found |= hidden_pairs_boxes::<N, K>(board, &mut allowed, &mut box_count, &mut box_pos, &mut row_count, &mut col_count, &mut row_pos, &mut col_pos, &mut queue);

        if !pair_found { break; }
    }

    // Phase 4: rebuild conflict masks
    if total_filled > 0 {
        for r in 0..N {
            for c in 0..N {
                let b = Board::<N>::box_idx(r, c);
                masks.conflict[r][c] = masks.rows[r] | masks.cols[c] | masks.boxs[b];
            }
        }
    }

    total_filled
}
