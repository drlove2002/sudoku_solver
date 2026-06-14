fn main() {
    const N: usize = 9;
    const K: usize = 3;
    let puzzle: [[u8; N]; N] = [
        [8,0,0,0,0,7,0,9,0],
        [0,0,0,9,4,0,0,0,5],
        [0,0,3,0,8,0,0,7,0],
        [0,0,0,5,0,0,4,0,8],
        [5,6,0,0,0,0,0,1,0],
        [0,0,0,0,6,1,0,3,0],
        [0,0,8,0,7,0,6,0,0],
        [0,5,0,3,0,0,9,0,0],
        [1,0,0,0,0,0,0,0,0],
    ];
    
    let mut board = solver::types::Board::<N>::new(puzzle);
    let mut masks = solver::types::masks::Masks::<N>::default();
    masks.generate(&board);
    
    let filled = solver::solver::heuristics::propagate_constraints::<N, K>(&mut board, &mut masks);
    println!("Heuristics filled: {}", filled);
    println!("=== BOARD AFTER HEURISTICS ===");
    for r in 0..N {
        println!("{:?}", board.cells[r]);
    }
    
    let perms = solver::solver::permutations::generate_all_permutations::<N, K>(&board, &masks);
    let mut graph = solver::types::graph::Graph::<K, N>::new(perms);
    graph.create_edges();
    solver::solver::pruning::Pruner::new(&mut graph).run_local();
    
    let extractor = solver::solver::extraction::Extractor::<K, N>::new(&graph);
    let solutions = extractor.run();
    
    if let Some(sol) = solutions.first() {
        println!("\n=== ONE VALID SOLUTION ===");
        for r in 0..N {
            println!("{:?},", sol.board.cells[r]);
        }
        println!("\nValid: {}", sol.board.is_valid());
    }
    
    // Also print permutation counts per minigrid
    println!("\n=== PERMUTATION COUNTS ===");
    for mg_id in 0..N {
        println!("MG{}: {} perms", mg_id, graph.permutation_count(mg_id));
    }
    println!("Total edges: {}", graph.total_edges());
}
