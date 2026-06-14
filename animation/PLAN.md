# Animation Implementation Plan — Phases 2–6

## ✅ DONE: Data fixes

### 1. `DRY_RUN_SOLUTION` fixed
Replaced invalid solution with verified solver output:

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

### 2. `DRY_RUN_HEURISTIC_BOARD` added
Post-heuristics board (12 cells filled by Phase 2).

### 3. Graph stats corrected
- Pre-prune: **205 nodes, 1623 edges**
- Post-local-prune: **121 nodes, 580 edges** (84 removed in 3 rounds: 59→23→2)
- Post-exact-prune: ~10 supported nodes, 106 configurations

### 4. `results/graph.json` regenerated
With heuristics enabled via `--visualize` flag.


## ✅ DONE: Manim — Phase 1 (Mask Init)

**File:** `animation/scenes/phase1_mask_init.py` — COMPLETE

## ✅ DONE: Manim — Phase 5 (Local Support Pruning)

**File:** `animation/scenes/phase5_pruning.py` — COMPLETE
**Component:** `animation/components/graphviz.py` — `PermutationGraph` class

Shows full graph (205 nodes, 1623 edges), 3 pruning rounds with red pulse→fade cascade, live node/edge counters, summary card.

## ✅ DONE: HTML Visualizer — Phases 5 & 6

**File:** `tools/visualizer/graph.html` — ~2650 lines

### Phase 5: Pruning Simulator
- ▶ Run All Rounds / ⏭ Step Round / ↺ Reset
- Round timeline bars (proportional to removals, clickable)
- Support Inspector — click any node, see per-minigrid alive/dead counts
- Heatmap by Death Round toggle (red→orange→yellow)
- Focus Minigrid dropdown (isolates one mg + its neighbors)
- Cascade warnings + support-chain arrows
- Survivor mode toggle
- Dead edges hidden, dead nodes offscreen or display:none
- All timeouts guarded by `pruningRunToken`
- Reset fully clears all visual state

### Phase 6: Extraction Simulator
- Initialize Domains button — builds domain grid (3×3), candidate strip
- ⏭ Step button — state machine: MRV → candidate → assign+propagate+forced cascade
- Auto Solve — full backtracking search, 20000 step cap
- ← Undo button — restores previous state via domainHistory
- ↺ Reset button — clears all extraction state, compacts survivors, restores edges
- Domain grid renders 3×3 counts per minigrid (green ≤3, yellow 4-10, default 11+)
- Domain shrink animation (yellow flash on changed cells)
- Candidate strip shows sorted permutation IDs per selected minigrid
- Propagation wave — sequential orange pulses through changed minigrids (Step mode)
- Node death: red pulse → CSS opacity fade → display:none (300ms total)
- Causal edges only for shrinking domains (N→M, M>0), not emptied (contradiction)
- Forced singletons auto-assigned with green glow, chain reaction
- Contradiction: domain cell red shake, cause-node red highlight, auto-undo
- Search tree: unicode ├── └── per decision level, forced entries marked
- Solution board: 9×9 table with given digits white, solved blue, block borders
- SOLUTION event: 6-phase cinematic reveal (pulse→fade→hide→reveal→edges→board)

### Architecture (Phase 6)
```
solverNextEvent()     — pure state machine, no DOM
animateSolverEvent()  — DOM renderer, no state mutation
advanceSolver()       — orchestrator, calls both
```

Step and Auto share `advanceSolver()`. Events: MRV → CANDIDATE → PROPAGATE → FORCED × N → CONTRADICTION or SOLUTION.

### Node layout
- `node.originalX`, `node.originalY` — immutable, set at graph creation
- `node.x`, `node.y` — mutable, used for repacking and edge drawing
- `repackMinigrid()` and `relayoutSurvivors()` sort by `originalX` to preserve perm ordering
- Reset and Initialize compact survivors evenly (no "weird gaps" from sparse rows)


## TODO: Manim — Phases 2, 3, 4, 6

These are still stubs in `animation/scenes/phaseN_*.py`.

| Phase | What it does | Scene file | Status |
|-------|-------------|------------|--------|
| 1 | Mask initialization | `phase1_mask_init.py` | ✅ DONE |
| 2 | Deterministic deduction (naked/hidden singles) | `phase2_deduction.py` | 🟡 PLANNED (this doc) |
| 3 | Permutation generation (DFS + MRV) | `phase3_permutations.py` | 🔴 STUB |
| 4 | Graph construction (compatibility edges) | `phase4_graph.py` | 🔴 STUB |
| 5 | Local support pruning | `phase5_pruning.py` | ✅ DONE |
| 6 | Extraction (MRV + propagation + board) | `phase6_extraction.py` | 🔴 STUB |

The HTML visualizer (`tools/visualizer/graph.html`) fully covers Phases 5 and 6
interactively. The Manim scenes for Phase 2–4 and 6 are not yet implemented.


## 🟡 PLANNED: Manim — Phase 2 (Deduction) — v2

### Goal

Replace the current 12-fill illustrative scene with one that **shows the actual
Rust algorithm in action** — pencil marks, queue, pair-scan rounds, and the
constraint-propagation cascade.

