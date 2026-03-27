use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::{BTreeSet, VecDeque};
use std::io::{self, BufRead, Write};

const FLIP_PROB: f64 = 0.02;
const MAX_R_RATIO: f64 = 1.0;
const MAX_CONE_LEN: usize = 10;
// Exponent for shop-density weighting when choosing next move.
const ATTRACTION_TEMP: f64 = 0.85;

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

    // Parse coordinates
    let mut coords = vec![(0i64, 0i64); n];
    for i in 0..n {
        let line = lines.next().unwrap().unwrap();
        let mut it = line.split_whitespace();
        let x: i64 = it.next().unwrap().parse().unwrap();
        let y: i64 = it.next().unwrap().parse().unwrap();
        coords[i] = (x, y);
    }

    // Precompute static shop attraction for each vertex: sum of 1/dist to each shop
    let shop_attraction: Vec<f64> = (0..n)
        .map(|v| {
            (0..k)
                .map(|s| {
                    let dx = coords[v].0 - coords[s].0;
                    let dy = coords[v].1 - coords[s].1;
                    let dist = ((dx * dx + dy * dy) as f64).sqrt().max(1.0);
                    1.0 / dist
                })
                .sum::<f64>()
        })
        .collect();

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
            // Avoid shops that already have the current cone
            // If cone is at the limit, prefer shops that don't have the cone yet
            let novel_neighbors: Vec<usize> = neighbors
                .iter()
                .copied()
                .filter(|&v| v >= k || !shops[v].contains(&cone))
                .collect();
            let candidates = if novel_neighbors.is_empty() {
                &neighbors
            } else {
                &novel_neighbors
            };
            // Weighted selection by shop attraction (raised to ATTRACTION_TEMP)
            let weighted_next = |pool: &[usize], rng: &mut StdRng| -> usize {
                if ATTRACTION_TEMP == 0.0 || pool.len() == 1 {
                    return *pool.choose(rng).unwrap();
                }
                let weights: Vec<f64> = pool
                    .iter()
                    .map(|&v| shop_attraction[v].powf(ATTRACTION_TEMP))
                    .collect();
                let total: f64 = weights.iter().sum();
                let mut r = rng.r#gen::<f64>() * total;
                let mut chosen = *pool.last().unwrap();
                for (i, &w) in weights.iter().enumerate() {
                    r -= w;
                    if r <= 0.0 {
                        chosen = pool[i];
                        break;
                    }
                }
                chosen
            };
            let next = if cone.len() >= MAX_CONE_LEN {
                let shop_candidates: Vec<usize> =
                    candidates.iter().copied().filter(|&v| v < k).collect();
                if !shop_candidates.is_empty() {
                    weighted_next(&shop_candidates, &mut rng)
                } else {
                    // BFS from all novel shops to find nearest; follow that direction
                    let mut dist = vec![usize::MAX; n];
                    let mut queue = VecDeque::new();
                    for s in 0..k {
                        if !shops[s].contains(&cone) {
                            dist[s] = 0;
                            queue.push_back(s);
                        }
                    }
                    while let Some(v) = queue.pop_front() {
                        for &u in &adj[v] {
                            if dist[u] == usize::MAX {
                                dist[u] = dist[v] + 1;
                                queue.push_back(u);
                            }
                        }
                    }
                    if let Some(&best) = candidates.iter().min_by_key(|&&v| dist[v]) {
                        best
                    } else {
                        weighted_next(candidates, &mut rng)
                    }
                }
            } else {
                weighted_next(candidates, &mut rng)
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
