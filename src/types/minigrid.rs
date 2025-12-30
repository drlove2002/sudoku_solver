use crate::types::masks::EmptyMask;

use super::Board;

#[derive(Debug)]
pub struct Minigrid<const N: usize, const K: usize> {
    pub id: usize,
    pub cells: [u8; N],      // Flattened KxK = N
    pub empty: EmptyMask<N>, // Bitmask of empty cells
}

impl<const N: usize, const K: usize> Minigrid<N, K> {
    pub const K: usize = N.isqrt();

    pub fn new(id: usize, board: &Board<N>) -> Self {
        let mut cells = [0u8; N];
        let start_row = (id / Self::K) * Self::K;
        let start_col = (id % Self::K) * Self::K;
        let mut empty_mask = EmptyMask::default();
        for i in 0..Self::K {
            for j in 0..Self::K {
                let idx = i * Self::K + j;
                let value = board.cells[start_row + i][start_col + j];
                cells[idx] = value;
                empty_mask.set_value(idx, value);
            }
        }
        Self {
            id,
            cells,
            empty: empty_mask,
        }
    }
}
