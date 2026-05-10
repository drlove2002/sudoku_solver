use chrono::Utc;
use csv::Writer;
use serde::Serialize;
use solver::{
    SearchMode, SolveReport, SudokuSolver, solver::report::PuzzleClass, types::Board,
    utils::dataset::parse_puzzle_string,
};
use std::{env, time::Instant};

const N: usize = 9;
const K: usize = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BenchmarkMode {
    FullSolveStats,
    ClassifyUpToTwo,
    SolveOne,
}

impl BenchmarkMode {
    fn as_str(self) -> &'static str {
        match self {
            BenchmarkMode::FullSolveStats => "full_solve_stats",
            BenchmarkMode::ClassifyUpToTwo => "classify_up_to_two",
            BenchmarkMode::SolveOne => "solve_one",
        }
    }
}

#[derive(Clone, Copy)]
enum ClassificationBasis {
    BoardCheck,
    ClassifyUpToTwo,
    FullSolveStats,
}

impl ClassificationBasis {
    fn as_str(self) -> &'static str {
        match self {
            ClassificationBasis::BoardCheck => "board_check",
            ClassificationBasis::ClassifyUpToTwo => "classify_up_to_two",
            ClassificationBasis::FullSolveStats => "full_solve_stats",
        }
    }
}

#[derive(Clone, Copy, Default)]
struct PhaseTimes {
    mask_init_ns: u128,
    heuristic_ns: u128,
    permutation_ns: u128,
    edge_build_ns: u128,
    pruning_ns: u128,
    extraction_ns: u128,
    total_ns: u128,
}

impl PhaseTimes {
    fn phases(self) -> [(&'static str, u128); 7] {
        [
            ("mask_init", self.mask_init_ns),
            ("heuristic", self.heuristic_ns),
            ("permutation", self.permutation_ns),
            ("edge_build", self.edge_build_ns),
            ("pruning", self.pruning_ns),
            ("extraction", self.extraction_ns),
            ("total", self.total_ns),
        ]
    }
}

#[derive(Serialize)]
struct RawBenchmarkRow<'a> {
    run_id: &'a str,
    dataset_file: &'a str,
    dataset_label: &'a str,
    difficulty: &'a str,
    puzzle_index: usize,
    puzzle: &'a str,
    clues_count: usize,
    benchmark_mode: &'a str,
    search_mode: &'a str,
    is_locally_valid: bool,
    classification_basis: &'a str,
    observed_classification: &'a str,
    observed_classification_detail: &'a str,
    solution_count_observed: usize,
    mask_init_ns: u128,
    heuristic_ns: u128,
    permutation_ns: u128,
    edge_build_ns: u128,
    pruning_ns: u128,
    extraction_ns: u128,
    total_ns: u128,
}

#[derive(Serialize)]
struct LongBenchmarkRow<'a> {
    run_id: &'a str,
    dataset_file: &'a str,
    dataset_label: &'a str,
    difficulty: &'a str,
    puzzle_index: usize,
    puzzle: &'a str,
    clues_count: usize,
    benchmark_mode: &'a str,
    search_mode: &'a str,
    is_locally_valid: bool,
    classification_basis: &'a str,
    observed_classification: &'a str,
    observed_classification_detail: &'a str,
    solution_count_observed: usize,
    phase: &'a str,
    time_ns: u128,
    time_ms: f64,
}

struct RowMeta<'a> {
    run_id: &'a str,
    dataset_file: &'a str,
    dataset_label: &'a str,
    difficulty: &'a str,
    puzzle_index: usize,
    puzzle: &'a str,
    clues_count: usize,
    is_locally_valid: bool,
}

struct RowOutcome<'a> {
    benchmark_mode: BenchmarkMode,
    search_mode: SearchMode,
    classification_basis: ClassificationBasis,
    observed_classification: &'a str,
    observed_classification_detail: String,
    solution_count_observed: usize,
    times: PhaseTimes,
}

