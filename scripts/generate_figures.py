#!/usr/bin/env nix-shell
#!nix-shell -p python312Packages.matplotlib python312Packages.numpy -i python3
"""Generate publication-quality visualizations from experiment_results.csv.

Style:
- All time axes in microseconds (matches xlsx layout)
- Clean log-scale tick labels (no `10^x` overlap)
- Hatched empty bars for OOM categories instead of zero-height bars
- Speedup annotations inside plot area, never clipped
- Per-chart sample-size annotations to expose small-N caveats
- Legends outside plot area to free vertical space
"""
import csv, os
from collections import defaultdict, Counter
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.patches import Patch
from matplotlib.ticker import LogFormatterSciNotation, ScalarFormatter

CSV = 'results/experiment_results.csv'
OUT_DIR = 'results/figures'
os.makedirs(OUT_DIR, exist_ok=True)

plt.rcParams.update({
    'figure.dpi': 150, 'savefig.dpi': 200,
    'font.size': 9, 'axes.titlesize': 11,
    'axes.labelsize': 10, 'legend.fontsize': 8, 'figure.titlesize': 13,
    'axes.spines.top': False, 'axes.spines.right': False,
})

C = {'h_on': '#2196F3', 'h_off': '#FF5722', 'oom': '#D32F2F',
     'mask': '#E91E63', 'heuristic': '#9C27B0', 'perm': '#3F51B5',
     'graph': '#00BCD4', 'prune': '#4CAF50', 'extract': '#FFC107'}
SZ = {'9x9': '9×9', '16x16': '16×16', '25x25': '25×25'}
CAT = {'very_easy': 'Very Easy', 'easy': 'Easy', 'medium': 'Medium',
       'hard': 'Hard', 'very_hard': 'Very Hard', 'ambiguous': 'Ambiguous'}
CAT_ORDER = ['very_easy', 'easy', 'medium', 'hard', 'very_hard', 'ambiguous']
PH = ['mask_ns', 'heuristic_ns', 'perm_ns', 'graph_ns', 'prune_ns', 'extract_ns']
PH_LABEL = ['Mask', 'Heuristic', 'Perm.', 'Graph', 'Pruning', 'Extract.']
PH_C = [C['mask'], C['heuristic'], C['perm'], C['graph'], C['prune'], C['extract']]
SIZES = ['9x9', '16x16', '25x25']
BOARD_CELLS = {'9x9': 81, '16x16': 256, '25x25': 625}

# Per-size category lists — drop categories that have no data for that size
def categories_with_data(groups, size, h='true'):
    return [c for c in CAT_ORDER if (size, c, h) in groups]

# ----------------------------------------------------------------------------
# Load data
# ----------------------------------------------------------------------------
all_rows = list(csv.DictReader(open(CSV)))
valid = [r for r in all_rows if r['classification'] not in ('Panic', 'OOM')]

groups = defaultdict(list)
for r in valid:
    groups[(r['size'], r['category'], r['heuristic_on'])].append(r)

ooms = [r for r in all_rows if 'OomAt' in r.get('phase_progress', '')]
oom_cats = Counter(r['category'] for r in ooms)
oom_phases = Counter(r['phase_progress'] for r in ooms)


def avg(lst):
    return sum(lst) / len(lst) if lst else 0


agg = {}
for sz in SIZES:
    for cat in CAT_ORDER:
        for h in ['true', 'false']:
            grp = groups.get((sz, cat, h), [])
            if not grp:
                continue
            agg[(sz, cat, h)] = {
                'n': len(grp),
                't_us': avg([float(r['total_ns']) / 1e3 for r in grp]),
                'p': [avg([float(r[ph]) / 1e3 for r in grp]) for ph in PH],
                'mem': [avg([float(r.get(k, 0)) for r in grp]) for k in
                        ['mask_mem', 'heuristic_mem', 'perm_mem', 'graph_mem', 'prune_mem']],
                'cells': avg([int(r['cells_filled']) for r in grp]),
            }


