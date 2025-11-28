__all__ = ["BitBoard"]


class BitBoard:
    """A class representing a Sudoku board using bit manipulation for efficient storage and access.
    Each cell in the board is represented by a bit in an integer, allowing for quick operations
    to set, clear, and check the state of each cell.

    Attributes:
        size (int): Size of the board (9 for standard Sudoku)
        board (list[list[int]]): Original 2D list representing the Sudoku board
        row (list[int]): Bitmasks for each row (which digits are used)
        column (list[int]): Bitmasks for each column (which digits are used)
        minigrid (list[int]): Bitmasks for each 3x3 minigrid (which digits are used)
    """

    def __init__(self, board: list[list[int]]):
        """Initialize BitBoard from a 2D list representation.

        Args:
            board (list[list[int]]): 9x9 Sudoku board (0 represents empty cell)
        """
        self.size = len(board)
        self.board = [row[:] for row in board]  # Deep copy

        # Initialize bitmasks: all zeros initially
        self.row = [0] * self.size  # 9 row masks
        self.column = [0] * self.size  # 9 column masks
        self.minigrid = [0] * self.size  # 9 minigrid masks (3x3 boxes)

        # Populate bitmasks from the initial board
        for r in range(self.size):
            for c in range(self.size):
                if board[r][c] != 0:
                    digit = board[r][c]
                    # Set bit for this digit in row, column, and minigrid
                    self.row[r] |= 1 << digit
                    self.column[c] |= 1 << digit

                    minigrid_idx = (r // 3) * 3 + (c // 3)
                    self.minigrid[minigrid_idx] |= 1 << digit

    def set_cell(self, row: int, col: int, digit: int) -> None:
        """Set a cell to a digit and update bitmasks.

        Args:
            row (int): Row index (0-8)
            col (int): Column index (0-8)
            digit (int): Digit to place (1-9)
        """
        self.board[row][col] = digit

        # Set bit in row, column, and minigrid
        self.row[row] |= 1 << digit
        self.column[col] |= 1 << digit

        minigrid_idx = (row // 3) * 3 + (col // 3)
        self.minigrid[minigrid_idx] |= 1 << digit

    def clear_cell(self, row: int, col: int, digit: int) -> None:
        """Clear a cell and update bitmasks.

        Args:
            row (int): Row index (0-8)
            col (int): Column index (0-8)
            digit (int): Digit that was placed (1-9)
        """
        self.board[row][col] = 0

        # Clear bit in row, column, and minigrid
        self.row[row] &= ~(1 << digit)
        self.column[col] &= ~(1 << digit)

        minigrid_idx = (row // 3) * 3 + (col // 3)
        self.minigrid[minigrid_idx] &= ~(1 << digit)

    def is_digit_available(self, row: int, col: int, digit: int) -> bool:
        """Check if a digit can be placed at (row, col).

        Args:
            row (int): Row index (0-8)
            col (int): Column index (0-8)
            digit (int): Digit to check (1-9)

        Returns:
            bool: True if digit is available, False otherwise
        """
        # Check if digit is already used in row, column, or minigrid
        row_conflict = (self.row[row] >> digit) & 1
        col_conflict = (self.column[col] >> digit) & 1

        minigrid_idx = (row // 3) * 3 + (col // 3)
        box_conflict = (self.minigrid[minigrid_idx] >> digit) & 1

        return not (row_conflict or col_conflict or box_conflict)

    def get_available_digits(self, row: int, col: int) -> set:
        """Get all available digits for a cell.

        Args:
            row (int): Row index (0-8)
            col (int): Column index (0-8)

        Returns:
            set: Set of available digits (1-9)
        """
        available = set()
        for digit in range(1, 10):
            if self.is_digit_available(row, col, digit):
                available.add(digit)
        return available

    def get_minigrid_empty_cells(self, minigrid_idx: int) -> list:
        """Get all empty cell positions in a minigrid.

        Args:
            minigrid_idx (int): Minigrid index (0-8)

        Returns:
            list: List of (row, col) tuples for empty cells in minigrid
        """
        mini_row_start = (minigrid_idx // 3) * 3
        mini_col_start = (minigrid_idx % 3) * 3

        empty_cells = []
        for r in range(mini_row_start, mini_row_start + 3):
            for c in range(mini_col_start, mini_col_start + 3):
                if self.board[r][c] == 0:
                    empty_cells.append((r, c))

        return empty_cells

    def get_minigrid_available_digits(self, minigrid_idx: int) -> set:
        """Get digits that haven't been used in a minigrid yet.

        Args:
            minigrid_idx (int): Minigrid index (0-8)

        Returns:
            set: Set of unused digits (1-9)
        """
        used = set()
        for digit in range(1, 10):
            if (self.minigrid[minigrid_idx] >> digit) & 1:
                used.add(digit)

        return set(range(1, 10)) - used

    def __str__(self) -> str:
        """String representation of the board."""
        result = []
        for i, row in enumerate(self.board):
            if i % 3 == 0 and i != 0:
                result.append("------+-------+------")

            row_str = " ".join(str(x) if x != 0 else "." for x in row[:3])
            row_str += " | " + " ".join(str(x) if x != 0 else "." for x in row[3:6])
            row_str += " | " + " ".join(str(x) if x != 0 else "." for x in row[6:9])
            result.append(row_str)

        return "\n".join(result)
