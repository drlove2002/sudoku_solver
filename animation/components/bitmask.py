"""
Bitmask visualization for Phase 1: Mask Initialization.

- BitRow: one row of 9 zero/one characters representing a u32 bitmask
- MaskPanel: column of 9 BitRows with a digit header and panel title

Layout:
        1 2 3 4 5 6 7 8 9
rows[r]
  1     0 0 0 0 0 1 1 0 1
  2     0 0 0 1 1 0 0 0 1
  ...

Uses Text (not Tex) for better rendering speed.
"""

from manim import *


class BitRow(VGroup):
    """One row of 9 characters (0/1) representing a u32 bitmask.

    Example when bits = 0x068 (digits {3,5,6} set):
        1  0 0 1 0 1 1 0 0 0
    """

    def __init__(self, label: str = "", **kwargs):
        super().__init__(**kwargs)
        self.bits: int = 0
        self.bit_texts: list[Text] = []

        bits_group = VGroup()
        for _ in range(9):
            txt = Text("0", font="monospace", font_size=18, color=DARK_GREY)
            self.bit_texts.append(txt)
            bits_group.add(txt)

        bits_group.arrange(RIGHT, buff=0.10)

        self.row_label = Text(label, font="monospace", font_size=18, color=GREY)
        self.row_label.next_to(bits_group, LEFT, buff=0.25, aligned_edge=DOWN)

        self.add(self.row_label, bits_group)
        self.bits_group = bits_group

    def set_bit(self, digit: int):
        """Flip 0→1 at position (digit-1). Returns Succession animation."""
        pos = digit - 1
        if (self.bits >> pos) & 1:
            return None

        self.bits |= 1 << pos
        old = self.bit_texts[pos]
        new = Text("1", font="monospace", font_size=18, color=YELLOW)
        new.move_to(old.get_center())

        return Succession(
            Transform(old, new, run_time=0.08),
            Flash(old.get_center(), color=YELLOW, flash_radius=0.18, run_time=0.12),
        )


class MaskPanel(VGroup):
    """Column of 9 BitRows with a header and title.

    Args:
        title: panel heading (e.g. "rows[r]")
    """

    def __init__(self, title: str, **kwargs):
        super().__init__(**kwargs)

        self.title_tex = Text(title, font="monospace", font_size=22, color=WHITE)
        self.add(self.title_tex)

        self.header = VGroup(*[
            Text(str(i), font="monospace", font_size=15, color=GREY)
            for i in range(1, 10)
        ])
        # Don't arrange — we pin each digit above its column in _layout
        self.add(self.header)

        self.rows: list[BitRow] = []
        for i in range(9):
            row = BitRow(label=f"{i}")
            self.rows.append(row)
            self.add(row)

        self._layout()

    def _layout(self):
        """Position rows, header, and title. Header/title anchored to row group top."""
        rows_vg = VGroup(*self.rows)
        rows_vg.arrange(DOWN, buff=0.04, aligned_edge=LEFT)

        # Pin each header digit directly above its column
        for i in range(9):
            self.header[i].move_to(self.rows[0].bit_texts[i].get_center())
            self.header[i].shift(UP * 0.40)

        # Align all row labels to the right edge of row 0's label
        anchor_right = self.rows[0].row_label.get_right()
        for row in self.rows[1:]:
            row.row_label.align_to(anchor_right, RIGHT)

        # Title above header
        self.title_tex.next_to(self.header, UP, buff=0.12)

    def relayout(self):
        """Re-anchor header and title after rows have been shifted."""
        for i in range(9):
            self.header[i].move_to(self.rows[0].bit_texts[i].get_center())
            self.header[i].shift(UP * 0.40)
        self.title_tex.next_to(self.header, UP, buff=0.12)

    def set_bit(self, row_idx: int, digit: int):
        return self.rows[row_idx].set_bit(digit)
