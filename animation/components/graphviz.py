"""
Permutation graph visualization for the solver's minigrid compatibility graph.

9 minigrid clusters in a 3x3 Sudoku-board layout with permutation nodes and
compatibility edges. Used in Phase 4 (graph construction) and Phase 5
(local support pruning) scenes.
"""

from manim import *
import json
import math
import pathlib
import numpy as np

MINIGRID_COLORS = [
    "#FF6B6B",  # MG0
    "#4ECDC4",  # MG1
    "#45B7D1",  # MG2
    "#96CEB4",  # MG3
    "#FFEAA7",  # MG4
    "#DDA0DD",  # MG5
    "#98D8C8",  # MG6
    "#F7DC6F",  # MG7
    "#BB8FCE",  # MG8
]


class PermutationGraph(VGroup):
    """9-cluster minigrid compatibility graph with nodes, edges, and labels.

    Arranges 9 minigrid clusters in a 3x3 grid matching the Sudoku board
    layout. Each cluster contains permutation nodes (Dot) with compatibility
    edges (Line) between related minigrids.

    Supports pruning animations: pulse_node, fade_node, and iterative
    support checking via compute_pruning_rounds.
    """

    def __init__(self, graph_path: str | None = None, **kwargs):
        super().__init__(**kwargs)

        if graph_path is None:
            graph_path = str(
                pathlib.Path(__file__).parent.parent.parent
                / "results" / "graph.json"
            )

        with open(graph_path) as f:
            data = json.load(f)

        # ---- build node & compatibility maps ----
        self.nodes_data: dict[tuple[int, int], dict] = {}
        node_id_map: dict[str, tuple[int, int]] = {}
        for n in data["nodes"]:
            key = (n["minigrid"], n["perm_id"])
            self.nodes_data[key] = n
            node_id_map[n["id"]] = key

        self.compatible: dict[tuple[int, int], dict[int, set[int]]] = {
            key: {} for key in self.nodes_data
        }
        for e in data["edges"]:
            src = node_id_map[e["source"]]
            dst = node_id_map[e["target"]]
            mg_s, p_s = src
            mg_d, p_d = dst
            self.compatible[src].setdefault(mg_d, set()).add(p_d)
            self.compatible[dst].setdefault(mg_s, set()).add(p_s)

        # ---- pre-compute minigrid relations ----
        self.relation: dict[tuple[int, int], str] = {}
        for a in range(9):
            for b in range(9):
                if a == b:
                    self.relation[(a, b)] = "Not"
                elif a // 3 == b // 3:
                    self.relation[(a, b)] = "Row"
                elif a % 3 == b % 3:
                    self.relation[(a, b)] = "Col"
                else:
                    self.relation[(a, b)] = "Not"

        # ---- group nodes by minigrid ----
        mg_nodes: dict[int, list[int]] = {}
        for mg, perm in self.nodes_data:
            mg_nodes.setdefault(mg, []).append(perm)

        # ---- layout constants ----
        cw, ch = 2.6, 2.0
        gap = 0.7
        tw = 3 * cw + 2 * gap
        th = 3 * ch + 2 * gap

        self.dots: dict[tuple[int, int], Dot] = {}
        self.alive: dict[tuple[int, int], bool] = {
            key: True for key in self.nodes_data
        }
        self.edge_group = VGroup()
        self.node_edges: dict[tuple[int, int], list[Line]] = {
            key: [] for key in self.nodes_data
        }

        labels = []

        for mg_id in range(9):
            perms = sorted(mg_nodes[mg_id])
            n = len(perms)
            gr, gc = divmod(mg_id, 3)
            cc = np.array([
                -tw / 2 + cw / 2 + gc * (cw + gap),
                th / 2 - ch / 2 - gr * (ch + gap), 0
            ])

            cols = max(1, math.ceil(math.sqrt(n * cw / ch)))
            rows = max(1, math.ceil(n / cols))
            sx, sy = cw / cols, ch / rows
            radius = min(sx, sy) * 0.22
            color = MINIGRID_COLORS[mg_id]

            for i, pid in enumerate(perms):
                col, row = i % cols, i // cols
                pos = cc + np.array([
                    -cw / 2 + sx / 2 + col * sx,
                    ch / 2 - sy / 2 - row * sy, 0
                ])
                self.dots[(mg_id, pid)] = Dot(
                    point=pos, radius=radius,
                    fill_color=color, fill_opacity=0.85, stroke_width=0,
                )

            label = Text(f"MG{mg_id}", font="monospace", font_size=18, color=color)
            label.next_to(cc, UP, buff=0.15 + ch / 2)
            labels.append(label)

        # ---- build edges ----
        for e in data["edges"]:
            src = node_id_map[e["source"]]
            dst = node_id_map[e["target"]]
            line = Line(
                start=self.dots[src].get_center(),
                end=self.dots[dst].get_center(),
                stroke_width=0.6, stroke_opacity=0.06, color=GREY,
            )
            self.edge_group.add(line)
            self.node_edges[src].append(line)
            self.node_edges[dst].append(line)

        # ---- add to self (layering: edges → dots → labels) ----
        self.add(self.edge_group)
        for key in self.nodes_data:
            self.add(self.dots[key])
        for label in labels:
            self.add(label)

    # ---- query methods ----

    def compatible_set(self, mg_id: int, perm_id: int, other_mg: int) -> frozenset:
        """Return frozenset of compatible perm_ids in other_mg."""
        key = (mg_id, perm_id)
        if key not in self.compatible:
            return frozenset()
        return frozenset(self.compatible[key].get(other_mg, set()))

    def node_count(self, mg_id: int) -> int:
        """Return current number of active (non-faded) nodes in minigrid."""
        return sum(
            1 for (mg, _), alive in self.alive.items()
            if mg == mg_id and alive
        )

    def alive_count(self) -> int:
        """Return total number of active (non-faded) nodes."""
        return sum(1 for alive in self.alive.values() if alive)

    def all_nodes(self) -> list[tuple[int, int]]:
        """Return list of all (mg_id, perm_id) currently tracked."""
        return list(self.nodes_data.keys())

    # ---- animation methods ----

    def pulse_node(self, mg_id: int, perm_id: int, scene) -> None:
        """Animate a red glow pulse on the given node."""
        dot = self.dots.get((mg_id, perm_id))
        if dot is None:
            return
        circle = Circle(
            radius=dot.radius * 3, color=RED,
            fill_opacity=0.4, stroke_width=0,
        ).move_to(dot)
        scene.add(circle)
        scene.play(GrowFromCenter(circle), rate_func=there_and_back, run_time=0.5)
        scene.remove(circle)

    def fade_node(self, mg_id: int, perm_id: int, scene) -> None:
        """Fade out the dot and all its connected edge lines."""
        dot = self.dots.get((mg_id, perm_id))
        if dot is None:
            return
        edges = self.node_edges.get((mg_id, perm_id), [])
        scene.play(FadeOut(VGroup(dot, *edges)), run_time=0.3)
        self.alive[(mg_id, perm_id)] = False

    def pulse_and_fade(self, mg_id: int, perm_id: int, scene) -> None:
        """Convenience: pulse_node then fade_node."""
        self.pulse_node(mg_id, perm_id, scene)
        self.fade_node(mg_id, perm_id, scene)

    def pulse_nodes_batch(self, removals: list[tuple[int, int]], scene) -> None:
        """Pulse multiple nodes with red glow in one animation."""
        circles = VGroup()
        for mg_id, perm_id in removals:
            dot = self.dots.get((mg_id, perm_id))
            if dot is None:
                continue
            circle = Circle(
                radius=dot.radius * 3, color=RED,
                fill_opacity=0.4, stroke_width=0,
            ).move_to(dot)
            circles.add(circle)
        if len(circles) == 0:
            return
        scene.add(circles)
        scene.play(GrowFromCenter(circles), rate_func=there_and_back, run_time=0.5)
        scene.remove(circles)

    def fade_nodes_batch(self, removals: list[tuple[int, int]], scene) -> None:
        """Fade out multiple nodes and their edges in one animation."""
        group = VGroup()
        for mg_id, perm_id in removals:
            dot = self.dots.get((mg_id, perm_id))
            if dot is not None:
                group.add(dot)
            for line in self.node_edges.get((mg_id, perm_id), []):
                group.add(line)
        if len(group) == 0:
            return
        scene.play(FadeOut(group), run_time=0.5)
        for mg_id, perm_id in removals:
            self.alive[(mg_id, perm_id)] = False

    def pulse_and_fade_batch(self, removals: list[tuple[int, int]], scene) -> None:
        """Convenience: pulse then fade a batch of nodes."""
        self.pulse_nodes_batch(removals, scene)
        self.fade_nodes_batch(removals, scene)

    # ---- pruning ----

    def compute_pruning_rounds(self) -> list[list[tuple[int, int]]]:
        """Run local support pruning, returning rounds of removals.

        Each round: scan alive nodes, removing those without at least one
        alive compatible partner in every related (Row/Col) minigrid.
        Cascades until fixpoint. Returns list of removal rounds.
        """
        alive: dict[tuple[int, int], bool] = {
            key: True for key in self.nodes_data
        }
        rounds: list[list[tuple[int, int]]] = []

        while True:
            removals: list[tuple[int, int]] = []
            for key, is_alive in alive.items():
                if not is_alive:
                    continue
                mg, perm = key
                for other_mg in range(9):
                    rel = self.relation[(mg, other_mg)]
                    if rel == "Not":
                        continue
                    compat = self.compatible[key].get(other_mg, set())
                    has = any(alive.get((other_mg, cp), False) for cp in compat)
                    if not has:
                        removals.append(key)
                        break
            if not removals:
                break
            for key in removals:
                alive[key] = False
            rounds.append(removals)

        return rounds

    # ---- utility ----

    def relationship(self, a: int, b: int) -> str:
        return self.relation[(a, b)]
