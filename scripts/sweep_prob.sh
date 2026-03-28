#!/usr/bin/env bash
set -e

TESTS_DIR="/workspaces/AHC060/tools/in"
MAIN_RS="/workspaces/AHC060/src/main.rs"
BINARY="/workspaces/AHC060/target/release/ahc060"

best_score=0
best_prob=""

for prob in 0.00 0.01 0.02 0.03 0.04 0.05 0.06 0.07 0.08 0.09 0.10; do
    echo "=== Testing prob=$prob ==="

    # Replace gen_bool parameter in main.rs
    sed -i "s/rng\.gen_bool([0-9.]*)/rng.gen_bool($prob)/g" "$MAIN_RS"

    # Build
    cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null

    total=0
    for f in "$TESTS_DIR"/*.txt; do
        score=$("$BINARY" < "$f" 2>&1 | grep "Score" | awk -F'= ' '{print $2}')
        total=$((total + score))
    done

    echo "  Total score: $total"

    if [ "$total" -gt "$best_score" ]; then
        best_score=$total
        best_prob=$prob
    fi
done

echo ""
echo "=== Best probability: $best_prob (score=$best_score) ==="

# Set best probability
sed -i "s/rng\.gen_bool([0-9.]*)/rng.gen_bool($best_prob)/g" "$MAIN_RS"
echo "Updated main.rs to use prob=$best_prob"
