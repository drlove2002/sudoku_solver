#[allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub size: usize,
    pub cells: Vec<u8>,
}

impl Board {
    pub fn is_valid(&self) -> bool {
        let n = (self.size as f64).sqrt() as usize;
        // Check rows and columns
        for i in 0..self.size {
            let mut row_seen = vec![false; self.size + 1];
            let mut col_seen = vec![false; self.size + 1];
            for j in 0..self.size {
                let row_val = self.cells[i * self.size + j];
                let col_val = self.cells[j * self.size + i];
                if row_val != 0 {
                    if row_seen[row_val as usize] {
                        return false;
                    }
                    row_seen[row_val as usize] = true;
                }
                if col_val != 0 {
                    if col_seen[col_val as usize] {
                        return false;
                    }
                    col_seen[col_val as usize] = true;
                }
            }
        }
        // Check minigrids
        for box_row in 0..n {
            for box_col in 0..n {
                let mut box_seen = vec![false; self.size + 1];
                for i in 0..n {
                    for j in 0..n {
                        let val = self.cells[(box_row * n + i) * self.size + (box_col * n + j)];
                        if val != 0 {
                            if box_seen[val as usize] {
                                return false;
                            }
                            box_seen[val as usize] = true;
                        }
                    }
                }
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct Minigrid {
    pub id: usize,
    pub cells: Vec<usize>, // Indices into the board
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Permutation {
    pub minigrid_id: usize,
    pub values: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub vertices: Vec<Permutation>,
    pub edges: Vec<(usize, usize)>, // Adjacency list indices
}
