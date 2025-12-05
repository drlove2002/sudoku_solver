#[derive(Debug, Clone)]
pub struct Board<const N: usize> {
    pub cells: [[u8; N]; N],
}

impl<const N: usize> Board<N> {
    pub fn size(&self) -> usize {
        N
    }

    pub fn n(&self) -> usize {
        (N as f64).sqrt() as usize
    }

    pub fn get_cell(&self, idx: usize) -> u8 {
        let r = idx / N;
        let c = idx % N;
        self.cells[r][c]
    }

    pub fn is_valid(&self) -> bool {
        let root_n = self.n();
        // Check rows and columns
        for i in 0..N {
            let mut row_seen = vec![false; N + 1];
            let mut col_seen = vec![false; N + 1];
            for j in 0..N {
                let row_val = self.cells[i][j];
                let col_val = self.cells[j][i];
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
        for box_row in 0..root_n {
            for box_col in 0..root_n {
                let mut box_seen = vec![false; N + 1];
                for i in 0..root_n {
                    for j in 0..root_n {
                        let val = self.cells[box_row * root_n + i][box_col * root_n + j];
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