fn main() {
    let datasets = [
        ("unique", "easy"),
        ("unique", "medium"),
        ("unique", "hard"),
        ("ambiguous", "easy"),
        ("ambiguous", "medium"),
        ("ambiguous", "hard"),
        ("none", "easy"),
        ("none", "medium"),
        ("none", "hard"),
    ];
    let run_id = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let max_puzzles = env::var("BENCHMARK_MAX_PUZZLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());

    std::fs::create_dir_all("results").expect("results directory must be creatable");

    let mut raw_writer =
        Writer::from_path("results/benchmark_timings.csv").expect("benchmark raw CSV must open");
    let mut long_writer = Writer::from_path("results/benchmark_timings_long.csv")
        .expect("benchmark long CSV must open");

    println!("Starting benchmark run {run_id}...");
    println!("Run with `cargo run --release --bin benchmark` for meaningful timings.");
    if let Some(limit) = max_puzzles {
        println!("Limiting each dataset file to the first {limit} puzzle(s).");
    }

    let total_start = Instant::now();

    for (dataset_label, difficulty) in datasets {
        let filename = format!("data/{}_{}.txt", dataset_label, difficulty);
        println!("Processing {filename}...");

        let content = std::fs::read_to_string(&filename).expect("dataset file missing");
        let lines = content.lines().filter(|line| !line.trim().is_empty());
        let line_iter: Box<dyn Iterator<Item = (usize, &str)>> = match max_puzzles {
            Some(limit) => Box::new(lines.take(limit).enumerate()),
            None => Box::new(lines.enumerate()),
        };

        for (puzzle_index, line) in line_iter {
            let (board, clues_count) = parse_board(line);
            let meta = RowMeta {
                run_id: &run_id,
                dataset_file: &filename,
                dataset_label,
                difficulty,
                puzzle_index,
                puzzle: line,
                clues_count,
                is_locally_valid: board.is_valid(),
            };

            if !meta.is_locally_valid {
                write_row(
                    &mut raw_writer,
                    &mut long_writer,
                    &meta,
                    RowOutcome {
                        benchmark_mode: BenchmarkMode::FullSolveStats,
                        search_mode: SearchMode::EnumerateAll,
                        classification_basis: ClassificationBasis::BoardCheck,
                        observed_classification: "Invalid",
                        observed_classification_detail: "Invalid".to_string(),
                        solution_count_observed: 0,
                        times: PhaseTimes::default(),
                    },
                );
                write_row(
                    &mut raw_writer,
                    &mut long_writer,
                    &meta,
                    RowOutcome {
                        benchmark_mode: BenchmarkMode::ClassifyUpToTwo,
                        search_mode: SearchMode::Classify,
                        classification_basis: ClassificationBasis::BoardCheck,
                        observed_classification: "Invalid",
                        observed_classification_detail: "Invalid".to_string(),
                        solution_count_observed: 0,
                        times: PhaseTimes::default(),
                    },
                );
                write_row(
                    &mut raw_writer,
                    &mut long_writer,
                    &meta,
                    RowOutcome {
                        benchmark_mode: BenchmarkMode::SolveOne,
                        search_mode: SearchMode::First,
                        classification_basis: ClassificationBasis::BoardCheck,
                        observed_classification: "Invalid",
                        observed_classification_detail: "Invalid".to_string(),
                        solution_count_observed: 0,
                        times: PhaseTimes::default(),
                    },
                );
                continue;
            }

            let classify_timer = Instant::now();
            let classify_result = SudokuSolver::<N, K>::new(board).classify_up_to_two();
            let classify_times = PhaseTimes {
                total_ns: classify_timer.elapsed().as_nanos(),
                ..PhaseTimes::default()
            };
            write_row(
                &mut raw_writer,
                &mut long_writer,
                &meta,
                RowOutcome {
                    benchmark_mode: BenchmarkMode::ClassifyUpToTwo,
                    search_mode: SearchMode::Classify,
                    classification_basis: ClassificationBasis::ClassifyUpToTwo,
                    observed_classification: classify_result.coarse_label(),
                    observed_classification_detail: classify_result.detail_label(),
                    solution_count_observed: puzzle_class_solution_count(&classify_result),
                    times: classify_times,
                },
            );

            let solve_one_timer = Instant::now();
            let solve_one_result = SudokuSolver::<N, K>::new(board).solve_one();
            let solve_one_times = PhaseTimes {
                total_ns: solve_one_timer.elapsed().as_nanos(),
                ..PhaseTimes::default()
            };
            let classify_detail = classify_result.detail_label();
            write_row(
                &mut raw_writer,
                &mut long_writer,
                &meta,
                RowOutcome {
                    benchmark_mode: BenchmarkMode::SolveOne,
                    search_mode: SearchMode::First,
                    classification_basis: ClassificationBasis::ClassifyUpToTwo,
                    observed_classification: classify_result.coarse_label(),
                    observed_classification_detail: classify_detail,
                    solution_count_observed: usize::from(solve_one_result.is_some()),
                    times: solve_one_times,
                },
            );

            let full_report = benchmark_full_solve(board);
            let full_stats = &full_report.stats;
            let full_times = PhaseTimes {
                mask_init_ns: full_stats.mask_init_time_ns,
                heuristic_ns: full_stats.heuristic_time_ns,
                permutation_ns: full_stats.permutation_time_ns,
                edge_build_ns: full_stats.edge_build_time_ns,
                pruning_ns: full_stats.pruning_time_ns,
                extraction_ns: full_stats.extraction_time_ns,
                total_ns: full_stats.total_time_ns,
            };
            write_row(
                &mut raw_writer,
                &mut long_writer,
                &meta,
                RowOutcome {
                    benchmark_mode: BenchmarkMode::FullSolveStats,
                    search_mode: SearchMode::EnumerateAll,
                    classification_basis: ClassificationBasis::FullSolveStats,
                    observed_classification: full_stats.puzzle_classification.coarse_label(),
                    observed_classification_detail: full_stats.puzzle_classification.detail_label(),
                    solution_count_observed: full_stats.solution_count,
                    times: full_times,
                },
            );
        }
    }

    raw_writer.flush().expect("raw benchmark CSV must flush");
    long_writer.flush().expect("long benchmark CSV must flush");

    println!(
        "Benchmark complete in {:.2}s. Wrote results/benchmark_timings.csv and results/benchmark_timings_long.csv",
        total_start.elapsed().as_secs_f32()
    );
}

fn parse_board(line: &str) -> (Board<N>, usize) {
    let parsed_cells = parse_puzzle_string(line).expect("benchmark puzzle format must be valid");
    let mut cells = [[0u8; N]; N];
    let mut clues_count = 0;

    for (idx, &val) in parsed_cells.iter().enumerate() {
        cells[idx / N][idx % N] = val;
        if val != 0 {
            clues_count += 1;
        }
    }

    (Board::<N>::new(cells), clues_count)
}

fn benchmark_full_solve(board: Board<N>) -> SolveReport<N> {
    SudokuSolver::<N, K>::new(board)
        .with_search_mode(SearchMode::EnumerateAll)
        .solve_with_stats()
}

fn puzzle_class_solution_count(classification: &PuzzleClass) -> usize {
    match classification {
        PuzzleClass::Unsolvable => 0,
        PuzzleClass::Unique => 1,
        PuzzleClass::Ambiguous(n) => *n,
    }
}

fn write_row(
    raw_writer: &mut Writer<std::fs::File>,
    long_writer: &mut Writer<std::fs::File>,
    meta: &RowMeta<'_>,
    outcome: RowOutcome<'_>,
) {
    let raw_row = RawBenchmarkRow {
        run_id: meta.run_id,
        dataset_file: meta.dataset_file,
        dataset_label: meta.dataset_label,
        difficulty: meta.difficulty,
        puzzle_index: meta.puzzle_index,
        puzzle: meta.puzzle,
        clues_count: meta.clues_count,
        benchmark_mode: outcome.benchmark_mode.as_str(),
        search_mode: search_mode_label(outcome.search_mode),
        is_locally_valid: meta.is_locally_valid,
        classification_basis: outcome.classification_basis.as_str(),
        observed_classification: outcome.observed_classification,
        observed_classification_detail: outcome.observed_classification_detail.as_str(),
        solution_count_observed: outcome.solution_count_observed,
        mask_init_ns: outcome.times.mask_init_ns,
        heuristic_ns: outcome.times.heuristic_ns,
        permutation_ns: outcome.times.permutation_ns,
        edge_build_ns: outcome.times.edge_build_ns,
        pruning_ns: outcome.times.pruning_ns,
        extraction_ns: outcome.times.extraction_ns,
        total_ns: outcome.times.total_ns,
    };
    raw_writer
        .serialize(raw_row)
        .expect("raw benchmark row must serialize");

    for (phase, time_ns) in outcome.times.phases() {
        if outcome.benchmark_mode != BenchmarkMode::FullSolveStats && phase != "total" {
            continue;
        }

        let long_row = LongBenchmarkRow {
            run_id: meta.run_id,
            dataset_file: meta.dataset_file,
            dataset_label: meta.dataset_label,
            difficulty: meta.difficulty,
            puzzle_index: meta.puzzle_index,
            puzzle: meta.puzzle,
            clues_count: meta.clues_count,
            benchmark_mode: outcome.benchmark_mode.as_str(),
            search_mode: search_mode_label(outcome.search_mode),
            is_locally_valid: meta.is_locally_valid,
            classification_basis: outcome.classification_basis.as_str(),
            observed_classification: outcome.observed_classification,
            observed_classification_detail: outcome.observed_classification_detail.as_str(),
            solution_count_observed: outcome.solution_count_observed,
            phase,
            time_ns,
            time_ms: time_ns as f64 / 1_000_000.0,
        };
        long_writer
            .serialize(long_row)
            .expect("long benchmark row must serialize");
    }
}

fn search_mode_label(search_mode: SearchMode) -> &'static str {
    match search_mode {
        SearchMode::First => "first",
        SearchMode::Classify => "classify",
        SearchMode::EnumerateAll => "enumerate_all",
        SearchMode::EnumerateUpTo(_) => "enumerate_up_to",
    }
}
