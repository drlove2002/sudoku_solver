# Animation Implementation Plan — Phases 2–6

## Data Issues to Fix First

### 1. `DRY_RUN_SOLUTION` is invalid
Duplicate digits in 5 rows and 10 column positions. Replace in `animation/puzzle.py`:

```python
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
```

### 2. Add `DRY_RUN_HEURISTIC_BOARD`
Post-heuristics board (12 cells filled by Phase 2):

```python
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
```

### 3. Graph stats in docstrings are wrong
Actual numbers from solver (with heuristics):
- Pre-prune: **205 nodes, 1623 edges**
- Post-local-prune: **121 nodes, 580 edges**
- Post-exact-prune: ~10 supported nodes, 106 configurations
- Animation uses post-local-prune graph (tractable to visualize, clear cascade)

### 4. Regenerate `results/graph.json`
Already done — `cargo run -- release --bin sudoku_solver -- /tmp/dry_run_ws.txt --visualize`

---

## Phase 2: Deterministic Deduction

**Complexity:** Moderate  
**File:** `animation/scenes/phase2_deduction.py`

### Visual Flow
1. Board appears with 29 given digits (reuse `SudokuBoard`)
2. Text label: `Naked Singles` — iterate empty cells, show `allowed[r][c]` as bitmask, highlight cells with popcount == 1, fill them
3. Text label: `Hidden Singles` — for each row/col/box, highlight digit in exactly one empty cell
4. Loop: Naked → Hidden → Naked → ... until quiescence
5. Counter tracking cells filled (reaches 12)
6. Final board with 41 cells

### Implementation
Pre-compute exact fill order by porting `propagate_constraints` logic to Python. Produces sequence of `(r, c, digit, reason)` tuples to animate.

**Key challenges:** Need minimal Python port of heuristics counter logic (~150 lines). Deterministic and verifiable against Rust output.

---

## Phase 3: Permutation Generation

**Complexity:** High  
**File:** `animation/scenes/phase3_permutations.py`

### Visual Flow
1. Board zooms out, camera focuses on minigrid 1 (top-center) — only 4 permutations
2. 3×3 minigrid blown up — shows existing digits (9,4,3) and 6 empty cells
3. DFS tree on right side: nodes = partial assignments, edges = digit choices
4. MRV highlight: "cell (r,c) has only 2 candidates → try 6"
5. Dead end shown as red X on tree node, backtrack arrow curves back
6. Success: 4 complete permutations appear as small 3×3 grids
7. Fast-forward: minigrids 0-8 flash with their permutation counts (24,4,6,18,3,4,41,8,13)

### Implementation
- DFS tree layout — manual node/edge positioning in `Tree` or raw `Dot`/`Line`
- Pre-compute or construct illustrative search path for minigrid 1
- Use MRV heuristic to pick cells (same as Rust `find_best_cell`)
- Tree can be simplified — show key decision points, not every branch

**Key challenges:** Tree layout for 6-empty-cell minigrid could be deep. Simplify to illustrative path showing concepts (branching, backtracking, complete perms).

---

## Phase 4: Graph Construction

**Complexity:** High  
**File:** `animation/scenes/phase4_graph.py`  
**New component:** `animation/components/graphviz.py`

### Visual Flow
1. Render 121 nodes as colored dots in 3×3 cluster layout (one group per minigrid)
2. Title: `Compatibility Graph: 121 nodes, 580 edges`
3. Edge creation sweep through minigrid pairs:
   - Row-related (e.g., MG0-MG1): horizontal edges
   - Column-related (e.g., MG0-MG3): vertical edges
4. Build edges gradually, coloring by relation type
5. Final graph: all 580 edges visible, stats displayed

### New Component: `PermutationGraph`
- Nodes: colored dots grouped by minigrid in 3×3 spatial grid
- Edges: thin semitransparent lines between compatible nodes
- Methods: `add_edge()`, `remove_node()`, `pulse_node()`, `fade_edge()`
- Reads from `results/graph.json`

**Key challenges:** 580 edges is dense — use 10-20% opacity to avoid visual noise. Nodes within each cluster spread evenly.

---

## Phase 5: Local Support Pruning

**Complexity:** Medium  
**File:** `animation/scenes/phase5_pruning.py`

### Visual Flow
1. Graph from Phase 4 (121 nodes, 580 edges)
2. Iterative pruning: for each node, check compatible neighbor in every related minigrid
3. Unsupported nodes pulse red, then fade — connected edges also fade
4. Cascade: node A drops → node B loses last neighbor → node B drops
5. Counter: "84 removed"
6. Final graph: 37 nodes after local pruning
7. Brief skip to exact pruning: "Exact global support → 27 more removed, 10 supported, 106 configurations"

### Implementation
Pre-compute what gets removed in each iteration using graph.json + local pruning algorithm. Animate in rounds.

---

## Phase 6: Extraction

**Complexity:** Medium  
**File:** `animation/scenes/phase6_extraction.py`

### Visual Flow
1. Pruned graph on one side, board on the other
2. MRV selection: highlight minigrid with fewest remaining permutations
3. Assign one permutation → propagate: other minigrid domains shrink
4. Domain size counters update in real-time
5. After all 9 assigned: board reconstruction animation (cells fill in)
6. Show final solution board
7. Label: `Puzzle: 106 Solutions (Ambiguous)`
8. Fast-forward: other solutions flash briefly

### Implementation
Trace one specific configuration assignment from solver. Animate each step: pick minigrid → assign perm → propagate → update domains.

---

## New Components

| Component | File | Purpose |
|-----------|------|---------|
| `PermutationGraph` | `animation/components/graphviz.py` | Nodes as dots, edges as lines, grouped by minigrid, add/remove/pulse animations |
| `DFSTree` | `animation/scenes/phase3_permutations.py` (inline) | DFS tree for Phase 3 — too custom to extract as component |

---

## Risks / Open Questions

1. **Heuristics fill order**: Pre-compute via Python port of `propagate_constraints`. Deterministic, verifiable against Rust output (12 cells, same board).

2. **DFS tree for minigrid 1**: Actual path depends on MRV choices. Use illustrative path showing key concepts (branching, backtracking, complete perms) rather than exact internal state.

3. **580 edges on screen**: Partial opacity (10-20%) or fade edges not currently in focus.

4. **`results/graph.json`**: Currently 1623 edges (pre-local-prune). Need Python analysis script to pre-compute local pruning iteration state for Phase 5.

---

## Verification

- `manim -ql` (low quality) for each scene to verify layout and timing
- `cargo test` to confirm solver still works
- Visual inspection of rendered MP4s

---

## Execution Order

1. Fix `puzzle.py` data (solution + heuristic board)
2. Write Python heuristics script for Phase 2 fill order
3. Phase 2 implementation
4. Phase 3 implementation (with inline DFS tree)
5. `graphviz.py` component
6. Phase 4 implementation
7. Python analysis script for pruning iterations
8. Phase 5 implementation
9. Phase 6 implementation
10. Render all, verify
