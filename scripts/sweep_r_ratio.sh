#!/usr/bin/env bash
set -e

TESTS_DIR="/workspaces/AHC060/tools/in"
MAIN_RS="/workspaces/AHC060/src/main.rs"
BINARY="/workspaces/AHC060/target/release/ahc060"
OUT_DIR="/workspaces/AHC060/tools/out"
mkdir -p "$OUT_DIR"

best_score=0
best_ratio=""

declare -A results

for ratio in 0.1 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9 1.0; do
    echo "=== Testing MAX_R_RATIO=$ratio ==="

    sed -i "s/^const MAX_R_RATIO: f64 = .*/const MAX_R_RATIO: f64 = $ratio;/" "$MAIN_RS"

    cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null

    scores=()
    total=0
    for f in "$TESTS_DIR"/*.txt; do
        score=$("$BINARY" < "$f" 2>&1 | grep "Score" | awk -F'= ' '{print $2}')
        scores+=("$score")
        total=$((total + score))
    done

    # Statistics
    sorted=($(printf '%s\n' "${scores[@]}" | sort -n))
    n=${#sorted[@]}
    min=${sorted[0]}
    max=${sorted[$((n-1))]}
    mid_idx=$((n/2))
    median=${sorted[$mid_idx]}
    avg=$((total / n))

    echo "  Total=$total  Avg=$avg  Median=$median  Min=$min  Max=$max"
    results[$ratio]=$total

    if [ "$total" -gt "$best_score" ]; then
        best_score=$total
        best_ratio=$ratio
    fi
done

echo ""
echo "=== Summary ==="
for ratio in 0.1 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9 1.0; do
    echo "  ratio=$ratio  total=${results[$ratio]}"
done
echo ""
echo "=== Best MAX_R_RATIO: $best_ratio (total=$best_score) ==="

sed -i "s/^const MAX_R_RATIO: f64 = .*/const MAX_R_RATIO: f64 = $best_ratio;/" "$MAIN_RS"
cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null
echo "Updated main.rs to use MAX_R_RATIO=$best_ratio"
