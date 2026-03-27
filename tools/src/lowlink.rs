use super::graph::Graph;

#[derive(Clone, Debug)]
pub struct LowLinkData {
    ord: Vec<usize>,
    low: Vec<usize>,
    pub bridges: Vec<(usize, usize)>,
    pub articulations: Vec<usize>,
}

pub trait LowLink {
    fn lowlink(&self) -> LowLinkData;
}

impl LowLink for Graph {
    fn lowlink(&self) -> LowLinkData {
        let n = self.n;
        let mut visited = vec![false; n];
        let ord = vec![usize::MAX; n];
        let low = vec![usize::MAX; n];
        let mut order = 0;

        let mut lowlink = LowLinkData {
            ord,
            low,
            bridges: vec![],
            articulations: vec![]
        };

        for i in 0..n {
            if !visited[i] {
                build_lowlink(i, usize::MAX, &self, &mut visited, &mut order, &mut lowlink);
            }
        }
        lowlink
    }
}

fn build_lowlink(now: usize, prev: usize, graph: &Graph, visited: &mut Vec<bool>, order: &mut usize, lowlink: &mut LowLinkData) {
    if visited[now] {
        return;
    }
    visited[now] = true;
    lowlink.ord[now] = *order;
    lowlink.low[now] = *order;
    *order += 1;
    let mut is_articulation = false;
    let mut tmp_bool = false;
    let mut cnt = 0;

    for &(nxt, _) in &graph.g[now] {
        if nxt == prev {
            let nowtmp = tmp_bool;
            tmp_bool = true;
            if !nowtmp {
                continue;
            }
        }

        if !visited[nxt] {
            cnt += 1;
            build_lowlink(nxt, now, graph, visited, order, lowlink);
            lowlink.low[now] = lowlink.low[now].min(lowlink.low[nxt]);
            is_articulation = is_articulation || prev != usize::MAX && lowlink.low[nxt] >= lowlink.ord[now];
            if lowlink.ord[now] < lowlink.low[nxt] {
                lowlink.bridges.push((now.min(nxt), now.max(nxt)));
            }
        } else {
            lowlink.low[now] = lowlink.low[now].min(lowlink.ord[nxt]);
        }
    }
    is_articulation = is_articulation || prev == usize::MAX && cnt >= 2;
    if is_articulation {
        lowlink.articulations.push(now);
    }
}
