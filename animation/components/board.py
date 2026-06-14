"""
Shared sudoku board rendering component for all animation phases.

Renders a 9x9 grid with:
- Thin cell borders
- Thick 3x3 minigrid borders
- Given digits in one style, placed digits in another
- Optional cell highlighting (color fills)
- Optional minigrid highlight outlines
"""

from manim import *
import numpy as np

from animation.puzzle import K, N, box_idx

# Colors
GIVEN_COLOR = WHITE
PLACED_COLOR = BLUE
GRID_COLOR = GREY
MINIGRID_BORDER_COLOR = WHITE
HIGHLIGHT_CELL = YELLOW
HIGHLIGHT_MINIGRID = GREEN
CONFLICT_CELL = RED
CANDIDATE_COLOR = GRAY


class SudokuBoard(VGroup):
    """A 9x9 Sudoku grid with digits.

    Usage:
        board = SudokuBoard(puzzle_matrix)
        self.play(FadeIn(board))
        board.highlight_cell(0, 3)  # highlight row 0, col 3
        board.fill_cell(0, 3, 5, is_given=True)  # place digit 5
    """

    def __init__(self, cells: list, cell_size: float = 0.6, **kwargs):
        super().__init__(**kwargs)
        self.cell_size = cell_size
        self.N = 9
        self.K = 3

        # Active cell values (0 = empty)
        self.cells = [list(row) for row in cells]

        # Visual objects
        self.grid_lines = VGroup()
        self.minigrid_outlines = VGroup()
        self.digit_tex: dict[tuple[int, int], VMobject] = {}

        self._build_grid()
        self._build_digits()

    def _build_grid(self):
        """Create the grid lines and minigrid borders."""
        total = self.cell_size * self.N
        half = total / 2

        # Cell grid lines (thin)
        for i in range(self.N + 1):
            stroke = 1.0 if i % self.K == 0 else 0.5
            color = MINIGRID_BORDER_COLOR if i % self.K == 0 else GRID_COLOR
            # Horizontal
            y = half - i * self.cell_size
            line = Line(
                LEFT * half, RIGHT * half,
                stroke_width=stroke, color=color
            ).shift(UP * y)
            self.grid_lines.add(line)
            # Vertical
            x = -half + i * self.cell_size
            line = Line(
                UP * half, DOWN * half,
                stroke_width=stroke, color=color
            ).shift(RIGHT * x)
            self.grid_lines.add(line)

        self.add(self.grid_lines)

    def _build_digits(self):
        """Place initial digits on the board."""
        for r in range(self.N):
            for c in range(self.N):
                val = self.cells[r][c]
                if val != 0:
                    self._set_digit(r, c, val, is_given=True)

    def cell_center(self, r: int, c: int) -> np.ndarray:
        """Return the center point of cell (r, c) in board-local coordinates."""
        half = self.cell_size * self.N / 2
        x = -half + c * self.cell_size + self.cell_size / 2
        y = half - r * self.cell_size - self.cell_size / 2
        return np.array([x, y, 0])

    def cell_center_global(self, r: int, c: int) -> np.ndarray:
        """Return the center point of cell (r, c) in scene-global coordinates."""
        return self.cell_center(r, c) + self.get_center()

    def _set_digit(self, r: int, c: int, value: int, is_given: bool = False):
        """Place or update a digit at cell (r, c)."""
        if (r, c) in self.digit_tex:
            self.digit_tex[(r, c)].remove(self)
        color = GIVEN_COLOR if is_given else PLACED_COLOR
        tex = Tex(str(value), color=color, font_size=int(self.cell_size * 45))
        tex.move_to(self.cell_center(r, c))
        self.digit_tex[(r, c)] = tex
        self.add(tex)
        self.cells[r][c] = value

    def make_highlight(self, r: int, c: int, color=HIGHLIGHT_CELL, opacity=0.3) -> Rectangle:
        """Create a highlight rectangle for cell (r, c) in scene-global coords.

        Returns the Rectangle. The caller is responsible for adding it to the
        scene and removing it. This avoids VGroup-child transform issues.
        """
        rect = Rectangle(
            width=self.cell_size, height=self.cell_size,
            fill_color=color, fill_opacity=opacity,
            stroke_width=0,
        )
        rect.move_to(self.cell_center_global(r, c))
        return rect

    def fill_cell(self, r: int, c: int, value: int, is_given: bool = False):
        """Animate a digit appearing in cell (r, c). Returns the animation."""
        color = GIVEN_COLOR if is_given else PLACED_COLOR
        tex = Tex(str(value), color=color, font_size=int(self.cell_size * 45))
        tex.move_to(self.cell_center(r, c))

        old_tex = self.digit_tex.pop((r, c), None)
        self.digit_tex[(r, c)] = tex
        self.cells[r][c] = value

        if old_tex:
            return Transform(old_tex, tex)
        else:
            return Write(tex)
