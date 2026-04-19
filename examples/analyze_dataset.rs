use solver::{SudokuSolver, utils::dataset::parse_puzzle_string, init_logger, types::Board};
use std::{env, fs, path::Path};

#[derive(Debug, Clone)]
struct AnalysisResult {
    puzzle_id: usize,
    label: String,
    permutation_counts: Vec<usize>,
    total_invocations: usize,
    initial_vertex_count: usize,
    initial_edge_count: usize,
    edge_build_time_ns: u128,
    pruned_vertex_count: usize,
    pruned_edge_count: usize,
    removed_vertices: usize,
    pruning_time_ns: u128,
    solution_count: usize,
    puzzle_classification: String,
    extraction_time_ns: u128,
    mask_init_time_ns: u128,
    permutation_time_ns: u128,
    total_time_ns: u128,
}

#[derive(Debug, Default, Clone)]
struct AggregateStats {
    puzzle_count: usize,
    unique_count: usize,
    ambiguous_count: usize,
    unsolvable_count: usize,
    total_solutions: usize,
    total_invocations: usize,
    total_initial_vertices: usize,
    total_pruned_vertices: usize,
    total_initial_edges: usize,
    total_pruned_edges: usize,
    total_removed_vertices: usize,
    total_mask_init_ns: u128,
    total_permutation_ns: u128,
    total_edge_build_ns: u128,
    total_pruning_ns: u128,
    total_extraction_ns: u128,
    total_total_ns: u128,
}

impl AggregateStats {
    fn observe(&mut self, result: &AnalysisResult) {
        self.puzzle_count += 1;
        self.total_solutions += result.solution_count;
        self.total_invocations += result.total_invocations;
        self.total_initial_vertices += result.initial_vertex_count;
        self.total_pruned_vertices += result.pruned_vertex_count;
        self.total_initial_edges += result.initial_edge_count;
        self.total_pruned_edges += result.pruned_edge_count;
        self.total_removed_vertices += result.removed_vertices;
        self.total_mask_init_ns += result.mask_init_time_ns;
        self.total_permutation_ns += result.permutation_time_ns;
        self.total_edge_build_ns += result.edge_build_time_ns;
        self.total_pruning_ns += result.pruning_time_ns;
        self.total_extraction_ns += result.extraction_time_ns;
        self.total_total_ns += result.total_time_ns;

        match result.puzzle_classification.as_str() {
            "Unique" => self.unique_count += 1,
            label if label.starts_with("Ambiguous(") => self.ambiguous_count += 1,
            _ => self.unsolvable_count += 1,
        }
    }

    fn avg_us(value: u128, count: usize) -> u128 {
        if count == 0 {
            0
        } else {
            value / count as u128 / 1_000
        }
    }

    fn avg_count(value: usize, count: usize) -> usize {
        if count == 0 { 0 } else { value / count }
    }
}

fn main() {
    init_logger();

    let limit = parse_limit(env::args().skip(1));
    fs::create_dir_all("results").expect("Failed to create results directory");

    let samples = [
        ("easy", "data/sample_easy.txt"),
        ("medium", "data/sample_medium.txt"),
        ("hard", "data/sample_hard.txt"),
    ];

    let mut all_results = Vec::new();

    for (label, path) in samples {
        println!("Analyzing {} puzzles from {}...", label, path);

        if !Path::new(path).exists() {
            eprintln!("  File not found: {}. Skipping.", path);
            continue;
        }

        let content = fs::read_to_string(path).expect("Failed to read sample file");
        let mut puzzles: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        if let Some(limit) = limit {
            puzzles.truncate(limit.min(puzzles.len()));
        }

        println!("  Loaded {} puzzles", puzzles.len());

        let mut label_results = Vec::with_capacity(puzzles.len());
        for (i, puzzle_str) in puzzles.iter().enumerate() {
            match analyze_single_puzzle(i, label, puzzle_str) {
                Ok(result) => label_results.push(result),
                Err(e) => eprintln!("  Error analyzing puzzle {}: {}", i, e),
            }

            if (i + 1) % 10 == 0 || i + 1 == puzzles.len() {
                println!("    Processed {} puzzles...", i + 1);
            }
        }

        write_analysis_csv(&format!("results/{}_analysis.csv", label), &label_results);
        println!("  Results written to results/{}_analysis.csv", label);

        all_results.extend(label_results);
    }

    write_analysis_csv("results/complete_analysis.csv", &all_results);
    write_phase_breakdown_csv("results/phase_breakdown.csv", &all_results);
    write_statistics_summary("results/statistics_summary.txt", &all_results);

    println!("\nAnalysis complete.");
}

