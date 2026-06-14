#!/usr/bin/env bash
# Render Sudanimation videos.
#
# Usage:
#   ./animation/render.sh              # all 6 phases, low quality, max parallel
#   ./animation/render.sh 1            # single phase
#   ./animation/render.sh -qh 1 2 3    # high quality, specific phases
#   ./animation/render.sh --fast 1     # dev mode: 8fps, flush cache, no verbosity
#
# Note: manim is single-threaded per scene. The -j flag parallelizes ACROSS phases.
# For max throughput during dev, render one phase at a time with --fast.

set -euo pipefail

QUALITY="ql"
FPS=""
JOBS=$(nproc)
PHASES=()
EXTRA_FLAGS="--verbosity warning"
NO_CACHE="-c animation/manim.cfg"
MEDIA_BASE="animation/media"

while [[ $# -gt 0 ]]; do
    case "$1" in
        -qh) QUALITY="h"; shift ;;
        -qm) QUALITY="m"; shift ;;
        -ql) QUALITY="l"; shift ;;
        --fast) QUALITY="l"; FPS="8"; EXTRA_FLAGS="--disable_caching --flush_cache --verbosity error"; shift ;;
        -j) JOBS="$2"; shift 2 ;;
        [1-6]) PHASES+=("$1"); shift ;;
        *) echo "Unknown flag: $1"; exit 1 ;;
    esac
done

[[ ${#PHASES[@]} -eq 0 ]] && PHASES=(1 2 3 4 5 6)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

declare -A SCENE_NAMES=(
    [1]=Phase1MaskInit
    [2]=Phase2Deduction
    [3]=Phase3Permutations
    [4]=Phase4Graph
    [5]=Phase5Pruning
    [6]=Phase6Extraction
)

render_one() {
    local phase="$1"
    local scene_file
    scene_file=$(ls "$PROJECT_DIR/animation/scenes/phase${phase}_"*.py 2>/dev/null | head -1)
    if [[ -z "$scene_file" ]]; then
        echo "✗ Phase $phase: no scene file" >&2
        return 1
    fi

    local name="${SCENE_NAMES[$phase]}"
    local fps_flag=""
    [[ -n "$FPS" ]] && fps_flag="--fps $FPS"

    manim $fps_flag -"$QUALITY" \
        --media_dir "$PROJECT_DIR/$MEDIA_BASE" \
        $EXTRA_FLAGS \
        "$scene_file" \
        "$name" \
        > "$PROJECT_DIR/$MEDIA_BASE/phase${phase}.log" 2>&1

    local out="$PROJECT_DIR/$MEDIA_BASE/videos/phase${phase}_"*"/${QUALITY}p"*"/${name}.mp4"
    if ls $out 2>/dev/null; then
        echo "[phase $phase] ✓ $(du -h $(ls -t $out | head -1) | cut -f1)" >&2
    else
        echo "[phase $phase] ✗ FAILED" >&2
    fi
}

export -f render_one
export PROJECT_DIR MEDIA_BASE QUALITY FPS EXTRA_FLAGS

echo "=== ${#PHASES[@]} phase(s) · -q${QUALITY} · ${JOBS} jobs · ${FPS:-default} fps ===" >&2
printf '%s\n' "${PHASES[@]}" | xargs -P "$JOBS" -I{} bash -c 'render_one "$@"' _ {}
echo "=== Done ===" >&2
