#!/usr/bin/env bash
set -e

TESTS_DIR="/workspaces/AHC060/tools/in"
MAIN_RS="/workspaces/AHC060/src/main.rs"
BINARY="/workspaces/AHC060/target/release/ahc060"

best_score=0
best_prob=""
best_ratio=""

# Header
printf "%-10s %-10s %s\n" "FLIP_PROB" "MAX_R_RATIO" "Total"
printf "%-10s %-10s %s\n" "---------" "-----------" "-----"

for prob in 0.01 0.02 0.05 0.10 0.20; do
    for ratio in 0.1 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9 1.0; do

        sed -i "s/^const FLIP_PROB: f64 = .*/const FLIP_PROB: f64 = $prob;/" "$MAIN_RS"
        sed -i "s/^const MAX_R_RATIO: f64 = .*/const MAX_R_RATIO: f64 = $ratio;/" "$MAIN_RS"

        cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null

        total=0
        for f in "$TESTS_DIR"/*.txt; do
            score=$("$BINARY" < "$f" 2>&1 | grep "Score" | awk -F'= ' '{print $2}')
            total=$((total + score))
        done

        printf "%-10s %-10s %d\n" "$prob" "$ratio" "$total"

        if [ "$total" -gt "$best_score" ]; then
            best_score=$total
            best_prob=$prob
            best_ratio=$ratio
        fi
    done
done

echo ""
echo "=== Best: FLIP_PROB=$best_prob  MAX_R_RATIO=$best_ratio  Total=$best_score ==="

sed -i "s/^const FLIP_PROB: f64 = .*/const FLIP_PROB: f64 = $best_prob;/" "$MAIN_RS"
sed -i "s/^const MAX_R_RATIO: f64 = .*/const MAX_R_RATIO: f64 = $best_ratio;/" "$MAIN_RS"
cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null
echo "Updated main.rs: FLIP_PROB=$best_prob  MAX_R_RATIO=$best_ratio"
