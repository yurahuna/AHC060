use rand::prelude::*;
use rand::rngs::StdRng;
use std::collections::{BinaryHeap, HashSet, VecDeque};
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

/// Encode cone as a u64 key: bits 0-31 = R-bit flags (bit i=1 means R at position i),
/// bits 32-39 = cone length. Supports cone_len up to 255, val bits 0-31.
#[inline(always)]
fn cone_key(val: u32, len: u8) -> u64 {
    (val as u64) | ((len as u64) << 32)
}

/// Weighted random selection from pool using shop_attraction weights.
/// Uses a stack array for weights to avoid heap allocation.
fn weighted_next(
    pool: &[usize],
    shop_attraction: &[f64],
    attraction_temp: f64,
    rng: &mut StdRng,
) -> usize {
    if attraction_temp == 0.0 || pool.len() == 1 {
        return *pool.choose(rng).unwrap();
    }
    let mut weights = [0f64; 32];
    let mut total = 0f64;
    for (i, &v) in pool.iter().enumerate() {
        let w = shop_attraction[v].powf(attraction_temp);
        weights[i] = w;
        total += w;
    }
    let mut r = rng.random::<f64>() * total;
    let mut chosen = *pool.last().unwrap();
    for (i, &v) in pool.iter().enumerate() {
        r -= weights[i];
        if r <= 0.0 {
            chosen = v;
            break;
        }
    }
    chosen
}

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
///
/// Performance optimisations vs. the naive version:
///   - Cone represented as (val: u32, len: u8) bitmask — zero heap allocation in hot path.
///   - Shop delivered-set stored as HashSet<u64> — O(1) lookup/insert.
///   - ice_type as Vec<bool> (1 byte vs 4 bytes for char).
///   - Neighbour lists stored in fixed-size stack arrays — no Vec alloc per step.
///   - weighted_next uses a stack array for weights.
///   - prev stored as usize (usize::MAX = "no previous") — no Option overhead.
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
    // usize::MAX means "no previous move" (no valid vertex has this index).
    let mut prev: usize = usize::MAX;
    // Cone as bitmask: bit i = 1 → position i is R, 0 → W.
    let mut cone_val: u32 = 0;
    let mut cone_len: u8 = 0;
    // false = W (vanilla), true = R (strawberry).
    let mut ice_is_r: Vec<bool> = vec![false; n];
    // Delivered cones per shop, keyed by cone_key(val, len).
    let mut shops: Vec<HashSet<u64>> = (0..k).map(|_| HashSet::new()).collect();
    let max_r = ((n - k) as f64 * MAX_R_RATIO).round() as usize;
    let mut r_count = 0usize;
    let mut moves: Vec<i32> = Vec::with_capacity(t);

    // Pre-allocated stack buffers — reused every step to avoid Vec allocs.
    // Max degree in these graphs is well below 32.
    let mut nb_buf = [0usize; 32];
    let mut nov_buf = [0usize; 32];
    let mut shop_buf = [0usize; 16];
    let mut bfs_queue: VecDeque<usize> = VecDeque::with_capacity(n);

    for _ in 0..t {
        // Build neighbour list, excluding the vertex we came from.
        let mut nb_len = 0usize;
        for &v in &adj[pos] {
            if v != prev {
                nb_buf[nb_len] = v;
                nb_len += 1;
            }
        }
        let neighbors = &nb_buf[..nb_len];

        let can_flip = pos >= k && !ice_is_r[pos] && r_count < max_r;

        if can_flip && rng.random_bool(flip_prob) {
            moves.push(-1);
            ice_is_r[pos] = true;
            r_count += 1;
        } else {
            let cur_key = cone_key(cone_val, cone_len);

            // Novel neighbours: trees (always novel) or shops where cur cone is new.
            let mut nov_len = 0usize;
            for &v in neighbors {
                if v >= k || !shops[v].contains(&cur_key) {
                    nov_buf[nov_len] = v;
                    nov_len += 1;
                }
            }
            let candidates: &[usize] = if nov_len == 0 { neighbors } else { &nov_buf[..nov_len] };

            let mut greedy_step: Option<usize> = None;
            if USE_GREEDY_BFS {
                let mut best_d = usize::MAX;
                for s in 0..k {
                    for (d, first_step, trees) in &precomp[pos][s] {
                        if *d >= best_d { break; }
                        if *first_step == prev { continue; }
                        // Project cone through the path trees — pure bit arithmetic, no alloc.
                        let mut pv = cone_val;
                        let mut pl = cone_len;
                        for &ti in trees {
                            if ice_is_r[ti as usize] && pl < 32 {
                                pv |= 1u32 << pl;
                            }
                            pl += 1;
                        }
                        if shops[s].contains(&cone_key(pv, pl)) { continue; }
                        best_d = *d;
                        greedy_step = Some(*first_step);
                        break;
                    }
                }
            }

            let mut next = if let Some(step) = greedy_step {
                step
            } else if cone_len as usize >= max_cone_len {
                let mut sc_len = 0usize;
                for &v in candidates {
                    if v < k {
                        shop_buf[sc_len] = v;
                        sc_len += 1;
                    }
                }
                if sc_len > 0 {
                    weighted_next(&shop_buf[..sc_len], shop_attraction, attraction_temp, &mut rng)
                } else {
                    let mut dist = vec![usize::MAX; n];
                    bfs_queue.clear();
                    for s in 0..k {
                        if !shops[s].contains(&cur_key) {
                            dist[s] = 0;
                            bfs_queue.push_back(s);
                        }
                    }
                    while let Some(v) = bfs_queue.pop_front() {
                        for &u in &adj[v] {
                            if dist[u] == usize::MAX {
                                dist[u] = dist[v] + 1;
                                bfs_queue.push_back(u);
                            }
                        }
                    }
                    if let Some(&best) = candidates.iter().min_by_key(|&&v| dist[v]) {
                        best
                    } else {
                        weighted_next(candidates, shop_attraction, attraction_temp, &mut rng)
                    }
                }
            } else {
                weighted_next(candidates, shop_attraction, attraction_temp, &mut rng)
            };

            if !neighbors.contains(&next) {
                next = neighbors[0];
            }

            moves.push(next as i32);
            prev = pos;
            pos = next;
            if pos < k {
                shops[pos].insert(cur_key);
                cone_val = 0;
                cone_len = 0;
            } else {
                if ice_is_r[pos] && cone_len < 32 {
                    cone_val |= 1u32 << cone_len;
                }
                cone_len += 1;
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
