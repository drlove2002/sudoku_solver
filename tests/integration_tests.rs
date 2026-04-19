use solver::{SudokuSolver, dataset_parser::parse_puzzle_string, types::Board};

const N: usize = 9;
const K: usize = 3;

fn board_from_kaggle_line(puzzle: &str) -> Board<N> {
    let digits = parse_puzzle_string(puzzle).expect("valid puzzle string");
    let mut cells = [[0u8; N]; N];

    for (idx, value) in digits.into_iter().enumerate() {
        cells[idx / N][idx % N] = value;
    }

    Board::new(cells)
}

fn first_puzzle(path: &str) -> String {
    std::fs::read_to_string(path)
        .expect("sample file exists")
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("sample file contains at least one puzzle")
        .to_owned()
}

fn assert_solver_finds_valid_solution(sample_path: &str) {
    let board = board_from_kaggle_line(&first_puzzle(sample_path));
    let solver = SudokuSolver::<N, K>::new(board);
    let report = solver.solve_with_stats();

    assert!(
        !report.solutions.is_empty(),
        "solver returned no solutions for {sample_path}"
    );
    assert_eq!(report.stats.solution_count, report.solutions.len());
    assert!(report.stats.initial_vertex_count >= report.stats.pruned_vertex_count);

    for solution in &report.solutions {
        assert!(solution.board.is_valid(), "solver returned invalid board");
    }
}

#[test]
fn solves_first_easy_sample() {
    assert_solver_finds_valid_solution("data/sample_easy.txt");
}

#[test]
fn solves_first_medium_sample() {
    assert_solver_finds_valid_solution("data/sample_medium.txt");
}

#[test]
#[ignore = "hard puzzles are substantially slower with exact support pruning"]
fn solves_first_hard_sample() {
    assert_solver_finds_valid_solution("data/sample_hard.txt");
}
