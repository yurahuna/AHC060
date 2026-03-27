use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};

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
    let mut prev: Option<usize> = None; // previous source vertex of last Move action
    let mut cone: Vec<char> = vec![];
    let mut ice_type: Vec<char> = vec!['W'; n]; // only meaningful for k..n-1
    let mut shops: Vec<BTreeSet<Vec<char>>> = vec![BTreeSet::new(); k];

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for _ in 0..t {
        // Collect valid move targets: adjacent and not the prev vertex
        let neighbors: Vec<usize> = adj[pos]
            .iter()
            .copied()
            .filter(|&v| Some(v) != prev)
            .collect();

        // Decide whether to try action 2 first
        // Action 2 is possible only if pos >= k and ice_type[pos] == 'W'
        let can_flip = pos >= k && ice_type[pos] == 'W';

        if can_flip && rng.gen_bool(0.3) {
            // Do action 2: flip to strawberry
            writeln!(out, "-1").unwrap();
            ice_type[pos] = 'R';
        } else {
            // Action 1: move to a random valid neighbor
            let &next = neighbors.choose(&mut rng).unwrap();
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

