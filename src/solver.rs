use crate::types::Board;
use rayon::prelude::*;

pub struct SudokuSolver<const N: usize> {
    board: Board<N>,
    pub permutations: Vec<Vec<Vec<u8>>>, // Precomputed permutations per minigrid
    initial_allowed_masks: Vec<u32>,     // Precomputed allowed masks for each cell
}

impl<const N: usize> SudokuSolver<N> {
    pub fn new(board: Board<N>) -> Self {
        SudokuSolver {
            board,
            permutations: Vec::new(),
            initial_allowed_masks: Vec::new(),
        }
    }

    pub fn solve(&mut self) -> Vec<Vec<u8>> {
        println!("\n=== PHASE 1: PARSING AND MASK INITIALIZATION ===");
        self.initialize_masks();
        println!("✓ Initial allowed masks pre-calculated (optimized)");

        println!("\n=== PHASE 2: MINIGRID PERMUTATION GENERATION ===");
        self.generate_all_permutations();

        // Print permutation counts and details
        for (k, perms) in self.permutations.iter().enumerate() {
            println!("\nMinigrid {}: {} permutation(s)", k, perms.len());
            for (p_idx, perm) in perms.iter().enumerate() {
                print!("  P{}: [", p_idx);
                for (i, val) in perm.iter().enumerate() {
                    print!("{}", val);
                    if (i + 1) % N == 0 && i < perm.len() - 1 {
                        print!(" | ");
                    } else if i < perm.len() - 1 {
                        print!(" ");
                    }
                }
                println!("]");
            }
        }

        println!("\n=== PHASE 3: COMPATIBILITY GRAPH CONSTRUCTION ===");
        let (vertices, adj) = self.build_compatibility_graph();
        println!("Total vertices: {}", vertices.len());
        println!("Compatibility checks completed");

        println!("\n=== PHASE 4: ITERATIVE DEGREE-BASED PRUNING ===");
        let active = self.prune_graph(&vertices, &adj);

        println!("\n=== PHASE 5: SOLUTION EXTRACTION ===");
        self.extract_solution(&vertices, &active)
    }

    fn initialize_masks(&mut self) {
        let size = self.board.size();
        let n = self.board.n();
        println!("Board size: {}x{}, Box size: {}x{}", size, size, n, n);

        // Optimized approach: O(N^2)
        // 1. Calculate used masks for rows, cols, boxes
        let mut row_used = vec![0u32; size];
        let mut col_used = vec![0u32; size];
        let mut box_used = vec![0u32; size];

        for i in 0..size * size {
            let val = self.board.get_cell(i);
            if val != 0 {
                let r = i / size;
                let c = i % size;
                let b = (r / n) * n + (c / n);
                let mask = 1 << val;

                if (row_used[r] & mask) != 0
                    || (col_used[c] & mask) != 0
                    || (box_used[b] & mask) != 0
                {
                    panic!("Invalid board: duplicate value found");
                }

                row_used[r] |= mask;
                col_used[c] |= mask;
                box_used[b] |= mask;
            }
        }

        // 2. Calculate allowed masks for each cell
        let valid_digits_mask = (1 << (size + 1)) - 2;
        self.initial_allowed_masks = vec![0u32; size * size];

        for i in 0..size * size {
            if self.board.get_cell(i) == 0 {
                let r = i / size;
                let c = i % size;
                let b = (r / n) * n + (c / n);

                let used = row_used[r] | col_used[c] | box_used[b];
                self.initial_allowed_masks[i] = (!used) & valid_digits_mask;
            }
        }
    }

    fn generate_all_permutations(&mut self) {
        let initial_allowed_masks = &self.initial_allowed_masks;
        let size = self.board.size();
        let n = self.board.n();
        self.permutations = (0..size)
            .into_par_iter()
            .map(|k| {
                let mut box_cells = vec![0u8; size];
                let mut empty_indices = Vec::new();

                // Extract box cells and identify fixed digits
                let start_r = (k / n) * n;
                let start_c = (k % n) * n;

                for r in 0..n {
                    for c in 0..n {
                        let idx = (start_r + r) * size + (start_c + c);
                        let val = self.board.get_cell(idx);
                        box_cells[r * n + c] = val;
                        if val == 0 {
                            empty_indices.push(r * n + c);
                        }
                    }
                }

                // DFS to find all permutations
                let mut results = Vec::new();
                Self::generate_permutations_dfs(
                    box_cells,
                    &empty_indices,
                    initial_allowed_masks,
                    k,
                    n,
                    size,
                    0,
                    0,
                    &mut results,
                );
                results
            })
            .collect();
    }

