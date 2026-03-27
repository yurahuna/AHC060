#![allow(non_snake_case, unused_macros)]

mod graph;
mod lowlink;

use graph::Graph;
use lowlink::LowLink;

use delaunator::{triangulate, Point};
use proconio::input;
use rand::prelude::*;
use std::collections::BTreeSet;
use std::ops::RangeBounds;
use svg::node::{
    element::{Circle, Group, Line, Polygon, Rectangle, Style, Text, Title},
    Text as TextNode,
};

const N: usize = 100;
const K: usize = 10;
const T: usize = 10000;

pub trait SetMinMax {
    fn setmin(&mut self, v: Self) -> bool;
    fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T
where
    T: PartialOrd,
{
    fn setmin(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }
    fn setmax(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}

#[macro_export]
macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}

#[derive(Clone, Debug)]
pub struct Input {
    pub n: usize,
    pub m: usize,
    pub k: usize,
    pub t: usize,
    pub edges: Vec<(usize, usize)>,
    pub vertices: Vec<(i32, i32)>,
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {} {} {}", self.n, self.m, self.k, self.t)?;
        for &(u, v) in &self.edges {
            writeln!(f, "{} {}", u, v)?;
        }
        for (x, y) in &self.vertices {
            writeln!(f, "{} {}", x, y)?;
        }
        Ok(())
    }
}

pub fn parse_input(f: &str) -> Input {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        n: usize,
        m: usize,
        k: usize,
        t: usize,
        edges: [(usize, usize); m],
        vertices: [(i32, i32); n],
    }
    Input {
        n,
        m,
        k,
        t,
        edges,
        vertices,
    }
}

pub fn read<T: Copy + PartialOrd + std::fmt::Display + std::str::FromStr, R: RangeBounds<T>>(
    token: Option<&str>,
    range: R,
) -> Result<T, String> {
    if let Some(v) = token {
        if let Ok(v) = v.parse::<T>() {
            if !range.contains(&v) {
                Err(format!("Out of range: {}", v))
            } else {
                Ok(v)
            }
        } else {
            Err(format!("Parse error: {}", v))
        }
    } else {
        Err("Unexpected EOF".to_owned())
    }
}

pub struct Output {
    pub ops: Vec<Op>,
    pub parse_error: Option<String>,
}

pub enum Op {
    Move(usize),
    Flip,
}

pub fn parse_output(input: &Input, f: &str) -> Result<Output, String> {
    let mut ops = vec![];
    let mut parse_error = None;

    let mut tokens = f.split_whitespace();

    while let Some(token) = tokens.next() {
        match read(Some(token), -1..input.n as i32) {
            Ok(op_val) => {
                let op = match op_val {
                    -1 => Op::Flip,
                    _ => Op::Move(op_val as usize),
                };
                ops.push(op);
            }
            Err(err) => {
                parse_error = Some(err);
                break;
            }
        }
    }

    Ok(Output { ops, parse_error })
}

fn dist2(x: (i32, i32), y: (i32, i32)) -> i32 {
    (x.0 - y.0) * (x.0 - y.0) + (x.1 - y.1) * (x.1 - y.1)
}

pub fn gen(seed: u64) -> Input {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);

    let mut vertices = vec![];
    for i in 0..N {
        'outer: loop {
            let x = rng.gen_range(0..=300);
            let y = rng.gen_range(0..=300);
            for j in 0..i {
                if dist2((x, y), vertices[j]) <= 400 {
                    continue 'outer;
                }
            }
            vertices.push((x, y));
            break;
        }
    }

    let points: Vec<Point> = vertices
        .iter()
        .map(|v| Point {
            x: v.0 as f64,
            y: v.1 as f64,
        })
        .collect();
    let triangles = triangulate(&points).triangles;
    assert_eq!(triangles.len() % 3, 0);
    let mut edges = vec![];
    for i in 0..triangles.len() / 3 {
        for j in 0..3 {
            let u = triangles[i * 3 + j];
            let v = triangles[i * 3 + (j + 1) % 3];
            assert_ne!(u, v);
            if u < v {
                edges.push((u, v));
            } else {
                edges.push((v, u));
            }
        }
    }
    edges.sort();
    edges.dedup();
    edges.shuffle(&mut rng);

    let mut res_edges = vec![];

    for i in 0..edges.len() {
        let now_edges = [&res_edges[..], &edges[i + 1..edges.len()]].concat();
        let graph = Graph::from_edges(N, now_edges);
        let lowlink = graph.lowlink();
        if lowlink.bridges.is_empty() && lowlink.articulations.is_empty() {
            let p = rng.gen_range(0.0..1.0);
            if p > 0.7 {
                res_edges.push(edges[i]);
            }
        } else {
            res_edges.push(edges[i]);
        }
    }

    Input {
        n: N,
        m: res_edges.len(),
        k: K,
        t: T,
        edges: res_edges,
        vertices,
    }
}