def is_oom(sz, cat):
    """A category is OOM-only if h=off has no data but at least one h=off row OOMed."""
    return (sz, cat, 'false') not in agg and oom_cats.get(cat, 0) > 0


def style_log_axis(ax, axis='y'):
    """Clean log-scale ticks: show `1, 10, 100, 1000` not `10^0, 10^1, ...`."""
    def fmt(v, _pos):
        if v <= 0:
            return ''
        exp = int(np.log10(v))
        if 10 ** exp == v:
            return f'$10^{{{exp}}}$' if exp != 0 else '1'
        return ''
    if axis == 'y':
        ax.yaxis.set_major_formatter(plt.FuncFormatter(fmt))
        ax.yaxis.set_minor_formatter(plt.NullFormatter())
    else:
        ax.xaxis.set_major_formatter(plt.FuncFormatter(fmt))
        ax.xaxis.set_minor_formatter(plt.NullFormatter())


def annotate_oom(ax, x, ymax, count, color):
    """Hatched empty bar + OOM label above it."""
    ax.bar(x, ymax * 0.05, width=0.7, facecolor='none',
           edgecolor=color, hatch='//', linewidth=1.2, zorder=2)
    ax.text(x, ymax * 0.4, f'OOM\n({count})', ha='center', va='center',
            fontsize=7, color=color, fontweight='bold',
            bbox=dict(boxstyle='round,pad=0.25', facecolor='#FFEBEE',
                      edgecolor=color, alpha=0.9))


