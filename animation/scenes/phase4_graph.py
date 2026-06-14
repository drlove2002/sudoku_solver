"""
Phase 4: Compatibility Graph Construction

Shows the graph being built: permutation nodes appear, then compatibility
edges are drawn (green) or rejected (red flash) based on signature comparison.

Visual flow:
1. Show graph layout — nodes grouped by minigrid (like Figure 4.14 in report)
2. Animate edge creation: for each related minigrid pair (i,j):
   a. Compare each permutation pair
   b. Compatible → green edge
   c. Incompatible → brief red flash, no edge
3. Final graph statistics: 42 nodes, 152 edges (for the dry run with 5 from minigrid 9)
"""

import sys, os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from manim import *
from animation.puzzle import N, K


class Phase4Graph(Scene):
    def construct(self):
        title = Text("Phase 4: Compatibility Graph Construction", font_size=36, color=WHITE)
        title.to_edge(UP)
        self.play(Write(title))
        self.wait(0.5)

        # Placeholder
        placeholder = Text(
            "Nodes = permutations grouped by minigrid\n"
            "Edges = compatible pairs (green) / incompatible (red flash)\n"
            "42 nodes, 152 edges for the dry-run puzzle",
            font_size=20, color=GREY
        ).move_to(ORIGIN)
        self.play(Write(placeholder))
        self.wait(2)

        self.play(FadeOut(placeholder), FadeOut(title))
