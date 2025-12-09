pub mod board;
pub mod masks;
pub mod minigrid;

pub use board::Board;
pub use minigrid::Minigrid;

pub type Permutations<const N: usize> = Vec<Minigrid<N>>;