# ============================================================================
# Chart 1: Total time h_on vs h_off (per-size subplots)
# ============================================================================
fig, axes = plt.subplots(1, 3, figsize=(15, 5.2))
for i, sz in enumerate(SIZES):
    ax = axes[i]
    cats = categories_with_data(groups, sz, 'true')
    x = np.arange(len(cats))
    w = 0.36

    von = [agg[(sz, c, 'true')]['t_us'] for c in cats]
    voff, oom_x = [], []
    for c in cats:
        if (sz, c, 'false') in agg and agg[(sz, c, 'false')]['t_us'] > 0:
            voff.append(agg[(sz, c, 'false')]['t_us'])
        else:
            voff.append(0)
            if is_oom(sz, c):
                oom_x.append((x[len(voff) - 1] if voff else None, c))

    ax.bar(x - w / 2, von, w, label='With heuristics', color=C['h_on'],
           edgecolor='white', linewidth=0.5)
    bars_off = ax.bar(x + w / 2, voff, w, label='Without heuristics',
                      color=C['h_off'], alpha=0.75, edgecolor='white', linewidth=0.5)

    ymax = max([v for v in von + voff if v > 0] + [1.0]) * 8
    ax.set_ylim(bottom=max(min([v for v in von + voff if v > 0] + [1.0]) * 0.3, 0.1), top=ymax)

    for j, (vo, vf) in enumerate(zip(von, voff)):
        if vf > vo * 1.3 and vf > 0:
            ax.annotate(f'{vf / vo:.0f}×', xy=(x[j] + w / 2, vf),
                        xytext=(0, 6), textcoords='offset points',
                        ha='center', fontsize=7, color=C['oom'], fontweight='bold')
        if vo > 0:
            ax.annotate(f'{vo / 1e3:.2f}' if vo >= 1e3 else f'{vo:.0f}',
                        xy=(x[j] - w / 2, vo), xytext=(0, 4),
                        textcoords='offset points', ha='center', fontsize=6.5,
                        color=C['h_on'])

    for j, c in enumerate(cats):
        if (sz, c, 'false') not in agg and oom_cats.get(c, 0) > 0:
            annotate_oom(ax, x[j] + w / 2, ymax, oom_cats[c], C['oom'])

    n_on = sum(agg.get((sz, c, 'true'), {}).get('n', 0) for c in cats)
    ax.set_title(f'{SZ[sz]}  (n={n_on:,} puzzles)', fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels([CAT[c] for c in cats], rotation=25, ha='right', fontsize=8)
    ax.set_ylabel('Total time (µs)')
    ax.set_yscale('log')
    style_log_axis(ax, 'y')
    ax.legend(fontsize=7, loc='upper left', framealpha=0.95)
    ax.grid(axis='y', alpha=0.25, linestyle='--', linewidth=0.5)

plt.suptitle('Total Solve Time: With vs Without Constraint Propagation',
             fontweight='bold', y=1.0)
fig.text(0.5, -0.01, 'Time measured end-to-end. OOM bars indicate the h=off run '
         'exhausted memory (no time recorded).', ha='center', fontsize=8, style='italic')
plt.tight_layout()
plt.savefig(f'{OUT_DIR}/01_total_time.png', bbox_inches='tight')
plt.close()
print('1/6: Total time')

# ============================================================================
# Chart 2: Per-phase breakdown (stacked, with heuristics)
# ============================================================================
fig, axes = plt.subplots(1, 3, figsize=(15, 5.2))
for i, sz in enumerate(SIZES):
    ax = axes[i]
    cats = categories_with_data(groups, sz, 'true')
    x = np.arange(len(cats))
    bot = np.zeros(len(cats))
    for pi in range(6):
        vals = [agg[(sz, c, 'true')]['p'][pi] for c in cats]
        ax.bar(x, vals, 0.6, bottom=bot, label=PH_LABEL[pi],
               color=PH_C[pi], alpha=0.88, edgecolor='white', linewidth=0.5)
        bot += vals

    for j, total in enumerate(bot):
        if total > 0:
            label = f'{total / 1e3:.2f} ms' if total >= 1e3 else f'{total:.0f} µs'
            ax.text(x[j], total * 1.15, label, ha='center', fontsize=7,
                    fontweight='bold', color='#333')

    n = sum(agg.get((sz, c, 'true'), {}).get('n', 0) for c in cats)
    ax.set_title(f'{SZ[sz]}  (n={n:,})', fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels([CAT[c] for c in cats], rotation=25, ha='right', fontsize=8)
    ax.set_ylabel('Time (µs)')
    ax.set_yscale('log')
    style_log_axis(ax, 'y')
    ax.grid(axis='y', alpha=0.25, linestyle='--', linewidth=0.5)

handles, labels = axes[0].get_legend_handles_labels()
fig.legend(handles, labels, loc='lower center', ncol=6, fontsize=8,
           bbox_to_anchor=(0.5, -0.02), frameon=False)
plt.suptitle('Per-Phase Time Breakdown (Constraint Propagation Enabled)',
             fontweight='bold', y=1.0)
plt.tight_layout(rect=[0, 0.04, 1, 1])
plt.savefig(f'{OUT_DIR}/02_phase_breakdown.png', bbox_inches='tight')
plt.close()
print('2/6: Phase breakdown')

# ============================================================================
# Chart 3: Memory h_on vs h_off
# ============================================================================
fig, axes = plt.subplots(1, 3, figsize=(15, 5.2))
for i, sz in enumerate(SIZES):
    ax = axes[i]
    cats = categories_with_data(groups, sz, 'true')
    x = np.arange(len(cats))
    w = 0.36

    mon = [agg[(sz, c, 'true')]['mem'][2] + agg[(sz, c, 'true')]['mem'][3] for c in cats]
    moff = []
    for c in cats:
        if (sz, c, 'false') in agg:
            moff.append(agg[(sz, c, 'false')]['mem'][2] + agg[(sz, c, 'false')]['mem'][3])
        else:
            moff.append(0)

    ax.bar(x - w / 2, [v / 1e6 for v in mon], w, label='With heuristics',
           color=C['h_on'], edgecolor='white', linewidth=0.5)
    ax.bar(x + w / 2, [v / 1e6 for v in moff], w, label='Without heuristics',
           color=C['h_off'], alpha=0.75, edgecolor='white', linewidth=0.5)

    ymax = max([v for v in mon + moff if v > 0] + [1.0]) / 1e6 * 6
    ax.set_ylim(bottom=max(min([v for v in mon + moff if v > 0] + [1.0]) / 1e6 * 0.3, 1e-4), top=ymax)

    for j, (mo, mf) in enumerate(zip(mon, moff)):
        if mf > mo * 2 and mf > 0 and mo > 0:
            ax.annotate(f'{mf / mo:.0f}×', xy=(x[j] + w / 2, mf / 1e6),
                        xytext=(0, 6), textcoords='offset points',
                        ha='center', fontsize=7, color=C['oom'], fontweight='bold')

    for j, c in enumerate(cats):
        if (sz, c, 'false') not in agg and oom_cats.get(c, 0) > 0:
            annotate_oom(ax, x[j] + w / 2, ymax, oom_cats[c], C['oom'])

    n_on = sum(agg.get((sz, c, 'true'), {}).get('n', 0) for c in cats)
    ax.set_title(f'{SZ[sz]}  (n={n_on:,})', fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels([CAT[c] for c in cats], rotation=25, ha='right', fontsize=8)
    ax.set_ylabel('Permutation + Graph Memory (MB)')
    ax.set_yscale('log')
    style_log_axis(ax, 'y')
    ax.legend(fontsize=7, loc='upper left', framealpha=0.95)
    ax.grid(axis='y', alpha=0.25, linestyle='--', linewidth=0.5)

plt.suptitle('Peak Memory: With vs Without Constraint Propagation',
             fontweight='bold', y=1.0)
fig.text(0.5, -0.01, 'Memory = permutation nodes + graph edges. OOM bars indicate '
         'the h=off run exhausted memory.', ha='center', fontsize=8, style='italic')
plt.tight_layout()
plt.savefig(f'{OUT_DIR}/03_memory.png', bbox_inches='tight')
plt.close()
print('3/6: Memory')

# ============================================================================
# Chart 4: Speedup factors (horizontal bar, sorted by size then category)
# ============================================================================
fig, ax = plt.subplots(figsize=(13, 7))
rows = []
for sz in SIZES:
    for cat in CAT_ORDER:
        if (sz, cat, 'true') not in agg:
            continue
        to = agg[(sz, cat, 'true')]['t_us']
        n_on = agg[(sz, cat, 'true')]['n']
        n_off = agg.get((sz, cat, 'false'), {}).get('n', 0)
        oom_n = oom_cats.get(cat, 0)
        if (sz, cat, 'false') in agg and agg[(sz, cat, 'false')]['t_us'] > 0:
            tf = agg[(sz, cat, 'false')]['t_us']
            rows.append((sz, cat, to, tf, n_on, n_off, oom_n, 'measured'))
        elif oom_n > 0:
            rows.append((sz, cat, to, 0, n_on, 0, oom_n, 'oom'))

# Sort: size order, then by speedup descending within size
size_idx = {s: i for i, s in enumerate(SIZES)}
rows.sort(key=lambda r: (size_idx[r[0]], -r[3] / r[2] if r[3] > 0 else 0))

labels = [f"{SZ[r[0]]}  {CAT[r[1]]}" for r in rows]
vals = [r[3] / r[2] if r[3] > 0 else 0 for r in rows]
statuses = [r[7] for r in rows]

bar_colors = []
for v in vals:
    if v <= 0:
        bar_colors.append('#BDBDBD')
    elif v < 3:
        bar_colors.append(C['h_on'])
    elif v < 100:
        bar_colors.append('#FF9800')
    else:
        bar_colors.append(C['oom'])

# Compute a max x value that gives headroom for labels
max_speedup = max(v for v in vals if v > 0) if any(v > 0 for v in vals) else 1
xmax = max_speedup * 1.6
ax.set_xlim(0.5, xmax)
ax.set_xscale('log')

y_pos = np.arange(len(rows))
plot_vals = [max(v, 0.55) if v > 0 else 0.55 for v in vals]
ax.barh(y_pos, plot_vals, color=bar_colors, alpha=0.88,
        edgecolor='white', linewidth=0.5)

ax.set_yticks(y_pos)
ax.set_yticklabels(labels, fontsize=8.5)
ax.invert_yaxis()
ax.axvline(x=1, color='gray', linestyle='--', alpha=0.5, linewidth=1)
ax.set_xlabel('Speedup factor (time without heuristics ÷ time with heuristics)')
style_log_axis(ax, 'x')
ax.grid(axis='x', alpha=0.25, linestyle='--', linewidth=0.5)

for i, (v, r) in enumerate(zip(vals, rows)):
    if v > 0:
        label = f'{v:.0f}×  (n={r[4]})'
        color = C['oom'] if v >= 100 else 'black'
    else:
        label = f'OOM ({r[6]})'
        color = C['oom']
    ax.text(plot_vals[i] * 1.08, i, label, va='center', fontsize=8,
            fontweight='bold', color=color)

ax.legend(handles=[
    Patch(facecolor=C['h_on'], label='<3×  (minor)'),
    Patch(facecolor='#FF9800', label='3–100×  (significant)'),
    Patch(facecolor=C['oom'], label='≥100×  (critical)'),
    Patch(facecolor='#BDBDBD', label='OOM  (no h=off data)'),
], fontsize=8, loc='lower right', framealpha=0.95, title='Speedup tier')

plt.suptitle('Constraint Propagation Speedup (Higher = More Benefit)',
             fontweight='bold', y=1.0)
fig.text(0.5, -0.01, 'Sample size (n) shown after each value. 16×16 and 25×25 categories '
         'have small N — interpret with care.', ha='center', fontsize=8, style='italic')
plt.tight_layout()
plt.savefig(f'{OUT_DIR}/04_speedup_factors.png', bbox_inches='tight')
plt.close()
print('4/6: Speedup factors')

# ============================================================================
# Chart 5: OOM phase + category breakdown
# ============================================================================
if ooms:
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(13, 5))

    # Left: phases — horizontal bar
    ph_labels_raw = list(oom_phases.keys())
    ph_display = []
    for p in ph_labels_raw:
        name = p.replace('OomAt(', '').replace(')', '').replace('_', ' ')
        ph_display.append(name.title())
    ph_counts = list(oom_phases.values())
    ph_colors = ['#D32F2F', '#FF5722', '#FF9800'][:len(ph_display)]
    ph_y = np.arange(len(ph_display))
    bars = ax1.barh(ph_y, ph_counts, color=ph_colors, alpha=0.9,
                    edgecolor='white', linewidth=0.5)
    ax1.set_yticks(ph_y)
    ax1.set_yticklabels(ph_display, fontsize=9)
    ax1.invert_yaxis()
    ax1.set_xlabel('OOM count')
    ax1.set_title('Phase Where OOM Occurred', fontweight='bold')
    ax1.grid(axis='x', alpha=0.25, linestyle='--', linewidth=0.5)
    total_ph = sum(ph_counts)
    for i, (c, bar) in enumerate(zip(ph_counts, bars)):
        ax1.text(c + 0.15, i, f'{c}  ({c / total_ph * 100:.0f}%)',
                 va='center', fontsize=8.5, fontweight='bold')
    ax1.set_xlim(0, max(ph_counts) * 1.35)

    # Right: by category (only 25x25 in practice, but filter dynamically)
    oom_cat_data = [(CAT[c], oom_cats[c]) for c in CAT_ORDER if oom_cats.get(c, 0) > 0]
    cat_labels = [d[0] for d in oom_cat_data]
    cat_counts = [d[1] for d in oom_cat_data]
    cat_x = np.arange(len(cat_labels))
    bars2 = ax2.bar(cat_x, cat_counts, color='#D32F2F', alpha=0.85,
                    edgecolor='white', linewidth=0.5)
    ax2.set_title('OOM Count by Puzzle Category', fontweight='bold')
    ax2.set_ylabel('OOM count')
    ax2.set_xticks(cat_x)
    ax2.set_xticklabels(cat_labels, rotation=25, ha='right', fontsize=8)
    ax2.grid(axis='y', alpha=0.25, linestyle='--', linewidth=0.5)
    for i, c in enumerate(cat_counts):
        ax2.text(i, c + 0.3, str(c), ha='center', fontsize=9, fontweight='bold')
    ax2.set_ylim(0, max(cat_counts) * 1.25)

    plt.suptitle('OOM Analysis — All 25×25 Puzzles, Without Heuristics',
                 fontweight='bold', y=1.0)
    plt.tight_layout()
    plt.savefig(f'{OUT_DIR}/05_oom.png', bbox_inches='tight')
    plt.close()
    print('5/6: OOM analysis')
else:
    print('5/6: OOM (none)')

# ============================================================================
# Chart 6: % empty cells filled by constraint propagation
# ============================================================================
fig, axes = plt.subplots(1, 3, figsize=(15, 5.2))
for i, sz in enumerate(SIZES):
    ax = axes[i]
    cats = categories_with_data(groups, sz, 'true')
    x = np.arange(len(cats))
    pcts = []
    for c in cats:
        grp = groups.get((sz, c, 'true'), [])
        if not grp:
            pcts.append(0)
            continue
        n = len(grp)
        clues_per = [float(r.get('clues', 0)) for r in grp]
        cells_per = [int(r.get('cells_filled', 0)) for r in grp]
        valid_clues = [cl for cl in clues_per if cl > 0]
        if not valid_clues:
            pcts.append(0)
            continue
        avg_clues = sum(valid_clues) / len(valid_clues)
        avg_filled = sum(cells_per) / n
        empty = BOARD_CELLS[sz] - avg_clues
        pcts.append((avg_filled / empty) * 100 if empty > 0 else 0)

    colors = ['#9C27B0' if p < 100 else '#4CAF50' for p in pcts]
    ax.bar(x, pcts, 0.6, color=colors, alpha=0.88,
           edgecolor='white', linewidth=0.5)
    ax.axhline(y=100, color='#4CAF50', linestyle=':', alpha=0.6, linewidth=1)

    for j, p in enumerate(pcts):
        if p >= 100:
            label = f'{p:.0f}%\n(solved)'
        else:
            label = f'{p:.0f}%'
        ax.text(j, p + 2.5, label, ha='center', fontsize=7,
                color='#333', fontweight='bold')

    n = sum(agg.get((sz, c, 'true'), {}).get('n', 0) for c in cats)
    ax.set_title(f'{SZ[sz]}  (n={n:,})', fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels([CAT[c] for c in cats], rotation=25, ha='right', fontsize=8)
    ax.set_ylabel('Empty cells filled by constraint propagation (%)')
    ax.set_ylim(0, 115)
    ax.grid(axis='y', alpha=0.25, linestyle='--', linewidth=0.5)

plt.suptitle('Constraint Propagation Coverage: % of Empty Cells Resolved',
             fontweight='bold', y=1.0)
fig.text(0.5, -0.01, 'Green bars at 100% mean constraint propagation alone solved the '
         'puzzle. Categories with no data for a given size are omitted.',
         ha='center', fontsize=8, style='italic')
plt.tight_layout()
plt.savefig(f'{OUT_DIR}/06_cells_filled.png', bbox_inches='tight')
plt.close()
print('6/6: Cells filled')

print(f'\nAll charts saved to {OUT_DIR}/:')
for f in sorted(os.listdir(OUT_DIR)):
    print(f'  {f} ({os.path.getsize(f"{OUT_DIR}/{f}") / 1024:.0f} KB)')
