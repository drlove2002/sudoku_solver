pub struct BitMask<const N: usize>;

impl<const N: usize> BitMask<N> {
    pub const fn all_set() -> u32 {
        (1 << N) - 1
    }

    pub fn get(num: u8) -> u32 {
        1 << (num - 1)
    }
}
