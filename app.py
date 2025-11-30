import sudoku_solver

PROBLEM: list[list[int]] = [
    [7, 4, 5, 0, 0, 9, 0, 0, 0],
    [0, 3, 2, 1, 5, 0, 0, 4, 6],
    [0, 0, 0, 2, 8, 0, 5, 0, 3],
    [2, 0, 0, 0, 0, 0, 0, 0, 0],
    [9, 8, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 5, 4, 0, 0, 0, 0],
    [3, 0, 8, 0, 0, 0, 0, 0, 2],
    [0, 2, 0, 7, 6, 0, 0, 0, 0],
    [0, 6, 0, 0, 0, 0, 3, 4, 0],
]

grid_size = 9

board_list = [n for row in PROBLEM for n in row]

print("Solving the following Sudoku puzzle:")
solver = sudoku_solver.SudokuSolver(grid_size * grid_size, board_list)

solutions = solver.solve()

print(f"Found {len(solutions)} solution(s).")
