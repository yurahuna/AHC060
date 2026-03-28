#!/usr/bin/env bash
set -e

TESTS_DIR="/workspaces/AHC060/tools/in"
MAIN_RS="/workspaces/AHC060/src/main.rs"
BINARY="/workspaces/AHC060/target/release/ahc060"

best_score=0
best_len=""

printf "%-14s %s\n" "MAX_CONE_LEN" "Total"
printf "%-14s %s\n" "------------" "-----"

for len in 3 5 7 10 13 15 20 30 50 9999999; do
    # Replace the constant (handle usize::MAX specially)
    if [ "$len" = "9999999" ]; then
        sed -i 's/^const MAX_CONE_LEN: usize = .*/const MAX_CONE_LEN: usize = usize::MAX;/' "$MAIN_RS"
    else
        sed -i "s/^const MAX_CONE_LEN: usize = .*/const MAX_CONE_LEN: usize = $len;/" "$MAIN_RS"
    fi

    cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null

    total=0
    for f in "$TESTS_DIR"/*.txt; do
        score=$("$BINARY" < "$f" 2>&1 | grep "Score" | awk -F'= ' '{print $2}')
        total=$((total + score))
    done

    label="$len"
    [ "$len" = "9999999" ] && label="usize::MAX"
    printf "%-14s %d\n" "$label" "$total"

    if [ "$total" -gt "$best_score" ]; then
        best_score=$total
        best_len=$len
    fi
done

echo ""
echo "=== Best MAX_CONE_LEN: $best_len (total=$best_score) ==="

if [ "$best_len" = "9999999" ]; then
    sed -i 's/^const MAX_CONE_LEN: usize = .*/const MAX_CONE_LEN: usize = usize::MAX;/' "$MAIN_RS"
else
    sed -i "s/^const MAX_CONE_LEN: usize = .*/const MAX_CONE_LEN: usize = $best_len;/" "$MAIN_RS"
fi
cargo build --release --quiet --manifest-path /workspaces/AHC060/Cargo.toml 2>/dev/null
echo "Updated main.rs to MAX_CONE_LEN=$best_len"
