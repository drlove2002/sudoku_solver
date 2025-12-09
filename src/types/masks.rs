use log::{debug, trace};

#[derive(Debug)]
pub struct Masks<const N: usize> {
    pub boxs: [u32; N],
    pub rows: [u32; N],
    pub cols: [u32; N],
}

impl<const N: usize> Default for Masks<N> {
    fn default() -> Self {
        Masks {
            boxs: [0u32; N],
            rows: [0u32; N],
            cols: [0u32; N],
        }
    }
}

impl<const N: usize> Masks<N> {
    const K: usize = super::Board::<N>::K;

    pub fn generate(&mut self, board: &super::Board<N>) {
        debug!("Board size: {}x{}, Box size: {}x{}", N, N, Self::K, Self::K);

        for (r, row) in board.cells.iter().enumerate() {
            for (c, &val) in row.iter().enumerate() {
                if val != 0 {
                    let b = (r / Self::K) * Self::K + (c / Self::K);
                    let mask = 1 << (val - 1);
                    trace!("Cell ({}, {}): val={}, mask={:032b}", r, c, val, mask);

                    if (self.rows[r] & mask) != 0
                        || (self.cols[c] & mask) != 0
                        || (self.boxs[b] & mask) != 0
                    {
                        trace!(
                            "Conflict detected! self.rows={:032b}, self.cols={:032b}, self.boxs={:032b}",
                            self.rows[r], self.cols[c], self.boxs[b]
                        );
                        panic!("Invalid board: duplicate value found");
                    }

                    self.rows[r] |= mask;
                    self.cols[c] |= mask;
                    self.boxs[b] |= mask;
                    trace!(
                        "Updated masks: self.rows[{}]={:032b}, self.cols[{}]={:032b}, self.boxs[{}]={:032b}",
                        r, self.rows[r], c, self.cols[c], b, self.boxs[b]
                    );
                }
            }
        }
    }
}
