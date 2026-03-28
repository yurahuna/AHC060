#!/usr/bin/env bash
set -e

TESTS_DIR="/workspaces/AHC060/tools/in"
MAIN_RS="/workspaces/AHC060/src/main.rs"
BINARY="/workspaces/AHC060/target/release/ahc060"

best_score=0
best_ratio=""

printf "%-16s %s\n" "TARGET_R_RATIO" "Total (150 cases)"
printf "%-16s %s\n" "--------------" "------------------"

for ratio in 0.0 0.1 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9 1.0; do
    sed -i "s/^const TARGET_R_RATIO: f64 = .*/const TARGET_R_RATIO: f64 = $ratio;/" "$MAIN_RS"
    cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null

    total=0
    for f in "$TESTS_DIR"/*.txt; do
        score=$("$BINARY" < "$f" 2>&1 | grep "Score" | awk -F'= ' '{print $2}')
        total=$((total + score))
    done

    printf "%-16s %d\n" "$ratio" "$total"

    if [ "$total" -gt "$best_score" ]; then
        best_score=$total
        best_ratio=$ratio
    fi
done

echo ""
echo "=== Best TARGET_R_RATIO: $best_ratio (total=$best_score) ==="

sed -i "s/^const TARGET_R_RATIO: f64 = .*/const TARGET_R_RATIO: f64 = $best_ratio;/" "$MAIN_RS"
cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null
echo "Updated main.rs to TARGET_R_RATIO=$best_ratio"
