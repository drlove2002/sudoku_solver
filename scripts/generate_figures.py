#!/usr/bin/env python3
"""
Generate publication-quality figures for the full sudoku_solver paper.
"""

from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


RESULTS_DIR = Path("results")
FIGURES_DIR = Path("docs/figures")


plt.style.use("seaborn-v0_8-paper")
plt.rcParams["font.size"] = 10
plt.rcParams["axes.labelsize"] = 11
plt.rcParams["axes.titlesize"] = 12
plt.rcParams["xtick.labelsize"] = 9
plt.rcParams["ytick.labelsize"] = 9
plt.rcParams["legend.fontsize"] = 9
plt.rcParams["figure.titlesize"] = 13


def load_csv(name: str) -> pd.DataFrame | None:
    path = RESULTS_DIR / name
    if not path.exists():
        print(f"Warning: {path} not found")
        return None
    return pd.read_csv(path)


def load_analysis_data() -> dict[str, pd.DataFrame]:
    results: dict[str, pd.DataFrame] = {}
    for difficulty in ["easy", "medium", "hard"]:
        df = load_csv(f"{difficulty}_analysis.csv")
        if df is not None:
            results[difficulty] = df
    return results


def save(fig: plt.Figure, name: str) -> None:
    FIGURES_DIR.mkdir(parents=True, exist_ok=True)
    path = FIGURES_DIR / name
    fig.tight_layout()
    fig.savefig(path, dpi=300)
    plt.close(fig)
    print(f"✓ Generated: {path}")


def plot_permutation_distribution(results: dict[str, pd.DataFrame]) -> None:
    fig, axes = plt.subplots(1, 3, figsize=(10.5, 3.2), sharey=True)
    colors = ["#5b8c5a", "#c98a2e", "#b24a3a"]

    for idx, difficulty in enumerate(["easy", "medium", "hard"]):
        if difficulty not in results:
            continue

        df = results[difficulty]
        perm_cols = [f"P_{i}" for i in range(9)]
        perm_data = df[perm_cols].values.flatten()
        ax = axes[idx]
        ax.hist(perm_data, bins=25, color=colors[idx], alpha=0.8, edgecolor="black")
        ax.set_title(difficulty.capitalize())
        ax.set_xlabel("Permutation Count")
        if idx == 0:
            ax.set_ylabel("Frequency")
        ax.set_yscale("log")
        ax.grid(True, axis="y", alpha=0.25)

    fig.suptitle("Phase 2: Minigrid Permutation Distribution")
    save(fig, "permutation_distribution.pdf")


def plot_graph_sizes(results: dict[str, pd.DataFrame]) -> None:
    difficulties = []
    initial_vertices = []
    pruned_vertices = []
    initial_edges = []
    pruned_edges = []

    for difficulty in ["easy", "medium", "hard"]:
        if difficulty not in results:
            continue
        df = results[difficulty]
        difficulties.append(difficulty.capitalize())
        initial_vertices.append(df["initial_vertex_count"].mean())
        pruned_vertices.append(df["pruned_vertex_count"].mean())
        initial_edges.append(df["initial_edge_count"].mean())
        pruned_edges.append(df["pruned_edge_count"].mean())

    x = np.arange(len(difficulties))
    width = 0.35

    fig, axes = plt.subplots(1, 2, figsize=(10, 4))

    axes[0].bar(x - width / 2, initial_vertices, width, label="Before Pruning", color="#3a7ca5")
    axes[0].bar(x + width / 2, pruned_vertices, width, label="After Pruning", color="#81b29a")
    axes[0].set_xticks(x)
    axes[0].set_xticklabels(difficulties)
    axes[0].set_ylabel("Average Vertex Count")
    axes[0].set_title("Phase 4: Vertex Reduction")
    axes[0].legend()
    axes[0].grid(True, axis="y", alpha=0.25)

    axes[1].bar(x - width / 2, initial_edges, width, label="Before Pruning", color="#c97b63")
    axes[1].bar(x + width / 2, pruned_edges, width, label="After Pruning", color="#84a59d")
    axes[1].set_xticks(x)
    axes[1].set_xticklabels(difficulties)
    axes[1].set_ylabel("Average Edge Count")
    axes[1].set_title("Phase 4: Edge Reduction")
    axes[1].legend()
    axes[1].grid(True, axis="y", alpha=0.25)

    save(fig, "graph_reduction.pdf")


def plot_phase_breakdown() -> None:
    df = load_csv("phase_breakdown.csv")
    if df is None or df.empty:
        return

    grouped = df.groupby("difficulty", sort=False).mean(numeric_only=True)
    labels = [idx.capitalize() for idx in grouped.index]
    phases = [
        ("mask_init_ns", "Mask Init", "#4c956c"),
        ("permutation_ns", "Permutation", "#2c6e91"),
        ("edge_build_ns", "Edge Build", "#f4a259"),
        ("pruning_ns", "Pruning", "#bc4b51"),
        ("extraction_ns", "Extraction", "#8d99ae"),
    ]

    fig, ax = plt.subplots(figsize=(8.5, 4.5))
    bottom = np.zeros(len(labels))

    for column, phase_name, color in phases:
        values = grouped[column].to_numpy() / 1e6
        ax.bar(labels, values, bottom=bottom, label=phase_name, color=color)
        bottom += values

    ax.set_ylabel("Average Time (ms)")
    ax.set_title("End-to-End Phase Breakdown by Difficulty")
    ax.legend(ncol=3)
    ax.grid(True, axis="y", alpha=0.25)

    save(fig, "phase_breakdown.pdf")


def plot_solution_classification(results: dict[str, pd.DataFrame]) -> None:
    labels = []
    unique_counts = []
    ambiguous_counts = []
    unsat_counts = []

    for difficulty in ["easy", "medium", "hard"]:
        if difficulty not in results:
            continue
        df = results[difficulty]
        labels.append(difficulty.capitalize())
        unique_counts.append((df["puzzle_classification"] == "Unique").sum())
        ambiguous_counts.append(df["puzzle_classification"].str.startswith("Ambiguous").sum())
        unsat_counts.append((df["puzzle_classification"] == "Unsolvable").sum())

    x = np.arange(len(labels))
    fig, ax = plt.subplots(figsize=(7.5, 4))
    ax.bar(x, unique_counts, label="Unique", color="#4c956c")
    ax.bar(x, ambiguous_counts, bottom=unique_counts, label="Ambiguous", color="#f4a259")
    ax.bar(
        x,
        unsat_counts,
        bottom=np.array(unique_counts) + np.array(ambiguous_counts),
        label="Unsolvable",
        color="#bc4b51",
    )
    ax.set_xticks(x)
    ax.set_xticklabels(labels)
    ax.set_ylabel("Puzzle Count")
    ax.set_title("Phase 5: Solution Classification")
    ax.legend()
    ax.grid(True, axis="y", alpha=0.25)

    save(fig, "solution_classification.pdf")


def print_summary() -> None:
    summary_path = RESULTS_DIR / "statistics_summary.txt"
    if summary_path.exists():
        print()
        print(summary_path.read_text())


def main() -> None:
    print("Generating figures for the full sudoku_solver paper...")
    results = load_analysis_data()

    if not results:
        print("ERROR: No analysis CSVs found.")
        print("Run: cargo run --release --example analyze_dataset")
        return

    plot_permutation_distribution(results)
    plot_graph_sizes(results)
    plot_phase_breakdown()
    plot_solution_classification(results)
    print_summary()

    print("All figures generated successfully.")
    print(f"Output directory: {FIGURES_DIR}/")


if __name__ == "__main__":
    main()
