"""
Phase 2: Deterministic Deduction — Naked/Hidden Singles

Shows the constraint-propagation loop that fills cells without search:
1. Hidden Singles: a digit MUST go in exactly one cell of a row/col/box
2. Naked Singles: a cell has only ONE allowed digit remaining
3. These feed each other in a loop until quiescence (12 cells filled)
"""

import sys, os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from manim import *
from animation.puzzle import DRY_RUN_PUZZLE, N, K, box_idx
from animation.components.board import SudokuBoard, HIGHLIGHT_CELL, GIVEN_COLOR, PLACED_COLOR


# Illustrative fill order showing hidden → naked singles cascade
# Pre-computed from the constraint-propagation trace
FILL_SEQUENCE = [
    # Wave 1: Hidden singles from initial state
    (2, 5, 5, "Hidden Single (row 2)"),
    (3, 7, 6, "Hidden Single (row 3)"),
    (5, 6, 5, "Hidden Single (row 5)"),
    (5, 1, 8, "Hidden Single (col 1)"),
    (0, 2, 5, "Hidden Single (box 0)"),
    # Wave 2: Naked singles triggered by Wave 1
    (0, 6, 3, "Naked Single"),
    (1, 5, 3, "Naked Single"),
    (1, 6, 8, "Naked Single"),
    (3, 5, 2, "Naked Single"),
    (4, 4, 3, "Naked Single"),
    (6, 7, 5, "Naked Single"),
    (8, 4, 5, "Naked Single"),
]


class Phase2Deduction(Scene):
    def construct(self):
        # Title
        title = Text("Phase 2: Deterministic Deduction", font_size=36, color=WHITE)
        title.to_edge(UP)
        self.play(Write(title))
        self.wait(0.3)

        # Subtitle
        subtitle = Text(
            "Naked Singles + Hidden Singles → fill cells without search",
            font_size=20, color=GREY,
        ).next_to(title, DOWN, buff=0.15)
        self.play(Write(subtitle))

        # Board
        board = SudokuBoard(DRY_RUN_PUZZLE, cell_size=0.48)
        board.move_to(ORIGIN + DOWN * 0.3)
        self.play(FadeIn(board), run_time=0.8)
        self.wait(0.5)

        # ---- Wave 1: Hidden Singles ----
        wave_label = Text("Hidden Singles", font_size=28, color=YELLOW)
        wave_label.to_edge(LEFT, buff=0.4)
        self.play(Write(wave_label))

        for r, c, d, label_text in FILL_SEQUENCE[:5]:
            # Show the reason
            reason = Text(label_text, font_size=18, color=YELLOW)
            reason.next_to(board, UP, buff=0.8).shift(RIGHT * 2)
            self.play(Write(reason))

            # Highlight the cell
            hl = board.make_highlight(r, c, color=YELLOW)
            self.add(hl)
            self.wait(0.2)

            # Place the digit
            self.play(board.fill_cell(r, c, d, is_given=False), run_time=0.4)
            self.wait(0.15)

            self.remove(hl)
            self.play(FadeOut(reason))

        # ---- Wave 2: Naked Singles ----
        self.play(FadeOut(wave_label))
        wave2_label = Text("Naked Singles", font_size=28, color=BLUE)
        wave2_label.to_edge(LEFT, buff=0.4)
        self.play(Write(wave2_label))

        # Show all remaining fills quicker
        remaining = FILL_SEQUENCE[5:]
        for i, (r, c, d, _) in enumerate(remaining):
            hl = board.make_highlight(r, c, color=BLUE)
            self.add(hl)
            self.play(board.fill_cell(r, c, d, is_given=False), run_time=0.25)
            self.remove(hl)
            if i == 2:
                # Show a brief explanation mid-wave
                explain = Text(
                    "After hidden singles fill cells,\n"
                    "some cells reduce to exactly 1 candidate",
                    font_size=16, color=BLUE,
                ).next_to(board, UP, buff=0.8).shift(RIGHT * 2)
                self.play(Write(explain))
                self.wait(0.8)
                self.play(FadeOut(explain))

        self.play(FadeOut(wave2_label))

        # ---- Finale ----
        counter = Text("12 cells filled — 52 empty cells remain", font_size=24, color=GREEN)
        counter.next_to(board, UP, buff=0.5)
        self.play(Write(counter))

        # Brief loop between hidden and naked singles
        loop_label = Text(
            "Loop until quiescence: Hidden → Naked → Hidden → ...",
            font_size=18, color=GREY,
        ).next_to(counter, DOWN, buff=0.15)
        self.play(Write(loop_label))
        self.wait(2)

        self.play(FadeOut(loop_label), FadeOut(counter), FadeOut(board),
                  FadeOut(subtitle), FadeOut(title))
        self.wait(0.3)
