use log::{debug, trace};
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Board<const N: usize> {
    pub cells: [[u8; N]; N],
}

impl<const N: usize> Board<N> {
    pub const K: usize = N.isqrt();

    pub fn new(cells: [[u8; N]; N]) -> Self {
        assert_eq!(Self::K * Self::K, N, "N must be a perfect square");
        debug!("Creating new Board with size {}x{}", N, N);
        Self { cells }
    }

    #[inline]
    pub fn box_idx(r: usize, c: usize) -> usize {
        (r / Self::K) * Self::K + (c / Self::K)
    }

    pub fn is_valid(&self) -> bool {
        trace!("Checking board validity");
        let k = Self::K;
        // Check rows and columns
        for i in 0..N {
            let mut row_seen = vec![false; N + 1];
            let mut col_seen = vec![false; N + 1];
            for j in 0..N {
                let row_val = self.cells[i][j];
                let col_val = self.cells[j][i];
                if row_val != 0 {
                    if row_seen[row_val as usize] {
                        debug!("Board invalid: duplicate {} in row {}", row_val, i);
                        return false;
                    }
                    row_seen[row_val as usize] = true;
                }
                if col_val != 0 {
                    if col_seen[col_val as usize] {
                        debug!("Board invalid: duplicate {} in col {}", col_val, i);
                        return false;
                    }
                    col_seen[col_val as usize] = true;
                }
            }
        }
        // Check minigrids
        for box_row in 0..k {
            for box_col in 0..k {
                let mut box_seen = vec![false; N + 1];
                for i in 0..k {
                    for j in 0..k {
                        let val = self.cells[box_row * k + i][box_col * k + j];
                        if val != 0 {
                            if box_seen[val as usize] {
                                debug!(
                                    "Board invalid: duplicate {} in box ({}, {})",
                                    val, box_row, box_col
                                );
                                return false;
                            }
                            box_seen[val as usize] = true;
                        }
                    }
                }
            }
        }
        trace!("Board is valid");
        true
    }
}

impl<const N: usize> fmt::Display for Board<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let k = Self::K;
        for i in 0..N {
            if i > 0 && i % k == 0 {
                for _ in 0..N + k - 1 {
                    write!(f, "--")?;
                }
                writeln!(f)?;
            }
            for j in 0..N {
                if j > 0 && j % k == 0 {
                    write!(f, "| ")?;
                }
                write!(f, "{} ", self.cells[i][j])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
