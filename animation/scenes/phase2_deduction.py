"""
Phase 2: Deterministic Deduction

Applies Naked Singles and Hidden Singles in a loop until convergence.
Shows the board progressively filling in as each technique fires.

Visual flow:
1. Show board after Phase 1
2. Iteration loop:
   a. Sweep for Naked Singles — highlight cell, show it has only 1 candidate
   b. Fill Naked Singles, update board
   c. Sweep for Hidden Singles — show digit has only 1 legal position
   d. Fill Hidden Singles, update board
   e. If no changes, stop
3. Show final board state with 17 cells filled
"""

import sys, os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from manim import *
from animation.puzzle import DRY_RUN_PUZZLE, N, K, box_idx
from animation.components.board import SudokuBoard


class Phase2Deduction(Scene):
    def construct(self):
        title = Text("Phase 2: Deterministic Deduction", font_size=36, color=WHITE)
        title.to_edge(UP)
        self.play(Write(title))
        self.wait(0.5)

        # Start with the puzzle board
        board = SudokuBoard(DRY_RUN_PUZZLE, cell_size=0.5)
        board.shift(DOWN * 0.3)
        self.play(FadeIn(board))

        # Placeholder — will be fully implemented after Phase 1 is done
        # For now just show the structure
        placeholder = Text(
            "Naked Singles → Hidden Singles → loop until fixed point",
            font_size=20, color=GREY
        ).next_to(board, DOWN, buff=0.5)
        self.play(Write(placeholder))
        self.wait(2)

        self.play(FadeOut(board), FadeOut(placeholder), FadeOut(title))
