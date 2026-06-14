"""M2 milestone: single fill animation for one hidden single.

Walks through fill #1 = (2,5)=5 with the full treatment:
  1. Board with pencil marks
  2. Highlight the house (row 2)
  3. Show "Hidden Single (row 2)" label
  4. Erase digit 5 from all peer cells in row 2
  5. Place digit 5 at (2,5), clear its pencil marks
"""

import sys
import os

sys.path.insert(
    0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
)

from manim import *

from animation.components.board import SudokuBoard
from animation.puzzle import DRY_RUN_PUZZLE, compute_allowed


HIDDEN_COLOR = YELLOW
NAKED_COLOR = BLUE
PAIR_COLOR = ORANGE


class Phase2M2Fill(Scene):
    def construct(self):
        title = Text("Phase 2 M2 — Single fill animation", font_size=28, color=WHITE)
        title.to_edge(UP, buff=0.3)
        self.play(Write(title))

        board = SudokuBoard(DRY_RUN_PUZZLE, cell_size=0.50)
        board.move_to(ORIGIN + DOWN * 0.1)
        self.play(FadeIn(board), run_time=0.6)

        # Initialize pencil marks from allowed[r][c]
        allowed = compute_allowed(DRY_RUN_PUZZLE)
        marks = board.set_pencil_marks_for_board(allowed, color=GREY)
        self.play(*[FadeIn(m, run_time=0.3) for m in marks], run_time=2.0)
        self.wait(0.5)

        # ── Fill 1: (2,5)=5, hidden single from row 2 ──
        r, c, d = 2, 5, 5
        house = "row"

        # 1) Highlight the house — row 2 yellow stripe
        # Use a translucent rectangle spanning row 2
        from animation.puzzle import K
        cell_size = board.cell_size
        N = board.N
        half = cell_size * N / 2
        row_y_top = half - r * cell_size
        row_y_bot = half - (r + 1) * cell_size
        row_strip = Rectangle(
            width=cell_size * N,
            height=cell_size,
            fill_color=HIDDEN_COLOR, fill_opacity=0.18,
            stroke_color=HIDDEN_COLOR, stroke_width=1.5,
        )
        row_strip.move_to(board.cell_center_global(r, c))  # center on row
        # Use the cell center y for the strip
        row_strip_y = (row_y_top + row_y_bot) / 2
        row_strip.move_to(np.array([board.get_center()[0], board.get_center()[1] + row_strip_y, 0]))
        self.play(FadeIn(row_strip), run_time=0.4)
        self.wait(0.3)

        # 2) Label
        label = Text(
            f"Hidden Single (row {r}) — digit {d} can only go in (2,5)",
            font_size=18, color=HIDDEN_COLOR,
        )
        label.to_edge(DOWN, buff=0.4)
        self.play(Write(label))
        self.wait(0.5)

        # 3) Erase digit 5 from peer cells in row 2
        # Walk through cells (2, 0..8) except (2, 5). For each cell that has
        # the "5" pencil mark, FadeOut it.
        erase_anims = []
        for cc in range(N):
            if cc == c:
                continue
            tex = board.get_pencil_mark(r, cc, d)
            if tex is not None:
                erase_anims.append(FadeOut(tex, run_time=0.25))
        if erase_anims:
            self.play(*erase_anims, run_time=1.2)
        # Mark these as gone in the data structure
        for cc in range(N):
            if cc == c:
                continue
            if board.get_pencil_mark(r, cc, d) is not None:
                board.pencil_tex.pop((r, cc, d), None)
        self.wait(0.3)

        # 4) Place digit 5 at (2,5) and clear its remaining pencil marks
        cell_marks_to_clear = board.clear_pencil_marks(r, c)
        self.play(*[FadeOut(m) for m in cell_marks_to_clear], run_time=0.3)
        self.play(board.fill_cell(r, c, d, is_given=False), run_time=0.5)
        self.wait(0.5)

        # 5) Cleanup
        self.play(
            FadeOut(row_strip), FadeOut(label), FadeOut(board),
            FadeOut(title), run_time=1.0,
        )
