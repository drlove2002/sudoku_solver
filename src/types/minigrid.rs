use super::Board;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Minigrid<const N: usize> {
    pub id: usize,
    pub cells: [u8; N], // Flattened KxK = N
    pub zero_count: usize,
}

impl<const N: usize> Minigrid<N> {
    pub fn new(id: usize, board: &Board<N>) -> Self {
        let k = Board::<N>::K;
        let mut cells = [0u8; N];
        let start_row = (id / k) * k;
        let start_col = (id % k) * k;
        let mut zero_count = 0;
        for i in 0..k {
            for j in 0..k {
                let val = board.cells[start_row + i][start_col + j];
                cells[i * k + j] = val;
                if val == 0 {
                    zero_count += 1;
                }
            }
        }
        Self {
            id,
            cells,
            zero_count,
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
