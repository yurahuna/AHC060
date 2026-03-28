#!/usr/bin/env bash
# Parameter sweep script using TIME_LIMIT_MS=500 on seeds 0000-0019.
# Usage: bash scripts/sweep_params.sh [param_name]
# If no param given, sweeps all: FLIP_PROB, MAX_CONE_LEN, ATTRACTION_TEMP, K_PATHS

set -euo pipefail
cd "$(dirname "$0")/.."

BIN=./target/release/ahc060
SEEDS=($(ls tools/in/00{00..19}.txt 2>/dev/null || ls tools/in/0{000..019}.txt))
TL=500  # ms per trial-loop (shorter for fast sweep)

bench() {
    local -n _env=$1
    local total=0 count=0
    for f in "${SEEDS[@]}"; do
        sc=$(env "${_env[@]}" TIME_LIMIT_MS=$TL $BIN < "$f" 2>&1 >/dev/null | awk -F'= ' '/^Score/{print $2}')
        total=$((total + sc))
        count=$((count + 1))
    done
    echo "$((total / count))"
}

sweep_param() {
    local name=$1; shift
    local best_val="" best_avg=0
    echo "=== Sweeping $name ==="
    for val in "$@"; do
        env_arr=("${name}=${val}")
        total=0; count=0
        for f in "${SEEDS[@]}"; do
            sc=$(env "${env_arr[@]}" TIME_LIMIT_MS=$TL $BIN < "$f" 2>&1 >/dev/null | grep '^Score' | sed 's/Score = \([0-9]*\).*/\1/')
            total=$((total + sc)); count=$((count + 1))
        done
        avg=$((total / count))
        echo "  $name=$val  avg=$avg (total=$total)"
        if [[ $avg -gt $best_avg ]]; then best_avg=$avg; best_val=$val; fi
    done
    echo "  => Best $name=$best_val (avg=$best_avg)"
    echo "$best_val"
}

TARGET=${1:-all}

if [[ $TARGET == all || $TARGET == FLIP_PROB ]]; then
    sweep_param FLIP_PROB 0.005 0.01 0.02 0.03 0.05 0.08 0.12
fi

if [[ $TARGET == all || $TARGET == MAX_CONE_LEN ]]; then
    sweep_param MAX_CONE_LEN 5 7 8 10 12 15 20
fi

if [[ $TARGET == all || $TARGET == ATTRACTION_TEMP ]]; then
    sweep_param ATTRACTION_TEMP 0.3 0.5 0.7 0.85 1.0 1.3 1.5 2.0
fi

if [[ $TARGET == all || $TARGET == K_PATHS ]]; then
    sweep_param K_PATHS 3 5 8 10 15 20
fi
