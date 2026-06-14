"""
Shared sudoku board rendering component for all animation phases.

Renders a 9x9 grid with:
- Thin cell borders
- Thick 3x3 minigrid borders
- Given digits in one style, placed digits in another
- Optional cell highlighting (color fills)
- Optional minigrid highlight outlines
- Optional pencil marks (allowed candidates) per empty cell
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


def candidate_pos(center: np.ndarray, cell_size: float, idx: int) -> np.ndarray:
    """Return the position for the idx-th pencil mark in a 3×3 grid of marks.

    `idx` ranges 0..8, laid out row-major in a 3×3 micro-grid centered on the
    cell. The offset scales with cell_size so marks stay readable across cell
    sizes. Marks that overflow the candidate count leave gaps, so the layout
    always reads in numeric order.
    """
    return center + np.array(
        [
            (idx % 3 - 1) * cell_size * 0.32,
            (idx // 3 - 1) * cell_size * 0.32,
            0,
        ]
    )



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
        # Pencil marks: (r, c, d) -> Tex, kept so we can animate erasure.
        self.pencil_tex: dict[tuple[int, int, int], VMobject] = {}

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

    # ────────────────────────────────────────────────────
    # Pencil marks (allowed candidates)
    # ────────────────────────────────────────────────────

    def set_pencil_marks(
        self,
        r: int,
        c: int,
        digits: list[int],
        color=GREY,
        font_size: int | None = None,
    ) -> list[VMobject]:
        """Place pencil marks for `digits` (1..=9) in cell (r, c). Returns the list of Tex objects.

        Replaces any existing pencil marks for that cell. Use `digits=[]` to clear.
        """
        self.clear_pencil_marks(r, c)
        if not digits:
            return []
        if font_size is None:
            font_size = max(int(self.cell_size * 20), 10)
        center = self.cell_center(r, c)
        new_marks: list[VMobject] = []
        for i, d in enumerate(sorted(digits)):
            tex = Tex(str(d), color=color, font_size=font_size)
            tex.move_to(candidate_pos(center, self.cell_size, i))
            self.pencil_tex[(r, c, d)] = tex
            self.add(tex)
            new_marks.append(tex)
        return new_marks

    def clear_pencil_marks(self, r: int, c: int) -> list[VMobject]:
        """Remove every pencil mark from cell (r, c). Returns the removed Tex objects."""
        removed: list[VMobject] = []
        keys = [k for k in self.pencil_tex if k[0] == r and k[1] == c]
        for k in keys:
            tex = self.pencil_tex.pop(k)
            tex.remove(self)
            removed.append(tex)
        return removed

    def get_pencil_mark(self, r: int, c: int, d: int) -> VMobject | None:
        return self.pencil_tex.get((r, c, d))

    def set_pencil_marks_for_board(
        self,
        allowed: list[list[int]],
        color=GREY,
        font_size: int | None = None,
    ) -> VGroup:
        """Place pencil marks for every empty cell from an allowed[r][c] bitmask grid.

        Returns a VGroup of every Tex added (handy for batched animations).
        Cells with `allowed[r][c] == 0` (givens) get no marks.
        """
        all_marks: list[VMobject] = []
        for r in range(self.N):
            for c in range(self.N):
                if self.cells[r][c] != 0:
                    continue
                mask = allowed[r][c]
                if mask == 0:
                    continue
                digits = [d + 1 for d in range(self.N) if mask & (1 << d)]
                all_marks.extend(self.set_pencil_marks(r, c, digits, color=color, font_size=font_size))
        return VGroup(*all_marks)

    def peer_cells(self, r: int, c: int, house: str) -> list[tuple[int, int]]:
        """Return the cells that share a row/col/box with (r, c), excluding (r, c).

        `house` is one of: "row", "col", "box", "row+col+box" (all 20 peer cells).
        """
        out: list[tuple[int, int]] = []
        b = box_idx(r, c)
        if "row" in house:
            out.extend((r, cc) for cc in range(self.N) if cc != c)
        if "col" in house:
            out.extend((rr, c) for rr in range(self.N) if rr != r)
        if "box" in house:
            base_r = (b // K) * K
            base_c = (b % K) * K
            for dr in range(K):
                for dc in range(K):
                    rr, cc = base_r + dr, base_c + dc
                    if (rr, cc) == (r, c):
                        continue
                    out.append((rr, cc))
        # Dedup while preserving order
        seen = set(); deduped = []
        for cell in out:
            if cell not in seen:
                seen.add(cell); deduped.append(cell)
        return deduped
