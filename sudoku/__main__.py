from board import BitBoard

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


def main():
    print("Initializing Sudoku Board...")
    sudoku = BitBoard(PROBLEM)
    print("Initial board:")
    print(sudoku)


if __name__ == "__main__":
    main()
