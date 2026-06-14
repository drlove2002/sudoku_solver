"""
Phase 6: Global Selection & Backtracking Search

Shows MRV-based minigrid selection, constraint propagation, and the final
solution being assembled on the board.

Visual flow:
1. Show pruned graph
2. Select most constrained minigrid (MRV), assign a permutation
3. Propagate: domains of related minigrids shrink, incompatible permutations fade
4. Continue until all minigrids assigned → solution found
5. Reconstruct final board from the assignment
6. Fast-forward through remaining branches (if ambiguous)
"""

import sys, os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from manim import *
from animation.puzzle import DRY_RUN_PUZZLE, DRY_RUN_SOLUTION, N, K
from animation.components.board import SudokuBoard


class Phase6Extraction(Scene):
    def construct(self):
        title = Text("Phase 6: Global Selection & Backtracking Search", font_size=36, color=WHITE)
        title.to_edge(UP)
        self.play(Write(title))
        self.wait(0.5)

        # Show initial board
        board = SudokuBoard(DRY_RUN_PUZZLE, cell_size=0.5)
        board.shift(LEFT * 2.0)
        self.play(FadeIn(board))

        # Placeholder
        placeholder = Text(
            "MRV-based search → constraint propagation → solution found\n"
            "Board reconstruction from permutation assignment",
            font_size=20, color=GREY
        ).next_to(board, RIGHT, buff=0.5)
        self.play(Write(placeholder))
        self.wait(2)

        # Show final solution
        self.play(FadeOut(placeholder))
        final_text = Text("Unique Solution:", font_size=24, color=GREEN)
        final_text.next_to(board, RIGHT, buff=0.5)
        self.play(Write(final_text))

        # Fill the solution board
        for r in range(N):
            for c in range(N):
                if DRY_RUN_PUZZLE[r][c] == 0:
                    val = DRY_RUN_SOLUTION[r][c]
                    self.play(board.fill_cell(r, c, val, is_given=False), run_time=0.05)

        self.wait(2)
        self.play(FadeOut(board), FadeOut(final_text), FadeOut(title))
