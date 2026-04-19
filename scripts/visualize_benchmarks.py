import polars as pl
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np
import sys

def main():
    try:
        df = pl.read_csv("results/benchmark_timings.csv")
    except Exception as e:
        print(f"Could not load benchmark timings: {e}")
        sys.exit(1)

    # Calculate average timings per type/difficulty
    agg_df = df.group_by(["puzzle_type", "difficulty"]).agg([
        pl.col("mask_init_ns").mean() / 1e6,
        pl.col("heuristic_ns").mean() / 1e6,
        pl.col("permutation_ns").mean() / 1e6,
        pl.col("edge_build_ns").mean() / 1e6,
        pl.col("pruning_ns").mean() / 1e6,
        pl.col("extraction_ns").mean() / 1e6,
        pl.col("total_ns").mean() / 1e6
    ]).sort(["puzzle_type", "difficulty"])
    
    # Convert to pandas for easier stacked bar plotting with matplotlib/seaborn
    pdf = agg_df.to_pandas()

    pdf['category'] = pdf['puzzle_type'] + ' - ' + pdf['difficulty']
    
    # The phases to stack
    phases = ["mask_init_ns", "heuristic_ns", "permutation_ns", "edge_build_ns", "pruning_ns", "extraction_ns"]
    colors = sns.color_palette("deep", len(phases))

    # --- Plot 1: Stacked Bar Chart of the 5 phases ---
    fig, ax = plt.subplots(figsize=(14, 8))
    
    bottoms = np.zeros(len(pdf))
    
    for i, phase in enumerate(phases):
        ax.bar(
            pdf['category'], 
            pdf[phase], 
            bottom=bottoms, 
            label=phase.replace("_ns", "").replace("_", " ").title(),
            color=colors[i],
            edgecolor="white"
        )
        bottoms += pdf[phase]

    ax.set_title("Average Solver Time by Phase (ms)", fontsize=16, fontweight='bold')
    ax.set_ylabel("Time in milliseconds", fontsize=12)
    ax.set_xlabel("Puzzle Category", fontsize=12)
    
    # Use log scale since "hard" / "ambiguous" puzzles will blow out the Y axis
    ax.set_yscale('log')
    ax.legend(title="Solver Phase")
    
    plt.xticks(rotation=45, ha="right")
    plt.tight_layout()
    plt.savefig("results/benchmark_plot.png", dpi=300, bbox_inches="tight")
    print("Plot generated successfully at results/benchmark_plot.png")

if __name__ == "__main__":
    main()