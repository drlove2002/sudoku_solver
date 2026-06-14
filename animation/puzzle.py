"""The 29-clue dry-run puzzle from Section 4.7 of the report (Figure 4.3).

Board layout (row-major, 0 = empty):
    8 0 0 | 0 0 7 | 0 9 0
    0 0 0 | 9 4 0 | 0 0 5
    0 0 3 | 0 8 0 | 0 7 0
    ------+-------+------
    0 0 0 | 5 0 0 | 4 0 8
    5 6 0 | 0 0 0 | 0 1 0
    0 0 0 | 0 6 1 | 0 3 0
    ------+-------+------
    0 0 8 | 0 7 0 | 6 0 0
    0 5 0 | 3 0 0 | 9 0 0
    1 0 0 | 0 0 0 | 0 0 0

The report traces Phase 1 → Phase 2 (naked/hidden singles) → Phase 3 (sub-grid 1
DFS tree) → Phase 4 (42-node/152-edge graph) → Phase 5 (pruned to 9 nodes/18
edges) → Phase 6 (unique solution).

Use this module as the single source of truth for all animation scenes.
"""

from typing import List

# 9x9 board: rows indexed 0..8, columns 0..8
# Empty cells are 0
DRY_RUN_PUZZLE: List[List[int]] = [
    [8, 0, 0, 0, 0, 7, 0, 9, 0],
    [0, 0, 0, 9, 4, 0, 0, 0, 5],
    [0, 0, 3, 0, 8, 0, 0, 7, 0],
    [0, 0, 0, 5, 0, 0, 4, 0, 8],
    [5, 6, 0, 0, 0, 0, 0, 1, 0],
    [0, 0, 0, 0, 6, 1, 0, 3, 0],
    [0, 0, 8, 0, 7, 0, 6, 0, 0],
    [0, 5, 0, 3, 0, 0, 9, 0, 0],
    [1, 0, 0, 0, 0, 0, 0, 0, 0],
]

# The unique solution (from the report dry run):
# <1a, 2b, 3a, 4a, 5b, 6b, 7f, 8c, 9d>
DRY_RUN_SOLUTION: List[List[int]] = [
    [8, 2, 5, 6, 1, 7, 3, 9, 4],
    [7, 1, 6, 9, 4, 3, 8, 2, 5],
    [9, 4, 3, 2, 8, 5, 1, 7, 6],
    [3, 7, 1, 5, 9, 2, 4, 6, 8],
    [5, 6, 9, 8, 3, 4, 2, 1, 7],
    [2, 8, 4, 7, 6, 1, 5, 3, 9],
    [4, 3, 8, 1, 7, 9, 6, 5, 2],
    [6, 5, 7, 3, 2, 8, 9, 4, 1],
    [1, 9, 2, 4, 5, 6, 7, 8, 3],
]

# Post-heuristics board — 12 cells filled by Phase 2 naked/hidden singles
DRY_RUN_HEURISTIC_BOARD: List[List[int]] = [
    [8, 0, 5, 0, 0, 7, 3, 9, 0],
    [0, 0, 0, 9, 4, 3, 0, 0, 5],
    [0, 0, 3, 0, 8, 5, 0, 7, 0],
    [0, 0, 0, 5, 9, 2, 4, 6, 8],
    [5, 6, 0, 0, 3, 0, 0, 1, 0],
    [0, 8, 0, 0, 6, 1, 5, 3, 0],
    [0, 0, 8, 0, 7, 0, 6, 5, 0],
    [0, 5, 0, 3, 0, 0, 9, 0, 0],
    [1, 0, 0, 0, 5, 0, 0, 0, 0],
]

# Cell coordinates for each minigrid (0-indexed)
# Minigrid ID = (row // 3) * 3 + (col // 3)
K = 3
N = 9


def box_idx(r: int, c: int) -> int:
    """Return minigrid id (0..8) for global cell (r, c)."""
    return (r // K) * K + (c // K)


def minigrid_cells(mg_id: int):
    """Yield (r, c) pairs for every cell in minigrid <mg_id>."""
    base_r = (mg_id // K) * K
    base_c = (mg_id % K) * K
    for dr in range(K):
        for dc in range(K):
            yield (base_r + dr, base_c + dc)


def minigrid_values(board: List[List[int]], mg_id: int) -> List[int]:
    """Return flattened 9-element list of values in minigrid <mg_id>."""
    return [board[r][c] for r, c in minigrid_cells(mg_id)]


def is_given(board: List[List[int]], r: int, c: int) -> bool:
    """Check if cell (r, c) was a given clue in the original puzzle."""
    return DRY_RUN_PUZZLE[r][c] != 0


def compute_allowed(puzzle: List[List[int]] | None = None) -> List[List[int]]:
    """Compute the initial allowed[r][c] bitmask grid for a puzzle.

    Mirrors the Rust `Masks::generate` + `allowed = all_mask ^ conflict`:
      conflict[r][c] = rows[r] | cols[c] | boxes[b]
      allowed[r][c]  = (~conflict[r][c]) & all_mask, only for empty cells

    Each cell's bit (d-1) set means digit d is still a candidate.
    """
    src = puzzle if puzzle is not None else DRY_RUN_PUZZLE
    all_mask: int = (1 << N) - 1
    rows_mask = [0] * N
    cols_mask = [0] * N
    boxes_mask = [0] * N
    for r in range(N):
        for c in range(N):
            d = src[r][c]
            if d != 0:
                bit = 1 << (d - 1)
                rows_mask[r] |= bit
                cols_mask[c] |= bit
                boxes_mask[box_idx(r, c)] |= bit
    allowed = [[0] * N for _ in range(N)]
    for r in range(N):
        for c in range(N):
            if src[r][c] != 0:
                continue
            b = box_idx(r, c)
            conflict = rows_mask[r] | cols_mask[c] | boxes_mask[b]
            allowed[r][c] = all_mask & ~conflict
    return allowed
