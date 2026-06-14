"""M3 milestone: Wave A — play fills 1-5 in sequence.

Each fill follows the same template:
  1. Yellow house highlight (row / col / box)
  2. Brief "Hidden Single (X)" label
  3. Erase the digit from all peer cells in that house
  4. Clear pencil marks in the filled cell
  5. Place the digit

A small "Filled: N" counter ticks up on the right after each fill.
"""

import sys
import os

sys.path.insert(
    0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
)

from manim import *

from animation.components.board import SudokuBoard
from animation.puzzle import DRY_RUN_PUZZLE, compute_allowed, box_idx, K, N


HIDDEN_COLOR = YELLOW


# Pre-computed fill list (5 hidden singles) — house attribution from the
# Python simulator (matches Rust's queue priority row > col > box).
WAVE_A_FILLS = [
    # (r, c, d, kind, house_type, house_idx)
    (2, 5, 5, "hidden", "row", 2),
    (3, 7, 6, "hidden", "row", 3),
    (5, 6, 5, "hidden", "row", 5),
    (5, 1, 8, "hidden", "col", 1),
    (0, 2, 5, "hidden", "row", 0),
]


def make_house_highlight(board: SudokuBoard, r: int, c: int, d: int,
                          house_type: str, house_idx: int) -> Rectangle:
    """Return a translucent rectangle highlighting the house that enforced `d` at (r, c)."""
    cell_size = board.cell_size
    if house_type == "row":
        rect = Rectangle(
            width=cell_size * N,
            height=cell_size,
            fill_color=HIDDEN_COLOR, fill_opacity=0.18,
            stroke_color=HIDDEN_COLOR, stroke_width=1.5,
        )
        rect.move_to(board.cell_center_global(r, c))
    elif house_type == "col":
        rect = Rectangle(
            width=cell_size,
            height=cell_size * N,
            fill_color=HIDDEN_COLOR, fill_opacity=0.18,
            stroke_color=HIDDEN_COLOR, stroke_width=1.5,
        )
        rect.move_to(board.cell_center_global(r, c))
    else:  # box
        b = house_idx
        base_r = (b // K) * K
        base_c = (b % K) * K
        center = board.cell_center_global(base_r + 1, base_c + 1)
        rect = Rectangle(
            width=cell_size * K,
            height=cell_size * K,
            fill_color=HIDDEN_COLOR, fill_opacity=0.18,
            stroke_color=HIDDEN_COLOR, stroke_width=1.5,
        )
        rect.move_to(center)
    return rect


def erase_digit_from_peers(
    scene: Scene,
    board: SudokuBoard,
    r: int, c: int, d: int,
    house_type: str,
):
    """Animate FadeOut for the digit `d` pencil mark in every peer cell of (r, c)
    that has it. Also pops the marks out of `board.pencil_tex` so future queries
    don't see them.

    Returns the list of FadeOut animations (or empty list).
    """
    anims = []
    # Determine which cells are in the same house
    b = box_idx(r, c)
    peer_cells: list[tuple[int, int]] = []
    if house_type == "row":
        peer_cells = [(r, cc) for cc in range(N) if cc != c]
    elif house_type == "col":
        peer_cells = [(rr, c) for rr in range(N) if rr != r]
    else:  # box
        base_r = (b // K) * K
        base_c = (b % K) * K
        peer_cells = [
            (base_r + dr, base_c + dc)
            for dr in range(K) for dc in range(K)
            if (base_r + dr, base_c + dc) != (r, c)
        ]
    for (rr, cc) in peer_cells:
        tex = board.get_pencil_mark(rr, cc, d)
        if tex is not None:
            anims.append(FadeOut(tex, run_time=0.18))
            board.pencil_tex.pop((rr, cc, d), None)
    return anims


class Phase2M3WaveA(Scene):
    def construct(self):
        title = Text("Phase 2 M3 — Wave A: 5 hidden singles", font_size=28, color=WHITE)
        title.to_edge(UP, buff=0.3)
        self.play(Write(title))

        # Counter
        counter = Text("Filled: 0", font_size=22, color=GREY)
        counter.to_edge(RIGHT, buff=0.5).shift(UP * 2.5)
        self.play(Write(counter))

        # Board
        board = SudokuBoard(DRY_RUN_PUZZLE, cell_size=0.50)
        board.move_to(LEFT * 0.3 + DOWN * 0.1)
        self.play(FadeIn(board), run_time=0.6)

        # Pencil marks
        allowed = compute_allowed(DRY_RUN_PUZZLE)
        marks = board.set_pencil_marks_for_board(allowed, color=GREY)
        self.play(*[FadeIn(m, run_time=0.2) for m in marks], run_time=1.5)
        self.wait(0.3)

        for i, (r, c, d, kind, house_type, house_idx) in enumerate(WAVE_A_FILLS, 1):
            label_text = f"Hidden Single ({house_type} {house_idx}) → ({r},{c})={d}"
            label = Text(label_text, font_size=18, color=HIDDEN_COLOR)
            label.to_edge(DOWN, buff=0.4)

            house_hl = make_house_highlight(board, r, c, d, house_type, house_idx)

            self.play(FadeIn(house_hl), Write(label), run_time=0.35)
            self.wait(0.15)

            # Erase digit from peer cells
            erase_anims = erase_digit_from_peers(self, board, r, c, d, house_type)
            if erase_anims:
                self.play(*erase_anims, run_time=0.5)
            self.wait(0.1)

            # Clear pencil marks in the filled cell, then place digit
            cell_marks = board.clear_pencil_marks(r, c)
            clear_anims = [FadeOut(m) for m in cell_marks]
            if clear_anims:
                self.play(*clear_anims, run_time=0.15)
            self.play(board.fill_cell(r, c, d, is_given=False), run_time=0.35)

            # Tick counter
            new_counter = Text(f"Filled: {i}", font_size=22, color=GREEN).to_edge(RIGHT, buff=0.5).shift(UP * 2.5)
            self.play(Transform(counter, new_counter), run_time=0.2)

            self.play(FadeOut(house_hl), FadeOut(label), run_time=0.25)
            self.wait(0.1)

        self.wait(1.0)
        self.play(
            FadeOut(board), FadeOut(counter), FadeOut(title),
            run_time=1.0,
        )
