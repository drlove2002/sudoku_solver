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
    [8, 1, 5, 6, 3, 7, 2, 9, 4],
    [6, 7, 2, 9, 4, 8, 1, 3, 5],
    [9, 4, 3, 1, 8, 2, 5, 7, 6],
    [3, 2, 9, 5, 1, 4, 4, 6, 8],
    [5, 6, 4, 8, 7, 3, 9, 1, 2],
    [8, 9, 7, 2, 6, 1, 3, 3, 9],
    [7, 3, 8, 4, 7, 5, 6, 2, 1],
    [2, 5, 6, 3, 9, 8, 9, 4, 7],
    [1, 2, 9, 7, 5, 4, 8, 6, 3],
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
