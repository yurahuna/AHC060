use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::{BTreeSet, BinaryHeap, VecDeque};
use std::io::{self, BufRead, Write};
use std::time::Instant;

const FLIP_PROB: f64 = 0.01;
const MAX_R_RATIO: f64 = 1.0;
const MAX_CONE_LEN: usize = 10;
const USE_GREEDY_BFS: bool = true;
const K_PATHS: usize = 15;
const MAX_PRECOMP_STATES: usize = 500;
// Exponent for shop-density weighting when choosing next move.
const ATTRACTION_TEMP: f64 = 2.0;
// Time limit in milliseconds (leave 100ms buffer from 2000ms judge limit).
const TIME_LIMIT_MS: u128 = 1900;

/// Precompute up to `k_max` shortest simple paths from every vertex v to every shop s,
/// not passing through any other shop. Paths are stored as ordered sequences of tree
/// vertex indices (u8) so that ice_type can be resolved at runtime.
/// Returns precomp[v][s] = Vec<(path_len, first_step, trees)> sorted by path_len.
fn precompute_all_paths(
    n: usize,
    k: usize,
    adj: &[Vec<usize>],
    k_max: usize,
    max_states: usize,
) -> Vec<Vec<Vec<(usize, usize, Vec<u8>)>>> {
    let mut precomp = vec![vec![vec![]; k]; n];
    for v in 0..n {
        for s in 0..k {
            if v == s { continue; }
            // heap state: Reverse((len, cur, visited_mask, first_step, trees_so_far))
            let mut heap: BinaryHeap<std::cmp::Reverse<(usize, usize, u128, usize, Vec<u8>)>> =
                BinaryHeap::new();
            heap.push(std::cmp::Reverse((0, v, 1u128 << v, usize::MAX, vec![])));
            let mut expanded = 0usize;
            while let Some(std::cmp::Reverse((len, cur, visited, first, trees))) = heap.pop() {
                expanded += 1;
                if expanded > max_states { break; }
                if cur == s {
                    if first == usize::MAX { continue; }
                    precomp[v][s].push((len, first, trees));
                    if precomp[v][s].len() >= k_max { break; }
                    continue;
                }
                for &next in &adj[cur] {
                    if (next >= k || next == s) && (visited >> next) & 1 == 0 {
                        let new_first = if first == usize::MAX { next } else { first };
                        let new_visited = visited | (1u128 << next);
                        let mut new_trees = trees.clone();
                        if next >= k { new_trees.push(next as u8); }
                        heap.push(std::cmp::Reverse((
                            len + 1, next, new_visited, new_first, new_trees,
                        )));
                    }
                }
            }
        }
    }
    precomp
}

/// Run one simulation trial with the given RNG seed.
/// Returns (moves, score) where each move is -1 (flip) or the target vertex index.
fn simulate(
    n: usize,
    k: usize,
    t: usize,
    adj: &[Vec<usize>],
    precomp: &[Vec<Vec<(usize, usize, Vec<u8>)>>],
    shop_attraction: &[f64],
    flip_prob: f64,
    max_cone_len: usize,
    attraction_temp: f64,
    seed: u64,
) -> (Vec<i32>, usize) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut pos: usize = 0;
    let mut prev: Option<usize> = None;
    let mut cone: Vec<char> = vec![];
    let mut ice_type: Vec<char> = vec!['W'; n];
    let mut shops: Vec<BTreeSet<Vec<char>>> = vec![BTreeSet::new(); k];
    let max_r = ((n - k) as f64 * MAX_R_RATIO).round() as usize;
    let mut r_count = 0usize;
    let mut moves: Vec<i32> = Vec::with_capacity(t);

    let weighted_next = |pool: &[usize], rng: &mut StdRng| -> usize {
        if attraction_temp == 0.0 || pool.len() == 1 {
            return *pool.choose(rng).unwrap();
        }
        let weights: Vec<f64> = pool
            .iter()
            .map(|&v| shop_attraction[v].powf(attraction_temp))
            .collect();
        let total: f64 = weights.iter().sum();
        let mut r = rng.random::<f64>() * total;
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

    for _ in 0..t {
        let neighbors: Vec<usize> = adj[pos]
            .iter()
            .copied()
            .filter(|&v| Some(v) != prev)
            .collect();

        let can_flip = pos >= k && ice_type[pos] == 'W' && r_count < max_r;

        if can_flip && rng.random_bool(flip_prob) {
            moves.push(-1);
            ice_type[pos] = 'R';
            r_count += 1;
        } else {
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

            let mut greedy_step: Option<usize> = None;
            if USE_GREEDY_BFS {
                let mut best_d = usize::MAX;
                for s in 0..k {
                    for (d, first_step, trees) in &precomp[pos][s] {
                        if *d >= best_d { break; }
                        if Some(*first_step) == prev { continue; }
                        let mut proj = cone.clone();
                        for &t in trees {
                            proj.push(ice_type[t as usize]);
                        }
                        if shops[s].contains(&proj) { continue; }
                        best_d = *d;
                        greedy_step = Some(*first_step);
                        break;
                    }
                }
            }

            let mut next = if let Some(step) = greedy_step {
                step
            } else if cone.len() >= max_cone_len {
                let shop_candidates: Vec<usize> =
                    candidates.iter().copied().filter(|&v| v < k).collect();
                if !shop_candidates.is_empty() {
                    weighted_next(&shop_candidates, &mut rng)
                } else {
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

            if !neighbors.contains(&next) {
                next = neighbors[0];
            }

            moves.push(next as i32);
            prev = Some(pos);
            pos = next;
            if pos < k {
                shops[pos].insert(cone.clone());
                cone.clear();
            } else {
                cone.push(ice_type[pos]);
            }
        }
    }

    let score: usize = shops.iter().map(|s| s.len()).sum();
    (moves, score)
}

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

    // Read tunable parameters from environment variables (for parameter sweeping).
    let flip_prob: f64 = std::env::var("FLIP_PROB").ok().and_then(|s| s.parse().ok()).unwrap_or(FLIP_PROB);
    let max_cone_len: usize = std::env::var("MAX_CONE_LEN").ok().and_then(|s| s.parse().ok()).unwrap_or(MAX_CONE_LEN);
    let attraction_temp: f64 = std::env::var("ATTRACTION_TEMP").ok().and_then(|s| s.parse().ok()).unwrap_or(ATTRACTION_TEMP);
    let k_paths: usize = std::env::var("K_PATHS").ok().and_then(|s| s.parse().ok()).unwrap_or(K_PATHS);
    let time_limit_ms: u128 = std::env::var("TIME_LIMIT_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(TIME_LIMIT_MS);

    let precomp = precompute_all_paths(n, k, &adj, k_paths, MAX_PRECOMP_STATES);

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

    // Run multiple trials and keep the best.
    let start = Instant::now();
    let mut best_moves: Vec<i32> = vec![];
    let mut best_score = 0usize;
    let mut trial = 0u64;
    loop {
        let (moves, score) = simulate(n, k, t, &adj, &precomp, &shop_attraction,
            flip_prob, max_cone_len, attraction_temp, trial);
        if score > best_score {
            best_score = score;
            best_moves = moves;
        }
        trial += 1;
        if start.elapsed().as_millis() >= time_limit_ms { break; }
    }

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    for m in &best_moves {
        writeln!(out, "{}", m).unwrap();
    }
    eprintln!("Score = {} (trials={})", best_score, trial);
}