fn parse_limit(args: impl Iterator<Item = String>) -> Option<usize> {
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--limit=") {
            return value.parse::<usize>().ok();
        }

        if arg == "--limit" {
            return args.next().and_then(|value| value.parse::<usize>().ok());
        }
    }

    None
}

fn write_analysis_csv(path: &str, results: &[AnalysisResult]) {
    let mut writer = csv::Writer::from_path(path).expect("Failed to create CSV writer");

    writer
        .write_record([
            "difficulty",
            "puzzle_id",
            "P_0",
            "P_1",
            "P_2",
            "P_3",
            "P_4",
            "P_5",
            "P_6",
            "P_7",
            "P_8",
            "total_invocations",
            "initial_vertex_count",
            "initial_edge_count",
            "edge_build_ns",
            "pruned_vertex_count",
            "pruned_edge_count",
            "removed_vertices",
            "pruning_time_ns",
            "solution_count",
            "puzzle_classification",
            "extraction_time_ns",
            "mask_init_time_ns",
            "permutation_time_ns",
            "total_time_ns",
        ])
        .expect("Failed to write header");

    for result in results {
        let mut record = vec![result.label.clone(), result.puzzle_id.to_string()];
        for count in &result.permutation_counts {
            record.push(count.to_string());
        }
        record.push(result.total_invocations.to_string());
        record.push(result.initial_vertex_count.to_string());
        record.push(result.initial_edge_count.to_string());
        record.push(result.edge_build_time_ns.to_string());
        record.push(result.pruned_vertex_count.to_string());
        record.push(result.pruned_edge_count.to_string());
        record.push(result.removed_vertices.to_string());
        record.push(result.pruning_time_ns.to_string());
        record.push(result.solution_count.to_string());
        record.push(result.puzzle_classification.clone());
        record.push(result.extraction_time_ns.to_string());
        record.push(result.mask_init_time_ns.to_string());
        record.push(result.permutation_time_ns.to_string());
        record.push(result.total_time_ns.to_string());

        writer
            .write_record(&record)
            .expect("Failed to write record");
    }

    writer.flush().expect("Failed to flush CSV");
}

fn write_phase_breakdown_csv(path: &str, results: &[AnalysisResult]) {
    let mut writer = csv::Writer::from_path(path).expect("Failed to create phase breakdown CSV");

    writer
        .write_record([
            "difficulty",
            "puzzle_id",
            "mask_init_ns",
            "permutation_ns",
            "edge_build_ns",
            "pruning_ns",
            "extraction_ns",
            "total_time_ns",
        ])
        .expect("Failed to write header");

    for result in results {
        writer
            .write_record([
                result.label.as_str(),
                &result.puzzle_id.to_string(),
                &result.mask_init_time_ns.to_string(),
                &result.permutation_time_ns.to_string(),
                &result.edge_build_time_ns.to_string(),
                &result.pruning_time_ns.to_string(),
                &result.extraction_time_ns.to_string(),
                &result.total_time_ns.to_string(),
            ])
            .expect("Failed to write phase breakdown row");
    }

    writer.flush().expect("Failed to flush phase breakdown CSV");
}

fn write_statistics_summary(path: &str, results: &[AnalysisResult]) {
    let mut overall = AggregateStats::default();
    let mut easy = AggregateStats::default();
    let mut medium = AggregateStats::default();
    let mut hard = AggregateStats::default();

    for result in results {
        overall.observe(result);
        match result.label.as_str() {
            "easy" => easy.observe(result),
            "medium" => medium.observe(result),
            "hard" => hard.observe(result),
            _ => {}
        }
    }

    let content = format!(
        "{overall}\n{easy}\n{medium}\n{hard}",
        overall = render_summary_section("Overall", &overall),
        easy = render_summary_section("Easy", &easy),
        medium = render_summary_section("Medium", &medium),
        hard = render_summary_section("Hard", &hard),
    );

    fs::write(path, content).expect("Failed to write statistics summary");
}

