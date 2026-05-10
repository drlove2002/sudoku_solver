import polars as pl
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
import sys

def main():
    try:
        raw_df = pl.read_csv("results/benchmark_timings.csv")
        long_df = pl.read_csv("results/benchmark_timings_long.csv")
    except Exception as e:
        print(f"Could not load benchmark timings: {e}")
        sys.exit(1)

    sns.set_theme(style="whitegrid")

    phase_df = (
        long_df
        .filter(
            (pl.col("benchmark_mode") == "full_solve_stats")
            & (pl.col("phase") != "total")
        )
        .group_by(["observed_classification", "difficulty", "phase"])
        .agg(pl.col("time_ms").median().alias("median_time_ms"))
        .sort(["observed_classification", "difficulty", "phase"])
    )

    phase_pdf = phase_df.to_pandas()
    phase_pdf["category"] = (
        phase_pdf["observed_classification"] + " - " + phase_pdf["difficulty"]
    )
    category_order = list(dict.fromkeys(phase_pdf["category"]))
    phases = ["mask_init", "heuristic", "permutation", "edge_build", "pruning", "extraction"]
    colors = sns.color_palette("deep", len(phases))

    fig, ax = plt.subplots(figsize=(14, 8))
    bottoms = np.zeros(len(category_order))

    for i, phase in enumerate(phases):
        values = []
        for category in category_order:
            match = phase_pdf[
                (phase_pdf["category"] == category) & (phase_pdf["phase"] == phase)
            ]
            values.append(match["median_time_ms"].iloc[0] if not match.empty else 0.0)

        ax.bar(
            category_order,
            values,
            bottom=bottoms,
            label=phase.replace("_", " ").title(),
            color=colors[i],
            edgecolor="white",
        )
        bottoms += np.array(values)

    ax.set_title("Median Full-Solve Time by Phase (ms)", fontsize=16, fontweight='bold')
    ax.set_ylabel("Time in milliseconds", fontsize=12)
    ax.set_xlabel("Observed Classification", fontsize=12)
    ax.set_yscale('log')
    ax.legend(title="Solver Phase")
    plt.xticks(rotation=45, ha="right")
    plt.tight_layout()
    plt.savefig("results/benchmark_plot.png", dpi=300, bbox_inches="tight")
    plt.close(fig)

    total_df = (
        long_df
        .filter(pl.col("phase") == "total")
        .group_by(["benchmark_mode", "observed_classification", "difficulty"])
        .agg(pl.col("time_ms").median().alias("median_time_ms"))
        .sort(["benchmark_mode", "observed_classification", "difficulty"])
    )
    total_pdf = total_df.to_pandas()

    fig, ax = plt.subplots(figsize=(14, 8))
    sns.barplot(
        data=total_pdf,
        x="difficulty",
        y="median_time_ms",
        hue="observed_classification",
        ax=ax,
    )
    ax.set_title("Median Total Time by Classification (ms)", fontsize=16, fontweight='bold')
    ax.set_ylabel("Median total time (ms)", fontsize=12)
    ax.set_xlabel("Difficulty", fontsize=12)
    ax.set_yscale("log")
    plt.tight_layout()
    plt.savefig("results/benchmark_total_plot.png", dpi=300, bbox_inches="tight")
    plt.close(fig)

    classification_df = (
        raw_df
        .filter(pl.col("benchmark_mode") == "full_solve_stats")
        .group_by(["dataset_label", "observed_classification"])
        .agg(pl.len().alias("puzzle_count"))
        .sort(["dataset_label", "observed_classification"])
    )
    classification_pdf = classification_df.to_pandas()

    fig, ax = plt.subplots(figsize=(12, 7))
    sns.barplot(
        data=classification_pdf,
        x="dataset_label",
        y="puzzle_count",
        hue="observed_classification",
        ax=ax,
    )
    ax.set_title("Observed Classifications per Dataset Label", fontsize=16, fontweight='bold')
    ax.set_ylabel("Puzzle count", fontsize=12)
    ax.set_xlabel("Dataset label", fontsize=12)
    plt.tight_layout()
    plt.savefig("results/benchmark_classification_plot.png", dpi=300, bbox_inches="tight")
    plt.close(fig)

    print("Generated results/benchmark_plot.png")
    print("Generated results/benchmark_total_plot.png")
    print("Generated results/benchmark_classification_plot.png")

if __name__ == "__main__":
    main()
