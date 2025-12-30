use log::{debug, trace};

use crate::types::Board;
mod bitstring;
pub use bitstring::{DirtyMask, EmptyMask};

#[derive(Debug)]
pub struct Masks<const N: usize> {
    pub boxs: [DirtyMask<N>; N],
    pub rows: [DirtyMask<N>; N],
    pub cols: [DirtyMask<N>; N],
    pub conflict: [[DirtyMask<N>; N]; N],
}

impl<const N: usize> Default for Masks<N> {
    fn default() -> Self {
        Masks {
            boxs: [DirtyMask::default(); N],
            rows: [DirtyMask::default(); N],
            cols: [DirtyMask::default(); N],
            conflict: [[DirtyMask::default(); N]; N],
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
                    let val = val as usize;
                    let b = Board::<N>::box_idx(r, c);
                    trace!("Cell ({}, {}): val={}", r, c, val);

                    if self.rows[r].is_dirty(val)
                        || self.cols[c].is_dirty(val)
                        || self.boxs[b].is_dirty(val)
                    {
                        trace!(
                            "Conflict detected! self.rows={}, self.cols={}, self.boxs={}",
                            self.rows[r], self.cols[c], self.boxs[b]
                        );
                        panic!("Invalid board: duplicate value found");
                    }

                    self.rows[r].dirty_set(val);
                    self.cols[c].dirty_set(val);
                    self.boxs[b].dirty_set(val);
                    trace!(
                        "Updated masks: self.rows[{}]={}, self.cols[{}]={}, self.boxs[{}]={}",
                        r, self.rows[r], c, self.cols[c], b, self.boxs[b],
                    );
                }
            }
        }

        for (r, row) in self.conflict.iter_mut().enumerate() {
            for (c, val) in row.iter_mut().enumerate() {
                let b = Board::<N>::box_idx(r, c);
                *val = self.rows[r] | self.cols[c] | self.boxs[b];
                trace!("Updated self.conflict[{}][{}]={}", r, c, val);
            }
        }
    }
}
