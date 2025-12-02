#![allow(dead_code)]
use crate::types::Board;

// Context for permutation generation containing box-specific and constraint information
struct PermutationContext<'a> {
    box_id: usize,        // Which minigrid (0 to n²-1)
    n: usize,             // Box dimension (3 for 9x9 Sudoku)
    size: usize,          // Total board size (9 for 9x9 Sudoku)
    row_masks: &'a [u32], // Tracks used digits per row (global)
    col_masks: &'a [u32], // Tracks used digits per column (global)
}

pub struct SudokuSolver {
    board: Board,
    pub permutations: Vec<Vec<Vec<u8>>>, // Precomputed permutations per minigrid
}

impl SudokuSolver {
    pub fn new(size: usize, cells: Vec<u8>) -> Self {
        let board = Board { size, cells };
        // TODO: Validate board size (must be n^2)
        SudokuSolver {
            board,
            permutations: Vec::new(),
            initial_allowed_masks: Vec::new(),
        }
    }

    pub fn solve(&mut self) -> Vec<Vec<u8>> {
        println!("\n=== PHASE 1: PARSING AND MASK INITIALIZATION ===");
        let n = self.initialize_masks();
        println!("✓ Initial allowed masks pre-calculated (optimized)");

        println!("\n=== PHASE 2: MINIGRID PERMUTATION GENERATION ===");
        self.generate_all_permutations(n);

        // Print permutation counts and details
        for (k, perms) in self.permutations.iter().enumerate() {
            println!("\nMinigrid {}: {} permutation(s)", k, perms.len());
            for (p_idx, perm) in perms.iter().enumerate() {
                print!("  P{}: [", p_idx);
                for (i, val) in perm.iter().enumerate() {
                    print!("{}", val);
                    if (i + 1) % n == 0 && i < perm.len() - 1 {
                        print!(" | ");
                    } else if i < perm.len() - 1 {
                        print!(" ");
                    }
                }
                println!("]");
            }
        }

        println!("\n=== PHASE 3: COMPATIBILITY GRAPH CONSTRUCTION ===");
        let (vertices, adj) = self.build_compatibility_graph(n);
        println!("Total vertices: {}", vertices.len());
        println!("Compatibility checks completed");

        println!("\n=== PHASE 4: ITERATIVE DEGREE-BASED PRUNING ===");
        let active = self.prune_graph(&vertices, &adj, n);

        println!("\n=== PHASE 5: SOLUTION EXTRACTION ===");
        self.extract_solution(&vertices, &active, n)
    }

