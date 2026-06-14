"""
Phase 5: Local Support Pruning

Cascade effect: unsupported nodes fade away one by one, and their removal
cascades to remove nodes that relied on them for support.

Visual flow:
1. Show full graph (205 nodes, 1623 edges)
2. Iterative sweep in batches:
   a. Red pulse on all unsupported nodes in the current round
   b. Fade them out with their edges
   c. Live counters update (nodes remaining, edges active)
   d. Brief label: "Round N — cascade" if cascade detected
3. Final pruned graph: 121 nodes
4. Summary card with before/after stats
"""

import sys, os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from manim import *
from animation.puzzle import N, K
from animation.components.graphviz import PermutationGraph


class Phase5Pruning(Scene):
    def construct(self):
        # ---- Top bar: title + counters ----
        title = Text("Phase 5: Local Support Pruning", font_size=28, color=WHITE)
        title.to_edge(UP)
        self.play(Write(title))

        # ---- Load graph data + pre-compute rounds ----
        graph = PermutationGraph().scale(0.85).move_to(DOWN * 0.3)
        rounds = graph.compute_pruning_rounds()
        initial_nodes = len(graph.dots)
        initial_edges = len(graph.edge_group)

        # ---- Bottom counter bar ----
        node_label = Text(
            f"Nodes: {initial_nodes} / {initial_nodes}", font_size=22, color=WHITE
        )
        edge_label = Text(
            f"Edges: {initial_edges} / {initial_edges}", font_size=22, color=WHITE
        )
        counters = VGroup(node_label, edge_label).arrange(RIGHT, buff=1.0)
        counters.to_edge(DOWN, buff=0.5)

        # ---- Subtitle: algorithm explanation ----
        subtitle = Text(
            "Each node must have support in EVERY related minigrid",
            font_size=18, color=GREY,
        ).next_to(title, DOWN, buff=0.15)

        self.play(Write(subtitle))
        self.play(Write(counters))
        self.wait(0.3)

        # ---- Show the graph ----
        self.play(FadeIn(graph), run_time=1.2)
        self.wait(1.0)

        # ---- Live counters tracker ----
        remaining_nodes = initial_nodes
        remaining_edges = initial_edges

        def update_counters(removed_this_round: int):
            nonlocal remaining_nodes, remaining_edges, node_label, edge_label
            remaining_nodes -= removed_this_round
            # Count active edges: edges where both endpoints are alive
            active_edges = 0
            for line in graph.edge_group:
                start = line.get_start()
                end = line.get_end()
                # Find which nodes these endpoints belong to by position match
                e_start = None
                e_end = None
                for key, dot in graph.dots.items():
                    if np.allclose(dot.get_center(), start, atol=0.01):
                        e_start = key
                    if np.allclose(dot.get_center(), end, atol=0.01):
                        e_end = key
                if e_start and e_end:
                    if graph.alive.get(e_start, False) and graph.alive.get(e_end, False):
                        active_edges += 1
            remaining_edges = active_edges

            new_node = Text(
                f"Nodes: {remaining_nodes} / {initial_nodes}",
                font_size=22, color=WHITE,
            )
            new_edge = Text(
                f"Edges: {remaining_edges} / {initial_edges}",
                font_size=22, color=WHITE,
            )
            new_counters = VGroup(new_node, new_edge).arrange(RIGHT, buff=1.0)
            new_counters.to_edge(DOWN, buff=0.5)
            self.play(
                FadeOut(node_label, run_time=0.15),
                FadeIn(new_node, run_time=0.15),
                FadeOut(edge_label, run_time=0.15),
                FadeIn(new_edge, run_time=0.15),
            )
            node_label, edge_label = new_node, new_edge

        # ---- Round labels ----
        round_label = Text("", font_size=20, color=YELLOW)
        round_label.to_edge(DOWN, buff=1.0)

        # ---- Animate pruning rounds ----
        for round_idx, removals in enumerate(rounds):
            # Before round: mark unsupported nodes with red pulse
            round_text = f"Round {round_idx + 1}: {len(removals)} unsupported"
            if round_idx > 0:
                round_text += " (cascade)"
            new_round_label = Text(round_text, font_size=20, color=YELLOW)
            new_round_label.to_edge(DOWN, buff=1.0)
            if round_idx == 0:
                self.play(Write(new_round_label))
            else:
                self.play(FadeOut(round_label, run_time=0.15), FadeIn(new_round_label, run_time=0.15))
            round_label = new_round_label

            self.wait(0.2)

            # Pulse all unsupported nodes in this round (red glow)
            graph.pulse_nodes_batch(removals, self)
            self.wait(0.1)

            # Fade them all out
            graph.fade_nodes_batch(removals, self)
            self.wait(0.1)

            # Update counters
            update_counters(len(removals))

        # Clean up round label
        self.play(FadeOut(round_label))

        # ---- Final summary card ----
        final_nodes = graph.alive_count()
        removed = initial_nodes - final_nodes

        self.wait(1.0)

        # Dim the remaining graph
        self.play(graph.animate.set_opacity(0.35), run_time=0.6)

        summary = VGroup(
            Text(f"Local Support Pruning — Complete", font_size=26, color=WHITE),
            Text(
                f"{removed} nodes removed in {len(rounds)} rounds\n"
                f"{initial_nodes} → {final_nodes} nodes\n"
                f"{initial_edges} → {remaining_edges} edges",
                font_size=20, color=GREY, line_spacing=0.6,
            ),
        ).arrange(DOWN, buff=0.3)
        summary.move_to(ORIGIN)

        self.play(Write(summary), run_time=1.0)
        self.wait(3.0)

        # Fade out
        self.play(FadeOut(summary), FadeOut(graph), FadeOut(counters), FadeOut(subtitle), FadeOut(title))
