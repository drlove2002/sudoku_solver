use crate::types::{Board, masks::Masks};
use std::collections::VecDeque;

/// Constraint propagation using incremental bitmask counters.
///
/// Simultaneously detects naked singles and hidden singles in a single
/// queue-driven pass. Only O(affected) work per placement.
///
/// === DATA STRUCTURES ===
///
/// `allowed[r][c]`: u32 bitmask. Bit (d-1) = 1 → digit d is still legal.
///   Example: N=9, conflict at (2,3) blocks {1,3,5} (bits 0,2,4 set).
///   all_mask = 0b111111111, !conflict.raw() = 0b110101011
///   → allowed[2][3] = 0b110101011 (digits 2,4,6,7,8,9 legal)
///
/// `row_count[r][d_idx]`: how many EMPTY cells in row r still allow digit d.
///   d_idx = d - 1, so row_count[2][4] = 3 means digit 5 appears in
///   3 cells of row 2.
///   When this drops to 1, the one remaining cell is a hidden single.
///
/// `col_count[c][d_idx]`: same for column c.
/// `box_count[b][d_idx]`: same for box b.
///
/// `row_pos[r][d_idx]`: bitmask of columns in row r where d is still allowed.
///   When row_count[r][d_idx] == 1, trailing_zeros(row_pos[r][d_idx]) gives
///   the column directly — no scanning needed.
///
/// `col_pos[c][d_idx]`: bitmask of rows in column c where d is still allowed.
/// `box_pos[b][d_idx]`: bitmask of box-local positions where d is still allowed.
///   Box-local index i in [0..N) maps to: r = base_r + i/K, c = base_c + i%K.
///
/// Queue: pending forced placements as (row, col, digit). Push order:
///   When a counter hits 1 → hidden single.
///   When popcount(allowed) hits 1 → naked single.
///
/// === ALGORITHM (single pass, no loops) ===
///
/// 1. INIT: for each empty cell, compute allowed from conflict mask.
///    Seed counters and position masks. Push any initial singles.
///
/// 2. PROPAGATE (drain queue):
///    a) Deactivate: for each digit od in old allowed, call
///       remove_digit(r,c,od) to take the cell out of all counters.
///       Any counter hitting 1 → push hidden single.
///    b) Place: board.cells[r][c] = d.
///    c) Propagate d: for each peer cell (same row, col, box),
///       call remove_digit(peer, d). This clears d from the peer's
///       allowed mask, decrements counters, and pushes any new singles.
///
/// 3. CLEANUP: rebuild masks from the final board via masks.generate(board).
///
/// === COMPLEXITY (N=25, worst case) ===
///
/// Space: ~18 KB (allowed: 2.5 KB, counters: 6 × 1.3 KB, positions: 6 × 2.5 KB,
///        queue: ~4 KB). Everything on stack + small VecDeque allocation.
/// Time: ~200K-635K ops total. Each placement touches up to 64 peers,
///        but typical peers won't have the bit set → ~5-15 inner ops per placement.
#[allow(clippy::doc_overindented_list_items)]
pub fn propagate_constraints<const N: usize, const K: usize>(
    board: &mut Board<N>,
    masks: &mut Masks<N>,
) -> usize {
    assert!(
        N <= 32,
        "Constraint propagation requires N <= 32 (u32 bitmask limit)"
    );
    assert_eq!(K * K, N, "K must be the square root of N");

    let mut total_filled = 0usize;
    let all_mask: u32 = (1u32 << N) - 1; // e.g. N=9: 0b111111111

    // --- Per-cell legal-digit bitmask ---
    let mut allowed = [[0u32; N]; N];

    // --- Per-house per-digit counters and position masks ---
    // Digit d is stored at index d_idx = d - 1.
    let mut row_count = [[0u16; N]; N];
    let mut col_count = [[0u16; N]; N];
    let mut box_count = [[0u16; N]; N];
    let mut row_pos = [[0u32; N]; N];
    let mut col_pos = [[0u32; N]; N];
    let mut box_pos = [[0u32; N]; N];

    // --- Queue of forced placements: (row, col, digit) ---
    let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::with_capacity(N * N);

    // ============================================================
    // HELPER: remove digit d from cell (r,c), update counters,
    //         and push any newly-revealed singles onto the queue.
    //
    // d: digit in 1..=N. d_idx = d - 1.
    // ============================================================
    #[allow(clippy::too_many_arguments)]
    fn remove_digit<const N: usize, const K: usize>(
        r: usize,
        c: usize,
        d: usize,
        board: &Board<N>,
        allowed: &mut [[u32; N]; N],
        row_count: &mut [[u16; N]; N],
        col_count: &mut [[u16; N]; N],
        box_count: &mut [[u16; N]; N],
        row_pos: &mut [[u32; N]; N],
        col_pos: &mut [[u32; N]; N],
        box_pos: &mut [[u32; N]; N],
        queue: &mut VecDeque<(usize, usize, usize)>,
    ) {
        // Already-filled cell or digit already removed — nothing to do
        if board.cells[r][c] != 0 {
            return;
        }
        let d_idx = d - 1;
        let d_mask = 1u32 << d_idx; // e.g. d=5 → bit 4 → 0b000010000
        if allowed[r][c] & d_mask == 0 {
            return;
        }

        // --- Example: removing digit 5 from cell (2,3) in box 1 ---
        // Before: allowed[2][3] = 0b001011100  (digits 3,4,5,7 legal: bits 2,3,4,6 set)
        // Clear bit 4                    ↓
        // After:  allowed[2][3] = 0b001001100  (digits 3,4,7 legal)
        allowed[r][c] &= !d_mask;

        let b = Board::<N>::box_idx(r, c);
        let box_local = (r % K) * K + (c % K);

        // --- Row: digit d no longer possible at column c of row r ---
        row_count[r][d_idx] -= 1;
        row_pos[r][d_idx] &= !(1u32 << c);
        if row_count[r][d_idx] == 1 {
            let col = row_pos[r][d_idx].trailing_zeros() as usize;
            queue.push_back((r, col, d));
        }

        // --- Column ---
        col_count[c][d_idx] -= 1;
        col_pos[c][d_idx] &= !(1u32 << r);
        if col_count[c][d_idx] == 1 {
            let row = col_pos[c][d_idx].trailing_zeros() as usize;
            queue.push_back((row, c, d));
        }

        // --- Box ---
        box_count[b][d_idx] -= 1;
        box_pos[b][d_idx] &= !(1u32 << box_local);
        if box_count[b][d_idx] == 1 {
            let local = box_pos[b][d_idx].trailing_zeros() as usize;
            let base_r = (b / K) * K;
            let base_c = (b % K) * K;
            let nr = base_r + local / K;
            let nc = base_c + local % K;
            queue.push_back((nr, nc, d));
        }

        // --- Naked single: cell now has exactly one legal digit left ---
        if allowed[r][c].count_ones() == 1 {
            let nd = allowed[r][c].trailing_zeros() as usize + 1;
            queue.push_back((r, c, nd));
        }
    }

    // ============================================================
    // PHASE 1: initialize allowed masks and counters from masks.conflict
    // ============================================================
    for r in 0..N {
        for c in 0..N {
            if board.cells[r][c] != 0 {
                continue; // given or already placed
            }

            let a = all_mask & !masks.conflict[r][c].raw();
            allowed[r][c] = a;

            let b = Board::<N>::box_idx(r, c);
            let box_local = (r % K) * K + (c % K);

            // Walk set bits; for each, seed the three house counters
            let mut bits = a;
            while bits != 0 {
                let d_idx = bits.trailing_zeros() as usize;
                bits &= bits - 1; // Kernighan: clear lowest set bit

                row_count[r][d_idx] += 1;
                row_pos[r][d_idx] |= 1u32 << c;
                col_count[c][d_idx] += 1;
                col_pos[c][d_idx] |= 1u32 << r;
                box_count[b][d_idx] += 1;
                box_pos[b][d_idx] |= 1u32 << box_local;
            }

            // Initial naked singles
            if a.count_ones() == 1 {
                let d = a.trailing_zeros() as usize + 1;
                queue.push_back((r, c, d));
            }
        }
    }

    // ============================================================
    // PHASE 2: seed initial hidden singles from counters
    // ============================================================
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

    // ============================================================
    // PHASE 3: drain queue
    // ============================================================
    while let Some((r, c, d)) = queue.pop_front() {
        // Skip duplicate entries or placements invalidated by earlier propagation.
        // A hidden single pushed in step (a) may have its digit removed from the
        // target cell in step (c) before the queue entry is processed.
        if board.cells[r][c] != 0 {
            continue;
        }
        if allowed[r][c] & (1u32 << (d - 1)) == 0 {
            continue;
        }

        // (a) Deactivate: remove ALL digits this cell could have held
        let saved_allowed = allowed[r][c];
        let mut bits = saved_allowed;
        while bits != 0 {
            let d_idx = bits.trailing_zeros() as usize;
            let od = d_idx + 1;
            bits &= bits - 1;
            remove_digit::<N, K>(
                r, c, od, board, &mut allowed, &mut row_count, &mut col_count,
                &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos,
                &mut queue,
            );
        }

        // (b) Place the digit and update row/col/box masks incrementally
        board.cells[r][c] = d as u8;
        let b = Board::<N>::box_idx(r, c);
        masks.rows[r].dirty_set(d);
        masks.cols[c].dirty_set(d);
        masks.boxs[b].dirty_set(d);
        total_filled += 1;

        // (c) Propagate: remove digit d from every peer cell
        //
        // Row peers
        for pc in 0..N {
            if pc == c {
                continue;
            }
            remove_digit::<N, K>(
                r, pc, d, board, &mut allowed, &mut row_count, &mut col_count,
                &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos,
                &mut queue,
            );
        }

        // Column peers
        for pr in 0..N {
            if pr == r {
                continue;
            }
            remove_digit::<N, K>(
                pr, c, d, board, &mut allowed, &mut row_count, &mut col_count,
                &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos,
                &mut queue,
            );
        }

        // Box peers
        let base_r = (b / K) * K;
        let base_c = (b % K) * K;
        for bi in 0..K {
            for bj in 0..K {
                let pr = base_r + bi;
                let pc = base_c + bj;
                if pr == r && pc == c {
                    continue;
                }
                remove_digit::<N, K>(
                    pr, pc, d, board, &mut allowed, &mut row_count, &mut col_count,
                    &mut box_count, &mut row_pos, &mut col_pos, &mut box_pos,
                    &mut queue,
                );
            }
        }
    }

    // ============================================================
    // PHASE 4: rebuild conflict masks for all cells from the
    // updated row/col/box masks (which we kept in sync above).
    // ============================================================
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
