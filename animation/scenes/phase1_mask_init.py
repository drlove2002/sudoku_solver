"""
Phase 1: Mask Initialization

Scans the input board cell-by-cell in row-major order. For each clue digit found,
three bitmasks are updated simultaneously:
  - Row mask (rows[r]): which digits appear in row r
  - Column mask (cols[c]): which digits appear in column c
  - Box mask (boxes[b]): which digits appear in minigrid b

After masks are built, the scene demonstrates conflict[r][c] computation:
  1. Pick an empty cell
  2. Highlight the three mask rows that intersect at that cell (r, c, b)
  3. Show the bitwise OR: conflict = rows[r] | cols[c] | boxes[b]
  4. Animate allowed digits appearing as pencil marks in that cell
  5. Then reveal all candidates across the board
"""

import os
import sys

sys.path.insert(
    0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
)

from manim import *

from animation.components.bitmask import MaskPanel
from animation.components.board import SudokuBoard, candidate_pos
from animation.puzzle import DRY_RUN_PUZZLE, K, N, box_idx


class Phase1MaskInit(Scene):
    def construct(self):
        title = Text("Phase 1: Mask Initialization", font_size=34, color=WHITE)
        title.to_edge(UP, buff=0.3)
        self.play(Write(title))

        # ── Board ──────────────────────────────────────────
        board = SudokuBoard(DRY_RUN_PUZZLE, cell_size=0.40)

        # ── Three bitmask panels ───────────────────────────
        row_masks = MaskPanel("rows[r]")
        col_masks = MaskPanel("cols[c]")
        box_masks = MaskPanel("boxes[b]")

        masks_group = VGroup(row_masks, col_masks, box_masks)
        masks_group.arrange(RIGHT, buff=0.35)

        # ════════════════════════════════════════════════════
        # LAYOUT: all positioning before dashboard
        # ════════════════════════════════════════════════════
        board.move_to(ORIGIN)
        for r in range(N):
            target_y = board.cell_center_global(r, 0)[1]
            for panel in [row_masks, col_masks, box_masks]:
                panel.rows[r].shift(UP * (target_y - panel.rows[r].get_center()[1]))

        for panel in [row_masks, col_masks, box_masks]:
            panel.relayout()

        dashboard = VGroup(board, masks_group)
        dashboard.arrange(RIGHT, buff=0.8)
        dashboard.move_to(ORIGIN + DOWN * 0.2)

        self.play(FadeIn(board))
        self.play(FadeIn(row_masks), FadeIn(col_masks), FadeIn(box_masks), run_time=0.8)

        # ── State: integer masks matching rust code ────────
        rows_int = [0] * N
        cols_int = [0] * N
        boxes_int = [0] * N
        clue_count = 0

        # ── Cursor ─────────────────────────────────────────
        cursor = Dot(radius=0.06, color=YELLOW)
        cursor.move_to(board.cell_center_global(0, 0))
        self.add(cursor)

        subtitle = Text(
            "Scanning board row-by-row...", font_size=20, color=GREY
        ).next_to(title, DOWN, buff=0.2)
        self.play(Write(subtitle))

        # ── Cell-by-cell scan ──────────────────────────────
        for r in range(N):
            for c in range(N):
                d = DRY_RUN_PUZZLE[r][c]
                target = board.cell_center_global(r, c)
                self.play(cursor.animate.move_to(target), run_time=0.01)
                if d == 0:
                    continue

                b = box_idx(r, c)
                clue_count += 1
                bit = 1 << (d - 1)

                hl = board.make_highlight(r, c, color=BLUE, opacity=0.4)
                self.add(hl)

                already_in_row = rows_int[r] & bit
                already_in_col = cols_int[c] & bit
                already_in_box = boxes_int[b] & bit
                rows_int[r] |= bit
                cols_int[c] |= bit
                boxes_int[b] |= bit

                anims = []
                guides = []
                cell_right = (
                    board.cell_center_global(r, c) + RIGHT * board.cell_size * 0.5
                )
                show_guides = clue_count <= 5

                if not already_in_row:
                    anim = row_masks.set_bit(r, d)
                    if anim:
                        anims.append(anim)
                    if show_guides:
                        guides.append(
                            DashedLine(
                                cell_right,
                                row_masks.rows[r].bits_group.get_left(),
                                color=YELLOW,
                                stroke_width=1,
                                dash_length=0.08,
                            )
                        )

                if not already_in_col:
                    anim = col_masks.set_bit(c, d)
                    if anim:
                        anims.append(anim)
                    if show_guides:
                        guides.append(
                            DashedLine(
                                cell_right,
                                col_masks.rows[c].bits_group.get_left(),
                                color=YELLOW,
                                stroke_width=1,
                                dash_length=0.08,
                            )
                        )

                if not already_in_box:
                    anim = box_masks.set_bit(b, d)
                    if anim:
                        anims.append(anim)
                    if show_guides:
                        guides.append(
                            DashedLine(
                                cell_right,
                                box_masks.rows[b].bits_group.get_left(),
                                color=YELLOW,
                                stroke_width=1,
                                dash_length=0.08,
                            )
                        )

                if anims or guides:
                    self.play(*[Create(g, run_time=0.06) for g in guides], *anims)
                    if guides:
                        self.play(*[FadeOut(g, run_time=0.06) for g in guides])

                self.remove(hl)

        self.play(FadeOut(cursor), FadeOut(subtitle))

        # ════════════════════════════════════════════════════
        # CONFLICT MASK DEMO
        # ════════════════════════════════════════════════════

        # Precompute conflict masks
        conflict = [[0] * N for _ in range(N)]
        for r in range(N):
            for c in range(N):
                b = box_idx(r, c)
                conflict[r][c] = rows_int[r] | cols_int[c] | boxes_int[b]

        for demo_r, demo_c in [(0, 0), (0, 1), (0, 3)]:
            demo_b = box_idx(demo_r, demo_c)
            if DRY_RUN_PUZZLE[demo_r][demo_c] != 0:
                continue

            allowed = [d for d in range(1, 10) if not (conflict[demo_r][demo_c] >> (d - 1)) & 1]
            forbidden = [d for d in range(1, 10) if (conflict[demo_r][demo_c] >> (d - 1)) & 1]

            pick_label = Text(f"Pick empty cell r={demo_r}, c={demo_c}, b={demo_b}", font_size=20, color=WHITE)
            formula = Text(f"conflict[{demo_r}][{demo_c}] = rows[{demo_r}] | cols[{demo_c}] | boxes[{demo_b}]", font_size=18, color=YELLOW)
            status_row = VGroup(
                Text(f"✗ {','.join(map(str, forbidden))}", font_size=16, color=RED),
                Text(f"| ✓ {','.join(map(str, allowed))}", font_size=16, color=GREEN),
            )
            status_row.arrange(RIGHT, buff=0.15)

            info_group = VGroup(pick_label, formula, status_row)
            info_group.arrange(DOWN, buff=0.10, aligned_edge=LEFT)
            info_group.next_to(title, DOWN, buff=0.25)

            cell_hl = board.make_highlight(demo_r, demo_c, color=GREEN, opacity=0.5)
            self.add(cell_hl)

            row_hl = row_masks.rows[demo_r].copy()
            col_hl = col_masks.rows[demo_c].copy()
            box_hl = box_masks.rows[demo_b].copy()
            for hl in [row_hl, col_hl, box_hl]:
                hl.set_color(GREEN)
                hl.set_z_index(10)
                self.add(hl)

            self.play(FadeIn(info_group))
            self.wait(0.4)

            # E: animate pencil marks into the cell
            center = board.cell_center_global(demo_r, demo_c)
            cell_marks = []
            for i, d in enumerate(allowed):
                mark = Tex(str(d), font_size=10, color=YELLOW)
                mark.move_to(candidate_pos(center, board.cell_size, i))
                cell_marks.append(mark)

            self.play(*[Write(m) for m in cell_marks], run_time=0.8)
            self.wait(0.8)

            # Clean up
            self.play(
                FadeOut(info_group),
                *[FadeOut(m) for m in cell_marks],
                FadeOut(cell_hl),
                FadeOut(row_hl),
                FadeOut(col_hl),
                FadeOut(box_hl),
                run_time=0.6,
            )

        # ── Show all candidates ────────────────────────────
        subtitle2 = Text(
            "conflict[r][c] = rows[r] | cols[c] | boxes[b]", font_size=22, color=YELLOW
        ).next_to(title, DOWN, buff=0.2)
        self.play(Write(subtitle2))

        hint = Text(
            "All allowed candidates shown in yellow", font_size=16, color=GREY
        ).next_to(subtitle2, DOWN, buff=0.15)
        self.play(Write(hint))

        candidate_marks: list[VMobject] = []
        for r in range(N):
            for c in range(N):
                if DRY_RUN_PUZZLE[r][c] != 0:
                    continue
                allowed = [
                    d for d in range(1, 10) if not (conflict[r][c] >> (d - 1)) & 1
                ]
                center = board.cell_center_global(r, c)
                for i, d in enumerate(allowed):
                    mark = Tex(str(d), font_size=8, color=YELLOW, fill_opacity=0.7)
                    mark.move_to(candidate_pos(center, board.cell_size, i))
                    candidate_marks.append(mark)

        self.play(*[Write(m) for m in candidate_marks], run_time=3)
        self.wait(2)

        # ── Clean up ───────────────────────────────────────
        self.play(
            FadeOut(board),
            FadeOut(row_masks),
            FadeOut(col_masks),
            FadeOut(box_masks),
            FadeOut(subtitle2),
            FadeOut(hint),
            *[FadeOut(m) for m in candidate_marks],
            FadeOut(title),
            run_time=1.5,
        )
