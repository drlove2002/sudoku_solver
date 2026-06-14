"""M1 milestone: render the board with allowed[r][c] pencil marks.

Standalone scene used for visual review of the pencil-mark layout before
we wire it into the full Phase 2 cascade.
"""

import sys
import os

sys.path.insert(
    0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
)

from manim import *

from animation.components.board import SudokuBoard
from animation.puzzle import DRY_RUN_PUZZLE, compute_allowed


class Phase2M1Pencils(Scene):
    def construct(self):
        title = Text("Phase 2 M1 — Pencil marks for allowed[r][c]", font_size=28, color=WHITE)
        title.to_edge(UP, buff=0.3)
        self.play(Write(title))

        board = SudokuBoard(DRY_RUN_PUZZLE, cell_size=0.50)
        board.move_to(ORIGIN + DOWN * 0.1)

        self.play(FadeIn(board), run_time=0.6)

        allowed = compute_allowed(DRY_RUN_PUZZLE)
        marks = board.set_pencil_marks_for_board(allowed, color=GREY)
        self.play(*[FadeIn(m, run_time=0.4) for m in marks], run_time=2.5)

        self.wait(1.5)
        self.play(FadeOut(board), FadeOut(title), *[FadeOut(m) for m in marks], run_time=1.0)
