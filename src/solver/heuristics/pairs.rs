use super::remove_digit::remove_digit;
use crate::types::Board;
use std::collections::VecDeque;

/// Naked pair in rows.
#[inline(never)]
pub(crate) fn naked_pairs_rows<const N: usize, const K: usize>(
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

/// Naked pair in columns.
#[inline(never)]
pub(crate) fn naked_pairs_cols<const N: usize, const K: usize>(
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

/// Naked pair in boxes.
#[inline(never)]
pub(crate) fn naked_pairs_boxes<const N: usize, const K: usize>(
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

/// Hidden pair in rows.
#[inline(never)]
pub(crate) fn hidden_pairs_rows<const N: usize, const K: usize>(
    board: &Board<N>,
    allowed: &mut [[u32; N]; N],
    row_count: &mut [[u16; N]; N],
    row_pos: &mut [[u32; N]; N],
    col_count: &mut [[u16; N]; N],
    box_count: &mut [[u16; N]; N],
    col_pos: &mut [[u32; N]; N],
    box_pos: &mut [[u32; N]; N],
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

/// Hidden pair in columns.
#[inline(never)]
pub(crate) fn hidden_pairs_cols<const N: usize, const K: usize>(
    board: &Board<N>,
    allowed: &mut [[u32; N]; N],
    col_count: &mut [[u16; N]; N],
    col_pos: &mut [[u32; N]; N],
    row_count: &mut [[u16; N]; N],
    box_count: &mut [[u16; N]; N],
    row_pos: &mut [[u32; N]; N],
    box_pos: &mut [[u32; N]; N],
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

/// Hidden pair in boxes.
#[inline(never)]
pub(crate) fn hidden_pairs_boxes<const N: usize, const K: usize>(
    board: &Board<N>,
    allowed: &mut [[u32; N]; N],
    box_count: &mut [[u16; N]; N],
    box_pos: &mut [[u32; N]; N],
    row_count: &mut [[u16; N]; N],
    col_count: &mut [[u16; N]; N],
    row_pos: &mut [[u32; N]; N],
    col_pos: &mut [[u32; N]; N],
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
