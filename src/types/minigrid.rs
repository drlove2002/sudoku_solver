use super::Board;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Minigrid<const N: usize> {
    pub id: usize,
    pub cells: [u8; N],  // Flattened KxK = N
    pub empty_mask: u32, // Bitmask of empty cells
}

impl<const N: usize> Minigrid<N> {
    pub const K: usize = N.isqrt();

    pub fn new(id: usize, board: &Board<N>) -> Self {
        let mut cells = [0u8; N];
        let start_row = (id / Self::K) * Self::K;
        let start_col = (id % Self::K) * Self::K;
        let mut empty_mask = 0u32;
        for i in 0..Self::K {
            for j in 0..Self::K {
                let idx = i * Self::K + j;
                let val = board.cells[start_row + i][start_col + j];
                cells[idx] = val;
                // Update empty_mask oneliner
                empty_mask |= ((val == 0) as u32) << idx;
            }
        }
        Self {
            id,
            cells,
            empty_mask,
        }
    }
}

impl<const N: usize> fmt::Display for Minigrid<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let k = Board::<N>::K;
        write!(f, "[")?;
        for (i, val) in self.cells.iter().enumerate() {
            if i > 0 {
                if i % k == 0 {
                    write!(f, " | ")?;
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "{}", val)?;
        }
        write!(f, "]")
    }
}
