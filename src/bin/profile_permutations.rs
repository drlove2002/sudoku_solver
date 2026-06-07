// Profiler: measures Phase 2 permutation generation for 25x25
// Counts solutions and branching stats without storing them.
use std::time::Instant;
use solver::{
    types::{Board, Minigrid, bitstring::DirtyMask, masks::Masks},
    utils::dataset::parse_puzzle_string,
};

const N: usize = 25;
const K: usize = 5;

struct CountDFS {
    mg: Minigrid<N, K>,
    masks: Masks<N>,
    count: u64,
    max_depth: usize,
    branch_hist: [u64; 26],
    nodes_visited: u64,
}

impl CountDFS {
    fn new(mg: Minigrid<N, K>, masks: &Masks<N>) -> Self {
        Self {
            mg,
            masks: Masks {
                boxs: masks.boxs,
                rows: masks.rows,
                cols: masks.cols,
                conflict: masks.conflict,
            },
            count: 0,
            max_depth: 0,
            branch_hist: [0; 26],
            nodes_visited: 0,
        }
    }

    fn run(&mut self) {
        let used = self.masks.boxs[self.mg.id];
        self.nodes_visited += 1;
        self.dfs(used, 0);
    }

    fn find_best_cell(&self, used: DirtyMask<N>) -> Option<(usize, DirtyMask<N>)> {
        let sr = (self.mg.id / K) * K;
        let sc = (self.mg.id % K) * K;
        let mut best = None;
        let mut best_count = 0u32;
        for idx in self.mg.empty {
            if self.mg.cells[idx] != 0 { continue; }
            let mut c = self.masks.conflict[sr + idx / K][sc + idx % K];
            c |= used;
            if c.is_all_set() { return None; }
            let n = c.get().count_ones();
            if n > best_count {
                best_count = n;
                best = Some((idx, c));
                if best_count == N as u32 - 1 { break; }
            }
        }
        best
    }

    fn dfs(&mut self, used: DirtyMask<N>, depth: usize) {
        if self.count % 500_000 == 0 && self.count > 0 {
            eprintln!("  ... {}M solutions ({}K visited)",
                self.count / 1_000_000, self.nodes_visited / 1000);
        }
        if self.count >= 5_000_000 {
            eprintln!("  ABORTING at 5M solutions");
            return;
        }
        self.max_depth = self.max_depth.max(depth);
        self.branch_hist[depth] += 1;

        if let Some((idx, conflict)) = self.find_best_cell(used) {
            let avail = (!conflict.get()) & ((1u32 << N) - 1);
            let mut cands = avail;
            while cands != 0 {
                let num = cands.trailing_zeros() as usize + 1;
                cands &= cands - 1;
                self.mg.cells[idx] = num as u8;
                self.mg.empty.reset(idx);
                let mut next = used;
                next.dirty_set(num);
                self.nodes_visited += 1;
                self.dfs(next, depth + 1);
                self.mg.cells[idx] = 0;
                self.mg.empty.set(idx);
                if self.count >= 5_000_000 { return; }
            }
        } else if used.is_all_set() {
            self.count += 1;
        }
    }
}

fn main() {
    let content = std::fs::read_to_string("data/raw_25_puzzle.txt")
        .expect("missing data/raw_25_puzzle.txt");
    let cleaned: String = content.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == '0' { '.' } else { c })
        .collect();
    let parsed = parse_puzzle_string(&cleaned).unwrap();
    let mut cells = [[0u8; N]; N];
    for (i, &v) in parsed.iter().enumerate() { cells[i/N][i%N] = v; }
    let board = Board::<N>::new(cells);

    let mut masks = Masks::<N>::default();
    masks.generate(&board);

    let mut prop = board;
    let filled = solver::solver::heuristics::propagate_constraints::<N, K>(&mut prop, &mut masks);
    println!("Propagation filled {} cells", filled);

    println!("{:-<90}", "");
    println!("{:>3} | {:>4} | {:>14} | {:>8} | {:>7} | Branch factor at depths 0..max",
        "MG", "Empt", "Solutions", "Time", "MaxD");
    println!("{:-<90}", "");

    for mg_id in 0..N {
        let mg = Minigrid::<N, K>::new(mg_id, &prop);
        let empties = mg.empty.get().count_ones() as usize;
        let start = Instant::now();
        let mut dfs = CountDFS::new(mg, &masks);
        dfs.run();
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let mut branch = String::new();
        for d in 0..=dfs.max_depth.min(10) {
            let b = dfs.branch_hist[d];
            if b > 1_000_000 { branch.push_str(&format!("{}M ", b/1_000_000)); }
            else if b > 1_000 { branch.push_str(&format!("{}K ", b/1_000)); }
            else { branch.push_str(&format!("{} ", b)); }
        }

        let sol_str = if dfs.count >= 5_000_000 {
            ">=5,000,000".to_string()
        } else {
            format!("{}", dfs.count)
        };

        println!("{:>3} | {:>4} | {:>14} | {:>6.1}ms | {:>5} | {}",
            mg_id, empties, sol_str, elapsed, dfs.max_depth, branch);
    }
}