    fn build_compatibility_graph(&self) -> (Vec<(usize, usize)>, Vec<Vec<usize>>) {
        let size = self.board.size();
        let n = self.board.n();
        // Flatten permutations into a list of vertices (box_idx, perm_idx)
        let mut vertices = Vec::new();
        let mut box_perm_start_indices = vec![0; size + 1];

        for (k, perms) in self.permutations.iter().enumerate() {
            box_perm_start_indices[k] = vertices.len();
            for (p_idx, _) in perms.iter().enumerate() {
                vertices.push((k, p_idx));
            }
        }
        box_perm_start_indices[size] = vertices.len();

        // Adjacency list: for each vertex, list of compatible neighbor vertices
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); vertices.len()];

        // We can parallelize edge creation
        let edges: Vec<(usize, usize)> = (0..size)
            .into_par_iter()
            .flat_map(|k1| {
                let mut local_edges = Vec::new();
                let r1 = k1 / n;
                let c1 = k1 % n;

                for k2 in (k1 + 1)..size {
                    let r2 = k2 / n;
                    let c2 = k2 % n;

                    let is_row_dep = r1 == r2;
                    let is_col_dep = c1 == c2;

                    if !is_row_dep && !is_col_dep {
                        continue;
                    }

                    // Compare all permutations of k1 with all permutations of k2
                    let start1 = box_perm_start_indices[k1];
                    let end1 = box_perm_start_indices[k1 + 1];
                    let end2 = box_perm_start_indices[k2 + 1];

                    for i in start1..end1 {
                        let p1_idx = vertices[i].1;
                        let p1 = &self.permutations[k1][p1_idx];

                        (box_perm_start_indices[k2]..end2).for_each(|j| {
                            let p2_idx = vertices[j].1;
                            let p2 = &self.permutations[k2][p2_idx];

                            let mut compatible = true;

                            if is_row_dep {
                                // Check shared rows
                                for r in 0..n {
                                    let mut mask1 = 0u32;
                                    let mut mask2 = 0u32;
                                    for c in 0..n {
                                        mask1 |= 1 << p1[r * n + c];
                                        mask2 |= 1 << p2[r * n + c];
                                    }
                                    if (mask1 & mask2) != 0 {
                                        compatible = false;
                                        break;
                                    }
                                }
                            }

                            if compatible && is_col_dep {
                                // Check shared columns
                                for c in 0..n {
                                    let mut mask1 = 0u32;
                                    let mut mask2 = 0u32;
                                    for r in 0..n {
                                        mask1 |= 1 << p1[r * n + c];
                                        mask2 |= 1 << p2[r * n + c];
                                    }
                                    if (mask1 & mask2) != 0 {
                                        compatible = false;
                                        break;
                                    }
                                }
                            }

                            if compatible {
                                local_edges.push((i, j));
                            }
                        });
                    }
                }
                local_edges
            })
            .collect();

        println!("Total edges found: {}", edges.len());

        // Populate adjacency list
        for (u, v) in edges {
            adj[u].push(v);
            adj[v].push(u);
        }

        (vertices, adj)
    }

    fn prune_graph(&self, vertices: &[(usize, usize)], adj: &[Vec<usize>]) -> Vec<bool> {
        let mut active = vec![true; vertices.len()];
        let mut degrees = vec![0; vertices.len()];
        let mut changed = true;
        let n = self.board.n();

        // Calculate initial degrees
        for i in 0..vertices.len() {
            degrees[i] = adj[i].len();
        }
        println!("Initial vertex degrees calculated");

        let mut iteration = 0;
        while changed {
            changed = false;
            let mut removed_count = 0;
            iteration += 1;

            for i in 0..vertices.len() {
                if !active[i] {
                    continue;
                }

                let (k, _p_idx) = vertices[i];
                // Determine min degree based on box position
                // Corner: 2, Edge: 3, Interior: 4
                let r = k / n;
                let c = k % n;
                let mut min_degree = 4;
                if r == 0 || r == n - 1 {
                    min_degree -= 1;
                }
                if c == 0 || c == n - 1 {
                    min_degree -= 1;
                }

                if degrees[i] < min_degree {
                    active[i] = false;
                    changed = true;
                    removed_count += 1;
                    // Update neighbors
                    for &neighbor in &adj[i] {
                        if active[neighbor] {
                            degrees[neighbor] -= 1;
                        }
                    }
                }
            }

            if removed_count > 0 {
                println!(
                    "Iteration {}: Removed {} vertices",
                    iteration, removed_count
                );
            }
        }
        println!("Pruning converged after {} iterations", iteration - 1);
        active
    }

    fn extract_solution(&self, vertices: &[(usize, usize)], active: &[bool]) -> Vec<Vec<u8>> {
        let size = self.board.size();
        let n = self.board.n();
        // Collect remaining valid permutations per box
        let mut valid_perms_per_box = vec![Vec::new(); size];
        let mut active_count = 0;
        for i in 0..vertices.len() {
            if active[i] {
                let (k, p_idx) = vertices[i];
                valid_perms_per_box[k].push(self.permutations[k][p_idx].clone());
                active_count += 1;
            }
        }
        println!("Active vertices after pruning: {}", active_count);

        for (k, perms) in valid_perms_per_box.iter().enumerate() {
            println!(
                "Minigrid {}: {} valid permutation(s) remaining",
                k,
                perms.len()
            );
        }

        // If any box has 0 valid permutations, unsolvable
        if valid_perms_per_box.iter().any(|v| v.is_empty()) {
            println!("❌ Puzzle is UNSOLVABLE (some boxes have 0 valid permutations)");
            return vec![];
        }

        let mut solutions = Vec::new();
        let mut current_solution = vec![vec![]; size];
        Self::find_solution(
            0,
            &valid_perms_per_box,
            &mut current_solution,
            &mut solutions,
            size,
            n,
        );

        let solution_count = solutions.len();
        if solution_count == 0 {
            println!("❌ Puzzle is UNSOLVABLE (no compatible configuration found)");
        } else if solution_count == 1 {
            println!("✓ Puzzle has a UNIQUE solution");
        } else {
            println!("⚠ Puzzle is AMBIGUOUS ({} solutions found)", solution_count);
        }

        solutions
    }

    fn find_solution(
        box_idx: usize,
        valid_perms: &Vec<Vec<Vec<u8>>>,
        current_solution: &mut Vec<Vec<u8>>,
        solutions: &mut Vec<Vec<u8>>,
        size: usize,
        n: usize,
    ) {
        if !solutions.is_empty() {
            return;
        } // Find just one for now

        if box_idx == size {
            // Construct full board
            let mut full_board = vec![0u8; size * size];
            (0..size).for_each(|k| {
                let start_r = (k / n) * n;
                let start_c = (k % n) * n;
                for r in 0..n {
                    for c in 0..n {
                        let idx = (start_r + r) * size + (start_c + c);
                        full_board[idx] = current_solution[k][r * n + c];
                    }
                }
            });
            solutions.push(full_board);
            return;
        }

        for perm in &valid_perms[box_idx] {
            // Check compatibility with already placed boxes
            let mut compatible = true;
            let r1 = box_idx / n;
            let c1 = box_idx % n;

            for (k_prev, prev_perm) in current_solution[..box_idx].iter().enumerate() {
                let r2 = k_prev / n;
                let c2 = k_prev % n;

                if r1 != r2 && c1 != c2 {
                    continue;
                }

                if r1 == r2 {
                    // Check shared rows
                    for r in 0..n {
                        let mut mask1 = 0u32;
                        let mut mask2 = 0u32;
                        for c in 0..n {
                            mask1 |= 1 << perm[r * n + c];
                            mask2 |= 1 << prev_perm[r * n + c];
                        }
                        if (mask1 & mask2) != 0 {
                            compatible = false;
                            break;
                        }
                    }
                }

                if compatible && c1 == c2 {
                    // Check shared columns
                    for c in 0..n {
                        let mut mask1 = 0u32;
                        let mut mask2 = 0u32;
                        for r in 0..n {
                            mask1 |= 1 << perm[r * n + c];
                            mask2 |= 1 << prev_perm[r * n + c];
                        }
                        if (mask1 & mask2) != 0 {
                            compatible = false;
                            break;
                        }
                    }
                }

                if !compatible {
                    break;
                }
            }

            if compatible {
                current_solution[box_idx] = perm.clone();
                Self::find_solution(
                    box_idx + 1,
                    valid_perms,
                    current_solution,
                    solutions,
                    size,
                    n,
                );
                if !solutions.is_empty() {
                    return;
                }
            }
        }
    }

    fn generate_permutations_dfs(
        mut box_cells: Vec<u8>,
        empty_indices: &[usize],
        initial_allowed_masks: &[u32],
        box_id: usize,
        n: usize,
        size: usize,
        idx: usize,
        used_mask: u32,
        results: &mut Vec<Vec<u8>>,
    ) {
        if idx == empty_indices.len() {
            results.push(box_cells);
            return;
        }

        let pos = empty_indices[idx];

        // Calculate global index to get the static allowed mask
        let box_r = pos / n;
        let box_c = pos % n;
        let start_r = (box_id / n) * n;
        let start_c = (box_id % n) * n;
        let global_r = start_r + box_r;
        let global_c = start_c + box_c;
        let global_idx = global_r * size + global_c;

        let allowed_mask = initial_allowed_masks[global_idx];

        // Valid digits are those allowed by static constraints AND not used dynamically in this box
        let mut candidates = allowed_mask & !used_mask;

        while candidates != 0 {
            let digit_bit = candidates & (!candidates + 1); // Lowest set bit
            candidates ^= digit_bit;
            let digit = digit_bit.trailing_zeros() as u8;

            box_cells[pos] = digit;
            Self::generate_permutations_dfs(
                box_cells.clone(),
                empty_indices,
                initial_allowed_masks,
                box_id,
                n,
                size,
                idx + 1,
                used_mask | digit_bit,
                results,
            );
        }
    }
}