### Exact fill trace (from `propagate_constraints`)

```
FILL  1: (2,5)=5   hidden single (row 2, d=5)
FILL  2: (3,7)=6   hidden single (row 3, d=6)
FILL  3: (5,6)=5   hidden single (row 5, d=5)
FILL  4: (5,1)=8   hidden single (col 1, d=8)
FILL  5: (0,2)=5   hidden single (box 0, d=5)
--- queue drains, pair scan round 1 runs, 1+ pair(s) push new singles ---
FILL  6: (6,7)=5   pair-driven single
FILL  7: (8,4)=5   pair-driven single
FILL  8: (3,5)=2   pair-driven single
FILL  9: (3,4)=9   pair-driven single
FILL 10: (4,4)=3   pair-driven single
FILL 11: (1,5)=3   pair-driven single
FILL 12: (0,6)=3   pair-driven single
--- quiescence: 12/52 cells filled, 40 empty, pairs+ singles exhausted ---
```

### Algorithm story (what the viewer sees)

1. **Initialize** — every empty cell shows pencil marks = `allowed[r][c]`
   (computed from conflict masks built in Phase 1).
2. **Queue seeds** — the Rust code seeds hidden singles from
   `row_count[r][d] == 1` etc. We visualize this as a "scan" pulse across
   rows, cols, and boxes, with yellow dots landing on the 5 hidden-single cells.
3. **Singles cascade (wave A, fills 1–5)** — for each fill, in Rust order:
   - Highlight the row/col/box where `count == 1` (the "house" enforcing the digit)
   - Erase digit `d` from pencil marks of all 20 peer cells in that house
     (this is `remove_digit` from the propagation loop)
   - Place digit `d`, remove the now-stale pencil marks from the filled cell
   - The same `remove_digit` calls may push new naked singles — show them
     appearing in the queue panel
4. **Pair scan round** — banner: "queue empty → scan pairs → enqueue new singles".
   No specific pair is shown in detail; the focus is the **transition**:
   "pairs reduced options, that surfaced new forced singles."
5. **Singles cascade (wave B, fills 6–12)** — same per-fill treatment, but
   color the digit **orange** instead of yellow to mark "pair-driven".
6. **Quiescence** — banner: "12/52 filled → 40 cells still ambiguous →
   backtracking required." Side-by-side before/after comparison.

### New / updated components

| File | Change |
|------|--------|
| `animation/puzzle.py` | Add `PRECOMPUTED_TRACE` — list of `(r, c, d, technique, house)` tuples (computed once, embedded) |
| `animation/components/board.py` | Add `set_pencil_marks(allowed)`, `erase_pencil_mark(r, c, d)`, `clear_pencil_marks(r, c)` |
| `animation/components/state_panel.py` | **NEW** — side panel showing queue contents + pair-round counter + fill count |
| `animation/scenes/phase2_deduction.py` | Full rewrite using new components + the trace |

### Build milestones (render+review after each)

1. **M1 — Pencil marks**: render empty board with allowed[r][c] pencil marks.
   Verify: marks are correct, font size is legible, no overlap with grid.
2. **M2 — Single fill animation**: pick fill 1 ((2,5)=5). Highlight row 2,
   show count==1 for digit 5, erase "5" from row 2's pencil marks, place "5".
   Verify: erase step is visible, count==1 visual is clear.
3. **M3 — Wave A loop**: render all 5 hidden singles in sequence with
   the same per-fill treatment.
4. **M4 — Queue + state panel**: add the side panel, show queue growing
   and draining as singles are filled.
5. **M5 — Pair scan banner + Wave B**: insert the banner animation,
   then render fills 6–12 in orange.
6. **M6 — Quiescence + finale**: 12/52 counter, side-by-side comparison.

### Verification per milestone

- `manim -pqh animation/scenes/phase2_deduction.py Phase2Deduction`
  (480p15 preview) after each milestone
- `cargo test` (must stay green; no Rust changes)
- Visual checks: pencil marks readable, highlights clearly attributed
  to a row/col/box, queue panel updates match the fill order

### What is NOT in scope

- Showing naked-pair / hidden-pair / pointing-pair / claiming-pair internals
  in detail. The scene treats "pair round" as a single banner moment.
  Detailed pair visualization is a future enhancement.
- No Rust changes. All algorithm fidelity comes from the precomputed trace.


## Deliverables summary

| Deliverable | Status | Lines |
|-------------|--------|-------|
| `animation/components/board.py` | ✅ DONE | Phase 1 |
| `animation/components/bitmask.py` | ✅ DONE | Phase 1 |
| `animation/components/graphviz.py` | ✅ DONE | ~270 |
| `animation/puzzle.py` | ✅ DONE (data fixed) | ~60 |
| `animation/scenes/phase1_mask_init.py` | ✅ DONE | Phase 1 |
| `animation/scenes/phase5_pruning.py` | ✅ DONE | ~150 |
| `tools/visualizer/graph.html` | ✅ DONE (Phase 5 + 6) | ~2650 |
| `animation/PLAN.md` | ✅ DONE (this file) | — |