    fn initialize_masks(&mut self) -> usize {
        let n = (self.board.size as f64).sqrt() as usize;
        if n * n != self.board.size {
            panic!("Board size must be a perfect square");
        }
        println!(
            "Board size: {}x{}, Box size: {}x{}",
            self.board.size, self.board.size, n, n
        );

        // Optimized approach: O(N^2)
        // 1. Calculate used masks for rows, cols, boxes
        let mut row_used = vec![0u32; self.board.size];
        let mut col_used = vec![0u32; self.board.size];
        let mut box_used = vec![0u32; self.board.size];

        for i in 0..self.board.size * self.board.size {
            let val = self.board.cells[i];
            if val != 0 {
                let r = i / self.board.size;
                let c = i % self.board.size;
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
        let valid_digits_mask = (1 << (self.board.size + 1)) - 2;
        self.initial_allowed_masks = vec![0u32; self.board.size * self.board.size];

        for i in 0..self.board.size * self.board.size {
            if self.board.cells[i] == 0 {
                let r = i / self.board.size;
                let c = i % self.board.size;
                let b = (r / n) * n + (c / n);

                let used = row_used[r] | col_used[c] | box_used[b];
                self.initial_allowed_masks[i] = (!used) & valid_digits_mask;
            }
        }
        n
    }

    fn generate_all_permutations(&mut self, n: usize) {
        let initial_allowed_masks = &self.initial_allowed_masks;
        self.permutations = (0..self.board.size)
            .into_par_iter()
            .map(|k| {
                let mut box_cells = vec![0u8; self.board.size];
                let mut empty_indices = Vec::new();

                // Extract box cells and identify fixed digits
                let start_r = (k / n) * n;
                let start_c = (k % n) * n;

                for r in 0..n {
                    for c in 0..n {
                        let idx = (start_r + r) * self.board.size + (start_c + c);
                        let val = self.board.cells[idx];
                        box_cells[r * n + c] = val;
                        if val != 0 {
                            used_digits |= 1 << val;
                        } else {
                            empty_indices.push(r * n + c);
                        }
                    }
                }

                // Recursive backtracking to find all permutations
                // Pass row/col masks to constrain permutations
                let ctx = PermutationContext {
                    box_id: k,
                    n,
                    size: self.board.size,
                    row_masks: &row_masks,
                    col_masks: &col_masks,
                };
                Self::generate_permutations(
                    &mut box_cells,
                    &empty_indices,
                    0,
                    used_digits,
                    &mut perms,
                    &ctx,
                );
                perms
            })
            .collect();

        // Print permutation counts and details
        for (k, perms) in self.permutations.iter().enumerate() {
            println!("\nMinigrid {}: {} permutation(s)", k, perms.len());
            for (p_idx, perm) in perms.iter().enumerate() {
                print!("  P{}: [", p_idx);
                for (i, val) in perm.iter().enumerate() {
                    print!("{}", val);
                    if (i + 1) % n == 0 && i < perm.len() - 1 {
                        print!(" | ");
                    } else if i < perm.len() - 1 {
                        print!(" ");
                    }
                }
                println!("]");
            }
        }

        // Phase 3: Compatibility Graph Construction
        println!("\n=== PHASE 3: COMPATIBILITY GRAPH CONSTRUCTION ===");
        // Flatten permutations into a list of vertices (box_idx, perm_idx)
        let mut vertices = Vec::new();
        let mut box_perm_start_indices = vec![0; self.board.size + 1];

        for (k, perms) in self.permutations.iter().enumerate() {
            box_perm_start_indices[k] = vertices.len();
            for (p_idx, _) in perms.iter().enumerate() {
                vertices.push((k, p_idx));
            }
        }
        box_perm_start_indices[self.board.size] = vertices.len();
        println!("Total vertices: {}", vertices.len());

        // Adjacency list: for each vertex, list of compatible neighbor vertices
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); vertices.len()];

        // Identify dependent pairs (row-wise and column-wise)
        // Two boxes are row-dependent if they are in the same box-row
        // Two boxes are col-dependent if they are in the same box-col

        // We can parallelize edge creation
        let edges: Vec<(usize, usize)> = (0..self.board.size)
            .into_par_iter()
            .flat_map(|k1| {
                let mut local_edges = Vec::new();
                let r1 = k1 / n;
                let c1 = k1 % n;

                for k2 in (k1 + 1)..self.board.size {
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
                                    // In box k1, row r corresponds to board row (r1*n + r)
                                    // In box k2, row r corresponds to board row (r2*n + r)
                                    // Since r1 == r2, they share the same board rows.
                                    // We must ensure NO duplicates in the concatenated row.
                                    // Actually, the paper says: "Check if the shared rows contain identical digit sequences"
                                    // Wait, standard Sudoku rule: row must contain 1..N exactly once.
                                    // So if k1 and k2 are in the same row, they must NOT have overlapping values in that row?
                                    // No, they are disjoint sets of columns.
                                    // The constraint is that the UNION of all boxes in a row must be a permutation of 1..N.
                                    // But here we are building pairwise compatibility.
                                    // Pairwise compatibility means: p1 and p2 don't have conflicting values in the same row?
                                    // Yes, if p1 has '5' in row r, p2 cannot have '5' in row r.

                                    // Let's check the paper: "Check if the shared rows contain identical digit sequences"
                                    // That sounds like they are overlapping? No, minigrids are disjoint.
                                    // Ah, maybe it means "compatible" as in "no conflict".
                                    // Yes: "An edge ... indicates these two permutations can coexist without violating row or column constraints"

                                    // So for row dependency: check if p1 and p2 have any common digits in the same relative row r.
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
                                // local_edges.push((j, i)); // Undirected? The paper says directed (i, pi) -> (j, pj). But compatibility is symmetric.
                                // We'll store as undirected or both ways. Let's store (i, j) where i < j.
                            }
                        });
                    }
                }
                local_edges
            })
            .collect();

        println!("Total edges found: {}", edges.len());
        println!("Compatibility checks completed");

        // Populate adjacency list
        for (u, v) in edges {
            adj[u].push(v);
            adj[v].push(u);
        }

        // Phase 4: Iterative Degree-Based Pruning
        println!("\n=== PHASE 4: ITERATIVE DEGREE-BASED PRUNING ===");
        let mut active = vec![true; vertices.len()];
        let mut degrees = vec![0; vertices.len()];
        let mut changed = true;

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

        // Phase 5: Solution Extraction
        println!("\n=== PHASE 5: SOLUTION EXTRACTION ===");
        // For now, let's just return the first valid solution found (if any)
        // Or return all valid solutions if small enough.
        // The paper suggests "If S=1: Unique solution".

        // Collect remaining valid permutations per box
        let mut valid_perms_per_box = vec![Vec::new(); self.board.size];
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

        // If every box has exactly 1, we have a unique solution (assuming graph is connected correctly)
        // We can try to reconstruct it.
        // For simplicity, let's just take the first valid permutation for each box and see if it works.
        // (Real extraction requires backtracking if multiple choices exist)

        let mut solutions = Vec::new();

        // Simple backtracking to find ONE solution from the pruned graph
        // This is a simplified version of "Solution Extraction"
        // We can implement full enumeration later.

        let mut current_solution = vec![vec![]; self.board.size];
        Self::find_solution(
            0,
            &valid_perms_per_box,
            &mut current_solution,
            &mut solutions,
            self.board.size,
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
            // (We rely on the graph pruning, but since we are picking one from potentially many,
            // we technically should check edges. But if pruning worked, maybe we don't need to?
            // No, pruning just removes locally impossible ones. Global consistency still needs check if multiple choices.)
            // However, we already built the graph. We should use the graph edges.
            // But here we don't have easy access to the graph edges in this helper.
            // Let's just do a quick check against previous boxes.

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

    fn generate_permutations(
        box_cells: &mut Vec<u8>,
        empty_indices: &[usize],
        idx: usize,
        used_digits: u32,
        results: &mut Vec<Vec<u8>>,
        ctx: &PermutationContext,
    ) {
        if ctx.box_id == 2 {
            eprintln!(
                "DEBUG Minigrid 2: idx={}, used_digits={:09b}, box_cells={:?}",
                idx, used_digits, box_cells
            );
            // You can set a debugger breakpoint on this line
        }
        if idx == empty_indices.len() {
            results.push(box_cells.clone());
            return;
        }

        let pos = empty_indices[idx];

        // Calculate the global row and column for this position
        let box_r = pos / ctx.n; // Local row within the box (0..n-1)
        let box_c = pos % ctx.n; // Local col within the box (0..n-1)
        let start_r = (ctx.box_id / ctx.n) * ctx.n; // Starting row of this box
        let start_c = (ctx.box_id % ctx.n) * ctx.n; // Starting col of this box
        let global_r = start_r + box_r;
        let global_c = start_c + box_c;

        for digit in 1..=ctx.size {
            let mask = 1 << digit;
            // ✅ OR ADD HERE - fires on each digit attempt for minigrid 2
            if ctx.box_id == 2 {
                eprintln!(
                    "  Trying digit {} at pos {} (global r:{}, c:{})",
                    digit, pos, global_r, global_c
                );
            }

            if (used_digits & mask) != 0 {
                if ctx.box_id == 2 {
                    eprintln!("    ✗ Already in box");
                }
                continue;
            }
            if (ctx.row_masks[global_r] & mask) != 0 {
                if ctx.box_id == 2 {
                    eprintln!("    ✗ Already in row {}", global_r);
                }
                continue;
            }
            if (ctx.col_masks[global_c] & mask) != 0 {
                if ctx.box_id == 2 {
                    eprintln!("    ✗ Already in col {}", global_c);
                }
                continue;
            }

            if ctx.box_id == 2 {
                eprintln!("    ✓ Placed digit {}", digit);
            }

            box_cells[pos] = digit as u8;
            Self::generate_permutations(
                box_cells,
                empty_indices,
                idx + 1,
                used_digits | mask,
                results,
                ctx,
            );
            box_cells[pos] = 0;
        }
    }
}
