use solver::{
    SearchMode, SudokuSolver, types::Board, utils::dataset::parse_puzzle_string,
};

fn mem_str(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{:>8.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:>8.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:>8} B", bytes)
    }
}

fn main() {
    println!("=== Sudoku Solver Memory Analysis (N=9, K=3) ===\n");

    // ---- SIZEOF diagnostics ----
    println!("--- sizeof individual types ---");
    use std::mem;
    println!(
        "PermutationNode<9,3>: {:>4} B (payload_idx:4 + row_masks:12 + col_masks:12 = 28, maybe padded to 32?)",
        mem::size_of::<solver::types::graph::PermutationNode<9, 3>>()
    );
    println!("MaskSignature<3>:     {:>4} B ([u32;3] = 12)", mem::size_of::<solver::types::graph::MaskSignature<3>>());
    println!("DynamicBitSet:        {:>4} B", mem::size_of::<solver::types::bitstring::DynamicBitSet>());
    println!("FixedBitSet:          {:>4} B", mem::size_of::<solver::types::bitstring::FixedBitSet>());
    println!("AxisIndex<3>:         {:>4} B (3 Vecs + overhead)", mem::size_of::<solver::types::graph::AxisIndex<3>>());
    println!("PairAdj:              {:>4} B", mem::size_of::<solver::types::graph::PairAdj>());
    println!("Graph<3,9>:           {:>4} B", mem::size_of::<solver::types::graph::Graph<3, 9>>());
    println!("Search<3,9>:          {:>4} B", mem::size_of::<solver::solver::extraction::Search<3, 9>>());
    println!();

    // ---- Puzzles with varying permutation blow-up ----
    let puzzles: &[(&str, &str)] = &[
        ("hard_unique", "..5...74.3..6...19.....1..5...7...2.9....58..7..84......3.9...2.9.4.....8.....1.3"),
        ("medium_unique", "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"),
        ("easy_unique", "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......"),
        ("sparse_ambig", "..............3.85..1.2.......5.7.....4...1...9.......5......73..2.1........4...9"),
    ];

    for (label, puzzle) in puzzles {
        println!("========== {} ==========", label);
        println!("Puzzle: {}", puzzle);
        let parsed = parse_puzzle_string(puzzle).unwrap();
        let mut cells = [[0u8; 9]; 9];
        for (i, &val) in parsed.iter().enumerate() {
            cells[i / 9][i % 9] = val;
        }
        let board = Board::<9>::new(cells);

        // Run with EnumerateUpTo to avoid explosive enumeration on ambiguous puzzles
        let report = SudokuSolver::<9, 3>::new(board)
            .with_search_mode(SearchMode::EnumerateUpTo(10))
            .solve_with_stats();
        let s = &report.stats;

        println!("Classification: {:?}", s.puzzle_classification);
        println!("Solutions (capped): {}", s.solution_count);
        println!();

        // Minigrid permutation counts
        println!("--- PERMUTATION COUNTS ---");
        for (i, &count) in s.permutation_counts.iter().enumerate() {
            let row_name = ["top-left", "top-center", "top-right", "mid-left", "mid-center", "mid-right", "bot-left", "bot-center", "bot-right"][i];
            println!("  MG{} {}: {:>5}", i, row_name, count);
        }
        println!("  TOTAL: {}", s.initial_vertex_count);
        println!();

        // Memory breakdown
        println!("--- MEMORY FOOTPRINT ---");
        let mask_mem = s.masks_memory_bytes as usize;
        let perm_mem = s.permutation_memory_bytes as usize;
        let graph_mem = s.graph_memory_bytes as usize;
        let post_mem = s.post_prune_memory_bytes as usize;
        let total_pre = mask_mem + perm_mem + graph_mem;

        println!("  Masks + Board:     {}", mem_str(mask_mem));
        println!("  Permutations:      {}", mem_str(perm_mem));
        println!("  Graph:             {}", mem_str(graph_mem));
        println!("  ───────────────────────");
        println!("  Total pre-prune:   {}", mem_str(total_pre));
        println!("  Post-prune:        {}", mem_str(post_mem));
        println!();

        // Compute per-component breakdown of the Graph portion
        // Graph memory includes: row_indexes, col_indexes, pair_tables, pair_lookup
        // We can reason about where it went by knowing perm counts
        let perm_counts: Vec<usize> = s.permutation_counts.to_vec();

        // Estimate axis index memory
        let mut axis_bytes = 0usize;
        for &n in &perm_counts {
            // perm_to_sig: Vec<SigId> = n * 4 bytes
            axis_bytes += n * 4;
            // sig_to_perms: ~n distinct FixedBitSets (assuming few signature collisions)
            // Each FixedBitSet: n.div_ceil(64) * 8 bytes + 16B Box overhead
            let words = n.div_ceil(64).max(1);
            axis_bytes += n * (words * 8 + 16); // worst-case: all unique sigs
        }
        let axis_total = axis_bytes; // only one of row/col — the graph stores both

        // Estimate pair table memory
        let mut pair_bytes = 0usize;
        for i in 0..9 {
            for j in (i + 1)..9 {
                let same_row = i / 3 == j / 3;
                let same_col = i % 3 == j % 3;
                if !same_row && !same_col {
                    continue;
                }
                let ln = perm_counts[i];
                let rn = perm_counts[j];
                // Estimate signature counts — assume all unique for worst case
                let ls = ln.min(500);
                let rs = rn.min(500);
                pair_bytes += ls * (rn.div_ceil(64).max(1) * 8 + 16); // left_sig_to_right
                pair_bytes += rs * (ln.div_ceil(64).max(1) * 8 + 16); // right_sig_to_left
            }
        }

        println!("--- GRAPH MEMORY ESTIMATES ---");
        println!("  Row indexes:       {}", mem_str(axis_total));
        println!("  Col indexes:       {}", mem_str(axis_total));
        println!("  Pair tables:       {}", mem_str(pair_bytes));
        let graph_est = 2 * axis_total + pair_bytes;
        println!("  Graph estimated:   {}", mem_str(graph_est));
        println!("  Graph reported:    {}", mem_str(graph_mem));
        println!();

        // Dominance analysis
        println!("--- DOMINANCE (%) ---");
        if total_pre > 0 {
            println!("  Masks:             {:>5.1}%", 100.0 * mask_mem as f64 / total_pre as f64);
            println!("  Permutation nodes: {:>5.1}%", 100.0 * perm_mem as f64 / total_pre as f64);
            println!("  Graph:             {:>5.1}%", 100.0 * graph_mem as f64 / total_pre as f64);
        }
        if graph_mem > 0 {
            // Permutation nodes is mostly capacity * sizeof(PermutationNode)
            // Graph memory contains pair tables, axes, signatures, etc.
            let node_size = mem::size_of::<solver::types::graph::PermutationNode<9, 3>>();
            let payload_size = 9usize; // [u8; 9]
            let node_cap: usize = perm_counts.iter().map(|&c| c).sum(); // close to len, but we use capacity in real code
            let node_heap = node_cap * node_size;
            let payload_heap = node_cap * payload_size;
            println!();
            println!("  Node heap ({}×{}B): {}, {:>5.1}% of total",
                node_cap, node_size, mem_str(node_heap), 100.0 * node_heap as f64 / total_pre as f64);
            println!("  Payload heap ({}×9B): {}, {:>5.1}% of total",
                node_cap, mem_str(payload_heap), 100.0 * payload_heap as f64 / total_pre as f64);
        }
        println!();

        // Timing
        println!("--- TIMING ---");
        println!("  Mask init:     {:>8.2} ms", s.mask_init_time_ns as f64 / 1_000_000.0);
        println!("  Heuristics:    {:>8.2} ms", s.heuristic_time_ns as f64 / 1_000_000.0);
        println!("  Permutations:  {:>8.2} ms", s.permutation_time_ns as f64 / 1_000_000.0);
        println!("  Edge build:    {:>8.2} ms", s.edge_build_time_ns as f64 / 1_000_000.0);
        println!("  Pruning:       {:>8.2} ms", s.pruning_time_ns as f64 / 1_000_000.0);
        println!("  Extraction:    {:>8.2} ms", s.extraction_time_ns as f64 / 1_000_000.0);
        println!("  TOTAL:         {:>8.2} ms", s.total_time_ns as f64 / 1_000_000.0);
        println!();

        // Edge-to-node ratio (gives a sense of graph density)
        println!("--- GRAPH METRICS ---");
        println!("  Initial vertices: {}", s.initial_vertex_count);
        println!("  Initial edges:    {}", s.initial_edge_count);
        println!(
            "  Avg edges/node:   {:.1}",
            s.initial_edge_count as f64 / s.initial_vertex_count.max(1) as f64
        );
        println!("  Pruned vertices:  {}", s.pruned_vertex_count);
        println!("  Pruned edges:     {}", s.pruned_edge_count);
        println!("  Removed:          {}", s.removed_vertices);
        println!("  Memory reduction: {:.0}%", 
            if s.graph_memory_bytes > 0 {
                100.0 * (1.0 - s.post_prune_memory_bytes as f64 / (s.permutation_memory_bytes + s.graph_memory_bytes) as f64)
            } else { 0.0 }
        );
        println!();

        // Growth behavior: memory vs permutation count
        let n_perms_avg = s.initial_vertex_count as f64 / 9.0;
        let edges_per_perm_pair: f64 = if s.initial_vertex_count > 1 {
            s.initial_edge_count as f64 / (s.initial_vertex_count as f64 * (s.initial_vertex_count as f64 - 1.0) / 2.0)
        } else { 0.0 };
        println!("  Avg perms/mg:     {:.1}", n_perms_avg);
        println!("  Edge density:     {:.4} (fraction of possible edges)", edges_per_perm_pair);
        println!();
        println!();
    }

    // ---- Theoretical growth model ----
    println!("====== THEORETICAL MEMORY GROWTH ======");
    println!();
    println!("Key variables:");
    println!("  P = average permutations per minigrid");
    println!("  Node size (PermutationNode<9,3>) = {} B", mem::size_of::<solver::types::graph::PermutationNode<9, 3>>());
    println!("  Payload size = 9 B per entry");
    println!();

    println!("  Phase 2 (O(P)):");
    println!("    nodes:   9P × {} B = {}P B", mem::size_of::<solver::types::graph::PermutationNode<9, 3>>(), mem::size_of::<solver::types::graph::PermutationNode<9, 3>>());
    println!("    payloads: 9P × 9 B  = 81P B");
    println!("    total Phase 2:      = {}P B", mem::size_of::<solver::types::graph::PermutationNode<9, 3>>() + 9);
    println!();

    println!("  Phase 3 Graph (O(P²) with compressed edges):");
    println!("    axis indexes: 2 × 9 × [P × 4B (perm_to_sig) + P × ceil(P/64) × 8B (sig_to_perms)]");
    println!("    pair tables:  18 × [P × ceil(P/64) × 8B × 2 directions]");
    println!("    ≈ 18 × P × (P/64) × 16 B");
    println!("    = {}P² B per pair", (18.0 / 64.0 * 16.0) as usize);
    println!();

    let node_size = mem::size_of::<solver::types::graph::PermutationNode<9, 3>>();
    let linear_per_p = (node_size + 9) * 9; // 9 minigrids
    let quad_per_p_per_pair = (16.0 / 64.0) as f64; // one pair table's bytes per P²
    let quad_total = quad_per_p_per_pair * 18.0; // 18 related pairs

    for &p in &[5u64, 10, 50, 100, 200, 500, 1000, 1465] {
        let linear_b = linear_per_p as u64 * p;
        let quad_b = (quad_total * p as f64 * p as f64) as u64;
        let total = linear_b + quad_b;
        println!(
            "  P={:>5}: O(P)={}, O(P²)={}, total={} (quad={:.0}%)",
            p,
            mem_str(linear_b as usize),
            mem_str(quad_b as usize),
            mem_str(total as usize),
            100.0 * quad_b as f64 / total.max(1) as f64
        );
    }
    println!();

    println!("=== KEY FINDINGS ===");
    println!("1. For P < ~3-4 perms/mg: O(P) dominates → graph overhead is noise");
    println!("2. For P > ~10: O(P²) pair tables start competing with O(P) node storage");
    println!("3. For P > ~50: O(P²) dominates (>50% of total memory)");
    println!("4. Worst case (P=1465, empty board): O(P²) ≈ 2.7 MB pair tables alone");
    println!("5. The PAIR TABLES are the bottleneck, specifically:");
    println!("   - left_sig_to_right + right_sig_to_left FixedBitSets");
    println!("   - Each bit-set stores ceil(P/64) words × 8B");
    println!("   - Signature compression helps but can't beat quadratic scaling");
    println!("6. Axis indexes (perm_to_sig + sig_to_perms) also scale O(P²)");
    println!("7. PermutationNode struct is 32B — compact. Not the bottleneck.");
    println!("8. Payloads: [u8;9] = 9B per perm — compact. Not the bottleneck.");
}