pub fn compute_score(input: &Input, out: &Output) -> (i64, String) {
    let (mut score, err, _) = compute_score_details(input, &out.ops);
    if err.len() > 0 {
        score = 0;
    }
    (score, err)
}

pub struct State {
    pub score: i64,
    pub pos: usize,
    pub ice_type: Vec<char>, // 'W' (vanilla) or 'R' (strawberry) for ice cream trees (K..N-1), not used for shops (0..K-1)
    pub cone: Vec<char>,     // Current cone (string of 'W' and 'R')
    pub shops: Vec<BTreeSet<Vec<char>>>, // Inventory sets for each shop
    pub visited: Vec<usize>, // History of visited vertices
    pub prev: Option<usize>,  // Previous position (source vertex of last move)
}

impl State {
    fn new(input: &Input) -> Self {
        let mut ice_type = vec![]; // Ice cream trees: 'W' (vanilla) or 'R' (strawberry) or 'S' (shop)
        for i in 0..input.n {
            if i < input.k {
                // Shops: not used, but we'll set to 'S' for clarity
                ice_type.push('S');
            } else {
                // Ice cream trees: initially all 'W' (vanilla)
                ice_type.push('W');
            }
        }
        let shops = vec![BTreeSet::new(); input.k];
        Self {
            score: 0,
            pos: 0,
            ice_type,
            cone: vec![],
            shops,
            visited: vec![0], // Add initial position (0) to history
            prev: None,       // No previous move yet
        }
    }
}

pub fn compute_score_details(input: &Input, out: &[Op]) -> (i64, String, State) {
    let mut g = vec![vec![]; input.n];
    for &(u, v) in &input.edges {
        g[u].push(v);
        g[v].push(u);
    }

    if out.len() > input.t {
        return (0, format!("Too many operations: {}", out.len()), State::new(input));
    }

    let mut state = State::new(input);
    for op in out {
        match op {
            Op::Move(idx) => {
                // Check if the destination is the previous position (forbidden)
                if let Some(prev_pos) = state.prev {
                    if *idx == prev_pos {
                        return (
                            0,
                            format!("Cannot move back to previous position {}", idx),
                            state,
                        );
                    }
                }

                if g[state.pos].contains(idx) {
                    let current_pos = state.pos;
                    state.pos = *idx;
                    state.visited.push(*idx); // Add visited vertex to history
                    state.prev = Some(current_pos); // Update previous position

                    if state.pos < input.k {
                        // Ice cream shop: deliver the cone
                        state.shops[state.pos].insert(state.cone);
                        state.cone = vec![];
                    } else {
                        // Ice cream tree: harvest ice cream
                        state.cone.push(state.ice_type[state.pos]);
                    }
                } else {
                    return (
                        0,
                        format!("Vertex {} is not adjacent to vertex {}", idx, state.pos),
                        state,
                    );
                }
            }
            Op::Flip => {
                if state.ice_type[state.pos] == 'W' {
                    state.ice_type[state.pos] = 'R';
                } else {
                    return (
                        0,
                        format!("Vertex {} must be W", state.pos),
                        state,
                    );
                }
            }
        }
    }

    for i in 0..input.k {
        state.score += state.shops[i].len() as i64;
    }

    (state.score, String::new(), state)
}

