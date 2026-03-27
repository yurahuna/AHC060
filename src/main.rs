use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};

const FLIP_PROB: f64 = 0.02;
const MAX_R_RATIO: f64 = 1.0;
const MAX_CONE_LEN: usize = 10;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    // Parse first line: N M K T
    let first = lines.next().unwrap().unwrap();
    let mut it = first.split_whitespace();
    let n: usize = it.next().unwrap().parse().unwrap();
    let m: usize = it.next().unwrap().parse().unwrap();
    let k: usize = it.next().unwrap().parse().unwrap();
    let t: usize = it.next().unwrap().parse().unwrap();

    // Parse edges
    let mut adj = vec![vec![]; n];
    for _ in 0..m {
        let line = lines.next().unwrap().unwrap();
        let mut it = line.split_whitespace();
        let a: usize = it.next().unwrap().parse().unwrap();
        let b: usize = it.next().unwrap().parse().unwrap();
        adj[a].push(b);
        adj[b].push(a);
    }

    // Parse coordinates (not used)
    for _ in 0..n {
        lines.next().unwrap().unwrap();
    }

    // State
    let mut rng = StdRng::seed_from_u64(42);
    let mut pos: usize = 0;
    let mut prev: Option<usize> = None;
    let mut cone: Vec<char> = vec![];
    let mut ice_type: Vec<char> = vec!['W'; n];
    let mut shops: Vec<BTreeSet<Vec<char>>> = vec![BTreeSet::new(); k];
    let max_r = ((n - k) as f64 * MAX_R_RATIO).round() as usize;
    let mut r_count = 0usize;

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for _ in 0..t {
        // Collect valid move targets: adjacent and not the prev vertex
        let neighbors: Vec<usize> = adj[pos]
            .iter()
            .copied()
            .filter(|&v| Some(v) != prev)
            .collect();

        // Action 2 is possible only if pos >= k, ice_type[pos] == 'W', and under the R cap
        let can_flip = pos >= k && ice_type[pos] == 'W' && r_count < max_r;

        if can_flip && rng.gen_bool(FLIP_PROB) {
            // Do action 2: flip to strawberry
            writeln!(out, "-1").unwrap();
            ice_type[pos] = 'R';
            r_count += 1;
        } else {
            // Action 1: move to a random valid neighbor
            // If cone is at the limit, prefer neighbors that are shops
            let next = if cone.len() >= MAX_CONE_LEN {
                let shop_neighbors: Vec<usize> =
                    neighbors.iter().copied().filter(|&v| v < k).collect();
                if !shop_neighbors.is_empty() {
                    *shop_neighbors.choose(&mut rng).unwrap()
                } else {
                    *neighbors.choose(&mut rng).unwrap()
                }
            } else {
                *neighbors.choose(&mut rng).unwrap()
            };
            writeln!(out, "{}", next).unwrap();
            prev = Some(pos);
            pos = next;
            if pos < k {
                // Shop: deliver cone
                shops[pos].insert(cone.clone());
                cone.clear();
            } else {
                // Ice cream tree: harvest
                cone.push(ice_type[pos]);
            }
        }
    }

    // Compute final score (for stderr reporting)
    let score: usize = shops.iter().map(|s| s.len()).sum();
    eprintln!("Score = {}", score);
}
