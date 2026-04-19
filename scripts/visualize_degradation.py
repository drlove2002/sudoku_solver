import polars as pl
import matplotlib.pyplot as plt
import seaborn as sns

def main():
    # Load the CSV
    df = pl.read_csv("results/degradation.csv")
    
    # Filter out OOM_PREVENTED rows for visualization purposes
    df = df.filter(pl.col("solutions_count") != "OOM_PREVENTED")
    
    # Cast solutions_count and time_ms to integers
    df = df.with_columns([
        pl.col("solutions_count").cast(pl.Int64),
        pl.col("time_ms").cast(pl.Int64),
        # Create a unique identifier for each puzzle trajectory
        (pl.col("file") + "_" + pl.col("puzzle_id").cast(pl.Utf8)).alias("puzzle_uid")
    ])

    # Convert to pandas for easier seaborn plotting
    pdf = df.to_pandas()

    # Set the style
    sns.set_theme(style="whitegrid")
    
    # Create a figure with two subplots side-by-side
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 7))

    # --- Plot 1: Hints Remaining vs Number of Solutions ---
    sns.lineplot(
        data=pdf,
        x="hints_remaining",
        y="solutions_count",
        hue="puzzle_uid",
        marker="o",
        ax=ax1,
        legend=False,
        alpha=0.7
    )
    ax1.set_title("Explosion of Valid Solutions as Hints are Removed", fontsize=14, fontweight='bold')
    ax1.set_xlabel("Number of Hints Remaining on Board", fontsize=12)
    ax1.set_ylabel("Valid Solutions Found (Log Scale)", fontsize=12)
    
    # Invert x-axis so it reads from more hints (left) to fewer hints (right)
    ax1.set_xlim(ax1.get_xlim()[::-1])
    ax1.set_yscale('log')

    # --- Plot 2: Hints Remaining vs Solve Time (ms) ---
    sns.lineplot(
        data=pdf,
        x="hints_remaining",
        y="time_ms",
        hue="puzzle_uid",
        marker="s",
        ax=ax2,
        legend=False,
        alpha=0.7
    )
    ax2.set_title("Computational Time vs. Hints Remaining", fontsize=14, fontweight='bold')
    ax2.set_xlabel("Number of Hints Remaining on Board", fontsize=12)
    ax2.set_ylabel("Solver Time in ms (Log Scale)", fontsize=12)
    
    # Invert x-axis
    ax2.set_xlim(ax2.get_xlim()[::-1])
    ax2.set_yscale('log')

    plt.suptitle("Sudoku Constraint Degradation: The Cost of Missing Hints", fontsize=18, y=1.02)
    plt.tight_layout()
    plt.savefig("results/degradation_plot.png", dpi=300, bbox_inches='tight')
    print("Plot saved to results/degradation_plot.png")

if __name__ == "__main__":
    main()
