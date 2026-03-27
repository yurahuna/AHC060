pub type Cost = i64;

#[derive(Clone, Debug)]
pub struct Graph {
    pub n: usize,
    pub g: Vec<Vec<(usize, Cost)>>,
    pub edges: Vec<(usize, usize, Cost)>,
}

impl Graph {
    pub fn new(n: usize) -> Self {
        Graph {
            n,
            g: vec![Vec::new(); n],
            edges: vec![],
        }
    }

    pub fn from_edges(n: usize, edges: Vec<(usize, usize)>) -> Self {
        let mut graph = Graph::new(n);
        for (u, v) in edges {
            graph.add_edge(u, v, 1);
            graph.add_edge(v, u, 1);
        }
        graph
    }

    pub fn add_edge(&mut self, a: usize, b: usize, c: Cost) {
        self.g.get_mut(a).unwrap().push((b, c));
        self.edges.push((a, b, c));
    }
}