/// 0 <= val <= 1
pub fn color(mut val: f64) -> String {
    val.setmin(1.0);
    val.setmax(0.0);
    let (r, g, b) = if val < 0.5 {
        let x = val * 2.0;
        (
            30. * (1.0 - x) + 144. * x,
            144. * (1.0 - x) + 255. * x,
            255. * (1.0 - x) + 30. * x,
        )
    } else {
        let x = val * 2.0 - 1.0;
        (
            144. * (1.0 - x) + 255. * x,
            255. * (1.0 - x) + 30. * x,
            30. * (1.0 - x) + 70. * x,
        )
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        r.round() as i32,
        g.round() as i32,
        b.round() as i32
    )
}

pub fn rect(x: usize, y: usize, w: usize, h: usize, fill: &str) -> Rectangle {
    Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", w)
        .set("height", h)
        .set("fill", fill)
}

pub fn group(title: String) -> Group {
    Group::new().add(Title::new().add(TextNode::new(title)))
}

pub fn vis_default(input: &Input, out: &Output) -> (i64, String, String) {
    let (mut score, err, svg, _state) = vis(input, &out, out.ops.len(), 11);
    if err.len() > 0 {
        score = 0;
    }
    (score, err, svg)
}

pub fn vis(
    input: &Input,
    out: &Output,
    t: usize,
    history_len: usize,
) -> (i64, String, String, State) {
    let W = 800;
    let H = 800;
    let scale = 2.666; // = 800 / 300
    let ops = &out.ops[..t];
    let (score, err, state) = compute_score_details(input, ops);
    let mut doc = svg::Document::new()
        .set("id", "vis")
        .set("viewBox", (-15, -15, W + 30, H + 30))
        .set("width", W + 30)
        .set("height", H + 30)
        .set("style", "background-color:white");
    doc = doc.add(Style::new(format!(
        "text {{text-anchor: middle;dominant-baseline: central;}}"
    )));

    // Draw edges
    for &(u, v) in &input.edges {
        let (x1, y1) = input.vertices[u];
        let (x2, y2) = input.vertices[v];
        let line = Line::new()
            .set("x1", (x1 as f64 * scale) as i32)
            .set("y1", (y1 as f64 * scale) as i32)
            .set("x2", (x2 as f64 * scale) as i32)
            .set("y2", (y2 as f64 * scale) as i32)
            .set("stroke", "#BBB")
            .set("stroke-width", 1);
        doc = doc.add(line);
    }

    // Get history of visited vertices (including current position)
    let actual_history_len = state.visited.len().min(history_len);
    let recent_visited: Vec<usize> = if actual_history_len > 0 {
        state.visited[state.visited.len() - actual_history_len..].to_vec()
    } else {
        vec![]
    };

    // Draw trajectory (path) as lines
    for i in 0..recent_visited.len().saturating_sub(1) {
        let v1 = recent_visited[i];
        let v2 = recent_visited[i + 1];
        let (x1, y1) = input.vertices[v1];
        let (x2, y2) = input.vertices[v2];

        let path_line = Line::new()
            .set("x1", (x1 as f64 * scale) as i32)
            .set("y1", (y1 as f64 * scale) as i32)
            .set("x2", (x2 as f64 * scale) as i32)
            .set("y2", (y2 as f64 * scale) as i32)
            .set("stroke", "#4169E1")
            .set("stroke-width", 4)
            .set("stroke-opacity", "0.6");
        doc = doc.add(path_line);
    }

    // Draw vertices
    for i in 0..input.n {
        let (x, y) = input.vertices[i];
        let cx = (x as f64 * scale) as i32;
        let cy = (y as f64 * scale) as i32;
        let is_current = i == state.pos;

        if i < input.k {
            // Ice cream shop: house shape (unified blue)
            let size = 17;
            let half_size = size / 2;
            let roof_height = 9;

            // House shape as a single polygon (roof + body)
            let house = Polygon::new()
                .set("points", format!(
                    "{},{} {},{} {},{} {},{} {},{}",
                    cx, cy - half_size - roof_height / 2,              // top center (roof peak)
                    cx + half_size, cy - half_size + roof_height / 2, // roof bottom right
                    cx + half_size, cy + half_size,                   // body bottom right
                    cx - half_size, cy + half_size,                   // body bottom left
                    cx - half_size, cy - half_size + roof_height / 2  // roof bottom left
                ))
                .set("fill", "#1e90ff")
                .set("stroke", "none");
            doc = doc.add(house);

            // Circle around current position
            if is_current {
                let highlight_circle = Circle::new()
                    .set("cx", cx)
                    .set("cy", cy)
                    .set("r", 14)
                    .set("fill", "none")
                    .set("stroke", "#4169E1")
                    .set("stroke-width", 2);
                doc = doc.add(highlight_circle);
            }

            // Display vertex number inside the house
            let vertex_text = Text::new()
                .set("x", cx)
                .set("y", cy)
                .set("fill", "white")
                .set("font-size", "12")
                .set("font-weight", "bold")
                .set("text-anchor", "middle")
                .set("dominant-baseline", "central")
                .add(TextNode::new(format!("{}", i)));
            doc = doc.add(vertex_text);

            // Transparent circle for tooltip
            let mut tooltip_circle = Circle::new()
                .set("cx", cx)
                .set("cy", cy)
                .set("r", 14)
                .set("fill", "transparent")
                .set("stroke", "none")
                .set("class", "shop-icon")
                .set("data-shop", i);
            tooltip_circle = tooltip_circle
                .add(Title::new().add(TextNode::new(format!("Ice Cream Shop\nVertex: {}\nInventory: {}", i, state.shops[i].len()))));
            doc = doc.add(tooltip_circle);
        } else {
            // Ice cream tree
            match state.ice_type[i] {
                'W' => {
                    // Vanilla: cream/white circle
                    let radius = 8;

                    let circle = Circle::new()
                        .set("cx", cx)
                        .set("cy", cy)
                        .set("r", radius)
                        .set("fill", "#FFF8DC")
                        .set("stroke", "#DAA520")
                        .set("stroke-width", 1);
                    doc = doc.add(circle);

                    // Circle around current position
                    if is_current {
                        let highlight_circle = Circle::new()
                            .set("cx", cx)
                            .set("cy", cy)
                            .set("r", 14)
                            .set("fill", "none")
                            .set("stroke", "#4169E1")
                            .set("stroke-width", 2);
                        doc = doc.add(highlight_circle);
                    }

                    // Transparent circle for tooltip
                    let mut tooltip_circle = Circle::new()
                        .set("cx", cx)
                        .set("cy", cy)
                        .set("r", 14)
                        .set("fill", "transparent")
                        .set("stroke", "none");
                    tooltip_circle = tooltip_circle
                        .add(Title::new().add(TextNode::new(format!("Ice Cream Tree (Vanilla)\nVertex: {}", i))));
                    doc = doc.add(tooltip_circle);
                }
                'R' => {
                    // Strawberry: red circle
                    let radius = 8;

                    let circle = Circle::new()
                        .set("cx", cx)
                        .set("cy", cy)
                        .set("r", radius)
                        .set("fill", "#FF6B6B");
                    doc = doc.add(circle);

                    // Circle around current position
                    if is_current {
                        let highlight_circle = Circle::new()
                            .set("cx", cx)
                            .set("cy", cy)
                            .set("r", 14)
                            .set("fill", "none")
                            .set("stroke", "#4169E1")
                            .set("stroke-width", 2);
                        doc = doc.add(highlight_circle);
                    }

                    // Transparent circle for tooltip
                    let mut tooltip_circle = Circle::new()
                        .set("cx", cx)
                        .set("cy", cy)
                        .set("r", 14)
                        .set("fill", "transparent")
                        .set("stroke", "none");
                    tooltip_circle = tooltip_circle
                        .add(Title::new().add(TextNode::new(format!("Ice Cream Tree (Strawberry)\nVertex: {}", i))));
                    doc = doc.add(tooltip_circle);
                }
                _ => {}
            }
        }
    }

    (score, err, doc.to_string(), state)
}