fn render_summary_section(label: &str, stats: &AggregateStats) -> String {
    format!(
        "{label}\n\
         puzzles={puzzles}\n\
         unique={unique}\n\
         ambiguous={ambiguous}\n\
         unsolvable={unsolvable}\n\
         avg_solutions={avg_solutions}\n\
         avg_invocations={avg_invocations}\n\
         avg_initial_vertices={avg_initial_vertices}\n\
         avg_pruned_vertices={avg_pruned_vertices}\n\
         avg_initial_edges={avg_initial_edges}\n\
         avg_pruned_edges={avg_pruned_edges}\n\
         avg_removed_vertices={avg_removed_vertices}\n\
         avg_mask_init_us={avg_mask_init_us}\n\
         avg_permutation_us={avg_permutation_us}\n\
         avg_edge_build_us={avg_edge_build_us}\n\
         avg_pruning_us={avg_pruning_us}\n\
         avg_extraction_us={avg_extraction_us}\n\
         avg_total_us={avg_total_us}\n",
        puzzles = stats.puzzle_count,
        unique = stats.unique_count,
        ambiguous = stats.ambiguous_count,
        unsolvable = stats.unsolvable_count,
        avg_solutions = AggregateStats::avg_count(stats.total_solutions, stats.puzzle_count),
        avg_invocations = AggregateStats::avg_count(stats.total_invocations, stats.puzzle_count),
        avg_initial_vertices =
            AggregateStats::avg_count(stats.total_initial_vertices, stats.puzzle_count),
        avg_pruned_vertices =
            AggregateStats::avg_count(stats.total_pruned_vertices, stats.puzzle_count),
        avg_initial_edges =
            AggregateStats::avg_count(stats.total_initial_edges, stats.puzzle_count),
        avg_pruned_edges = AggregateStats::avg_count(stats.total_pruned_edges, stats.puzzle_count),
        avg_removed_vertices =
            AggregateStats::avg_count(stats.total_removed_vertices, stats.puzzle_count),
        avg_mask_init_us = AggregateStats::avg_us(stats.total_mask_init_ns, stats.puzzle_count),
        avg_permutation_us = AggregateStats::avg_us(stats.total_permutation_ns, stats.puzzle_count),
        avg_edge_build_us = AggregateStats::avg_us(stats.total_edge_build_ns, stats.puzzle_count),
        avg_pruning_us = AggregateStats::avg_us(stats.total_pruning_ns, stats.puzzle_count),
        avg_extraction_us = AggregateStats::avg_us(stats.total_extraction_ns, stats.puzzle_count),
        avg_total_us = AggregateStats::avg_us(stats.total_total_ns, stats.puzzle_count),
    )
}

fn analyze_single_puzzle(
    id: usize,
    label: &str,
    puzzle_str: &str,
) -> Result<AnalysisResult, String> {
    let digits = parse_puzzle_string(puzzle_str)?;

    let board: Board<9> = Board::new(
        digits
            .chunks(9)
            .map(|row| row.to_vec().try_into().unwrap())
            .collect::<Vec<[u8; 9]>>()
            .try_into()
            .unwrap(),
    );

    let solver: SudokuSolver<9, 3> = SudokuSolver::new(board);
    let report = solver.solve_with_stats();
    let stats = report.stats;

    Ok(AnalysisResult {
        puzzle_id: id,
        label: label.to_string(),
        permutation_counts: stats.permutation_counts.into_iter().collect(),
        total_invocations: stats.total_invocations,
        initial_vertex_count: stats.initial_vertex_count,
        initial_edge_count: stats.initial_edge_count,
        edge_build_time_ns: stats.edge_build_time_ns,
        pruned_vertex_count: stats.pruned_vertex_count,
        pruned_edge_count: stats.pruned_edge_count,
        removed_vertices: stats.removed_vertices,
        pruning_time_ns: stats.pruning_time_ns,
        solution_count: stats.solution_count,
        puzzle_classification: format!("{:?}", stats.puzzle_classification),
        extraction_time_ns: stats.extraction_time_ns,
        mask_init_time_ns: stats.mask_init_time_ns,
        permutation_time_ns: stats.permutation_time_ns,
        total_time_ns: stats.total_time_ns,
    })
}
