use crate::types::Board;
use std::collections::VecDeque;

/// Remove digit d from cell (r,c).
///
/// Decrements house counters, updates position masks, and pushes
/// any newly-revealed naked/hidden singles onto the queue.
///
/// All state is passed explicitly — this is pure logic with no
/// captures, callable from pair scanners or the main propagation loop.
#[allow(clippy::too_many_arguments)]
pub(crate) fn remove_digit<const N: usize, const K: usize>(
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
    if board.cells[r][c] != 0 {
        return;
    }
    let d_idx = d - 1;
    let d_mask = 1u32 << d_idx;
    if allowed[r][c] & d_mask == 0 {
        return;
    }

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
    // Column
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
