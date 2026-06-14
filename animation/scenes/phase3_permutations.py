"""
Phase 3: Sub-Grid Permutation Generation

Runs DFS with MRV heuristic inside a single minigrid to generate all
locally valid completions. Shows the search tree growing with backtracking.

Visual flow:
1. Zoom into minigrid 1 (top-left 3x3)
2. Show the DFS tree: nodes are partial assignments, edges are digit choices
3. Dead ends are shown with red X, backtrack with curved arrow
4. Complete permutations are saved and displayed
5. Fast-forward: show remaining 8 minigrids completing quickly
"""

import sys, os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from manim import *
from animation.puzzle import DRY_RUN_PUZZLE, N, K
from animation.components.board import SudokuBoard


class Phase3Permutations(Scene):
    def construct(self):
        title = Text("Phase 3: Permutation Generation", font_size=36, color=WHITE)
        title.to_edge(UP)
        self.play(Write(title))
        self.wait(0.5)

        board = SudokuBoard(DRY_RUN_PUZZLE, cell_size=0.5)
        board.shift(LEFT * 2.0)
        self.play(FadeIn(board))

        # Placeholder
        placeholder = Text(
            "DFS + MRV inside minigrid 1 → 5 valid permutations\n"
            "Then: fast-forward through minigrids 2-9",
            font_size=20, color=GREY
        ).next_to(board, RIGHT, buff=0.5)
        self.play(Write(placeholder))
        self.wait(2)

        self.play(FadeOut(board), FadeOut(placeholder), FadeOut(title))
