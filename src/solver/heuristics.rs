use crate::types::{Board, masks::Masks};
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
#[allow(clippy::doc_overindented_list_items)]
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

    // ============================================================
    // HELPER: remove digit d from cell (r,c).
    // Decrements house counters, updates position masks, and pushes
    // any newly-revealed naked/hidden singles onto the queue.
    // ============================================================
    #[allow(clippy::too_many_arguments)]
    fn remove_digit<const N: usize, const K: usize>(
        r: usize, c: usize, d: usize,
        board: &Board<N>,
        allowed: &mut [[u32; N]; N],
        row_count: &mut [[u16; N]; N], col_count: &mut [[u16; N]; N],
        box_count: &mut [[u16; N]; N],
        row_pos: &mut [[u32; N]; N], col_pos: &mut [[u32; N]; N],
        box_pos: &mut [[u32; N]; N],
        queue: &mut VecDeque<(usize, usize, usize)>,
    ) {
        if board.cells[r][c] != 0 { return; }
        let d_idx = d - 1;
        let d_mask = 1u32 << d_idx;
        if allowed[r][c] & d_mask == 0 { return; }

        allowed[r][c] &= !d_mask;
        let b = Board::<N>::box_idx(r, c);
        let box_local = (r % K) * K + (c % K);

        // Row
        row_count[r][d_idx] -= 1;
        row_pos[r][d_idx] &= !(1u32 << c);
        if row_count[r][d_idx] == 1 {
            let col = row_pos[r][d_idx].trailing_zeros() as usize;
            queue.push_back((r, col, d));
        }
        // Col
        col_count[c][d_idx] -= 1;
        col_pos[c][d_idx] &= !(1u32 << r);
        if col_count[c][d_idx] == 1 {
            let row = col_pos[c][d_idx].trailing_zeros() as usize;
            queue.push_back((row, c, d));
        }
        // Box
        box_count[b][d_idx] -= 1;
        box_pos[b][d_idx] &= !(1u32 << box_local);
        if box_count[b][d_idx] == 1 {
            let local = box_pos[b][d_idx].trailing_zeros() as usize;
            let br = (b / K) * K;
            let bc = (b % K) * K;
            queue.push_back((br + local / K, bc + local % K, d));
        }
        // Naked single
        if allowed[r][c].count_ones() == 1 {
            let nd = allowed[r][c].trailing_zeros() as usize + 1;
            queue.push_back((r, c, nd));
        }
    }

    // ============================================================
    // HELPERS: naked/hidden pair scanners.
    // Each calls remove_digit directly — counters stay in sync.
    // Returns true if any candidate was removed.
    // ============================================================
    #[inline(never)]
    fn naked_pairs_rows<const N: usize, const K: usize>(
        board: &Board<N>, allowed: &mut [[u32; N]; N],
        row_count: &mut [[u16; N]; N], col_count: &mut [[u16; N]; N],
        box_count: &mut [[u16; N]; N],
        row_pos: &mut [[u32; N]; N], col_pos: &mut [[u32; N]; N],
        box_pos: &mut [[u32; N]; N],
        queue: &mut VecDeque<(usize, usize, usize)>,
    ) -> bool {
        let mut found = false;
        for r in 0..N {
            for c1 in 0..N {
                if board.cells[r][c1] != 0 { continue; }
                let mask = allowed[r][c1];
                if mask.count_ones() != 2 { continue; }
                let d1 = mask.trailing_zeros() as usize + 1;
                let d2 = (mask ^ (1u32 << (d1 - 1))).trailing_zeros() as usize + 1;
                let c2 = (c1 + 1..N).find(|&c| board.cells[r][c] == 0 && allowed[r][c] == mask);
                let Some(c2) = c2 else { continue };
                for c in 0..N {
                    if c == c1 || c == c2 || board.cells[r][c] != 0 { continue; }
                    if allowed[r][c] & (1u32 << (d1 - 1)) != 0 {
                        remove_digit::<N, K>(r, c, d1, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                    if allowed[r][c] & (1u32 << (d2 - 1)) != 0 {
                        remove_digit::<N, K>(r, c, d2, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                }
            }
        }
        found
    }

    #[inline(never)]
    fn naked_pairs_cols<const N: usize, const K: usize>(
        board: &Board<N>, allowed: &mut [[u32; N]; N],
        row_count: &mut [[u16; N]; N], col_count: &mut [[u16; N]; N],
        box_count: &mut [[u16; N]; N],
        row_pos: &mut [[u32; N]; N], col_pos: &mut [[u32; N]; N],
        box_pos: &mut [[u32; N]; N],
        queue: &mut VecDeque<(usize, usize, usize)>,
    ) -> bool {
        let mut found = false;
        for c in 0..N {
            for r1 in 0..N {
                if board.cells[r1][c] != 0 { continue; }
                let mask = allowed[r1][c];
                if mask.count_ones() != 2 { continue; }
                let d1 = mask.trailing_zeros() as usize + 1;
                let d2 = (mask ^ (1u32 << (d1 - 1))).trailing_zeros() as usize + 1;
                let r2 = (r1 + 1..N).find(|&r| board.cells[r][c] == 0 && allowed[r][c] == mask);
                let Some(r2) = r2 else { continue };
                for r in 0..N {
                    if r == r1 || r == r2 || board.cells[r][c] != 0 { continue; }
                    if allowed[r][c] & (1u32 << (d1 - 1)) != 0 {
                        remove_digit::<N, K>(r, c, d1, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                    if allowed[r][c] & (1u32 << (d2 - 1)) != 0 {
                        remove_digit::<N, K>(r, c, d2, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                }
            }
        }
        found
    }

    #[inline(never)]
    fn naked_pairs_boxes<const N: usize, const K: usize>(
        board: &Board<N>, allowed: &mut [[u32; N]; N],
        row_count: &mut [[u16; N]; N], col_count: &mut [[u16; N]; N],
        box_count: &mut [[u16; N]; N],
        row_pos: &mut [[u32; N]; N], col_pos: &mut [[u32; N]; N],
        box_pos: &mut [[u32; N]; N],
        queue: &mut VecDeque<(usize, usize, usize)>,
    ) -> bool {
        let mut found = false;
        for b in 0..N {
            let base_r = (b / K) * K;
            let base_c = (b % K) * K;
            for i1 in 0..N {
                let r1 = base_r + i1 / K;
                let c1 = base_c + i1 % K;
                if board.cells[r1][c1] != 0 { continue; }
                let mask = allowed[r1][c1];
                if mask.count_ones() != 2 { continue; }
                let d1 = mask.trailing_zeros() as usize + 1;
                let d2 = (mask ^ (1u32 << (d1 - 1))).trailing_zeros() as usize + 1;
                let i2 = (i1 + 1..N).find(|&i| {
                    let r = base_r + i / K;
                    let c = base_c + i % K;
                    board.cells[r][c] == 0 && allowed[r][c] == mask
                });
                let Some(i2) = i2 else { continue };
                let _r2 = base_r + i2 / K;
                let _c2 = base_c + i2 % K;
                for i in 0..N {
                    if i == i1 || i == i2 { continue; }
                    let r = base_r + i / K;
                    let c = base_c + i % K;
                    if board.cells[r][c] != 0 { continue; }
                    if allowed[r][c] & (1u32 << (d1 - 1)) != 0 {
                        remove_digit::<N, K>(r, c, d1, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                    if allowed[r][c] & (1u32 << (d2 - 1)) != 0 {
                        remove_digit::<N, K>(r, c, d2, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                        found = true;
                    }
                }
            }
        }
        found
    }

    #[inline(never)]
    fn hidden_pairs_rows<const N: usize, const K: usize>(
        board: &Board<N>, allowed: &mut [[u32; N]; N],
        row_count: &mut [[u16; N]; N],
        row_pos: &mut [[u32; N]; N],
        col_count: &mut [[u16; N]; N], box_count: &mut [[u16; N]; N],
        col_pos: &mut [[u32; N]; N], box_pos: &mut [[u32; N]; N],
        queue: &mut VecDeque<(usize, usize, usize)>,
    ) -> bool {
        let mut found = false;
        for r in 0..N {
            for d1 in 1..=N {
                if row_count[r][d1 - 1] != 2 { continue; }
                for d2 in (d1 + 1)..=N {
                    if row_count[r][d2 - 1] != 2 { continue; }
                    if row_pos[r][d1 - 1] != row_pos[r][d2 - 1] { continue; }
                    let cols = row_pos[r][d1 - 1];
                    let c1 = cols.trailing_zeros() as usize;
                    let c2 = (cols ^ (1u32 << c1)).trailing_zeros() as usize;
                    for &c in &[c1, c2] {
                        let mut bits = allowed[r][c];
                        bits &= !(1u32 << (d1 - 1));
                        bits &= !(1u32 << (d2 - 1));
                        while bits != 0 {
                            let d = bits.trailing_zeros() as usize + 1;
                            bits &= bits - 1;
                            remove_digit::<N, K>(r, c, d, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                            found = true;
                        }
                    }
                }
            }
        }
        found
    }

    #[inline(never)]
    fn hidden_pairs_cols<const N: usize, const K: usize>(
        board: &Board<N>, allowed: &mut [[u32; N]; N],
        col_count: &mut [[u16; N]; N],
        col_pos: &mut [[u32; N]; N],
        row_count: &mut [[u16; N]; N], box_count: &mut [[u16; N]; N],
        row_pos: &mut [[u32; N]; N], box_pos: &mut [[u32; N]; N],
        queue: &mut VecDeque<(usize, usize, usize)>,
    ) -> bool {
        let mut found = false;
        for c in 0..N {
            for d1 in 1..=N {
                if col_count[c][d1 - 1] != 2 { continue; }
                for d2 in (d1 + 1)..=N {
                    if col_count[c][d2 - 1] != 2 { continue; }
                    if col_pos[c][d1 - 1] != col_pos[c][d2 - 1] { continue; }
                    let rows = col_pos[c][d1 - 1];
                    let r1 = rows.trailing_zeros() as usize;
                    let r2 = (rows ^ (1u32 << r1)).trailing_zeros() as usize;
                    for &r in &[r1, r2] {
                        let mut bits = allowed[r][c];
                        bits &= !(1u32 << (d1 - 1));
                        bits &= !(1u32 << (d2 - 1));
                        while bits != 0 {
                            let d = bits.trailing_zeros() as usize + 1;
                            bits &= bits - 1;
                            remove_digit::<N, K>(r, c, d, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                            found = true;
                        }
                    }
                }
            }
        }
        found
    }

    #[inline(never)]
    fn hidden_pairs_boxes<const N: usize, const K: usize>(
        board: &Board<N>, allowed: &mut [[u32; N]; N],
        box_count: &mut [[u16; N]; N],
        box_pos: &mut [[u32; N]; N],
        row_count: &mut [[u16; N]; N], col_count: &mut [[u16; N]; N],
        row_pos: &mut [[u32; N]; N], col_pos: &mut [[u32; N]; N],
        queue: &mut VecDeque<(usize, usize, usize)>,
    ) -> bool {
        let mut found = false;
        for b in 0..N {
            for d1 in 1..=N {
                if box_count[b][d1 - 1] != 2 { continue; }
                for d2 in (d1 + 1)..=N {
                    if box_count[b][d2 - 1] != 2 { continue; }
                    if box_pos[b][d1 - 1] != box_pos[b][d2 - 1] { continue; }
                    let locals = box_pos[b][d1 - 1];
                    let i1 = locals.trailing_zeros() as usize;
                    let i2 = (locals ^ (1u32 << i1)).trailing_zeros() as usize;
                    let base_r = (b / K) * K;
                    let base_c = (b % K) * K;
                    for &i in &[i1, i2] {
                        let r = base_r + i / K;
                        let c = base_c + i % K;
                        let mut bits = allowed[r][c];
                        bits &= !(1u32 << (d1 - 1));
                        bits &= !(1u32 << (d2 - 1));
                        while bits != 0 {
                            let d = bits.trailing_zeros() as usize + 1;
                            bits &= bits - 1;
                            remove_digit::<N, K>(r, c, d, board, allowed, row_count, col_count, box_count, row_pos, col_pos, box_pos, queue);
                            found = true;
                        }
                    }
                }
            }
        }
        found
    }

    // ============================================================
    // PHASE 1: initialize allowed masks and counters
    // ============================================================
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

    // ============================================================
    // PHASE 2: seed hidden singles
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
    // PHASE 3: propagate to quiescence
    // Drain queue → scan pairs → repeat if pairs found anything
    // ============================================================
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

    // ============================================================
    // PHASE 4: rebuild conflict masks
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
