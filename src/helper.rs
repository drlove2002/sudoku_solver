pub struct BitMask<const N: usize>;

impl<const N: usize> BitMask<N> {
    pub fn is_all_set(bit: u32) -> bool {
        // Check if all bits from 0 to N-1 are set
        // example: N=4 -> 0b0000_1111 = (1 << 4) - 1 = 15
        bit == (1 << N) - 1
    }

    pub const fn all_set() -> u32 {
        (1 << N) - 1
    }

    pub fn candidates_count(forbidden_mask: u32) -> usize {
        // Count number of bits set to 1 in the allowed candidates mask
        let allowed_mask = !forbidden_mask & Self::all_set();
        allowed_mask.count_ones() as usize
    }

    pub fn get(num: u8) -> u32 {
        1 << (num - 1)
    }
}
