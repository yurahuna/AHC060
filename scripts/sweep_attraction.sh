#!/usr/bin/env bash
set -e

TESTS_DIR="/workspaces/AHC060/tools/in"
MAIN_RS="/workspaces/AHC060/src/main.rs"
BINARY="/workspaces/AHC060/target/release/ahc060"

best_score=0
best_temp=""

printf "%-18s %s\n" "ATTRACTION_TEMP" "Total (150 cases)"
printf "%-18s %s\n" "---------------" "------------------"

for temp in 0.0 0.5 1.0 1.5 2.0 3.0 5.0 8.0; do
    sed -i "s/^const ATTRACTION_TEMP: f64 = .*/const ATTRACTION_TEMP: f64 = $temp;/" "$MAIN_RS"
    cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null

    total=0
    for f in "$TESTS_DIR"/*.txt; do
        score=$("$BINARY" < "$f" 2>&1 | grep "Score" | awk -F'= ' '{print $2}')
        total=$((total + score))
    done

    printf "%-18s %d\n" "$temp" "$total"

    if [ "$total" -gt "$best_score" ]; then
        best_score=$total
        best_temp=$temp
    fi
done

echo ""
echo "=== Best ATTRACTION_TEMP: $best_temp (total=$best_score) ==="

sed -i "s/^const ATTRACTION_TEMP: f64 = .*/const ATTRACTION_TEMP: f64 = $best_temp;/" "$MAIN_RS"
cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null
echo "Updated main.rs to ATTRACTION_TEMP=$best_temp"
