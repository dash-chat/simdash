use std::collections::HashMap;

use petgraph::Direction;
use simdash_model_network::Graph;

pub fn outgoing_distribution(graph: &Graph) -> [usize; 5] {
    let mut dist = [0usize; 5];
    for node in graph.graph().node_indices() {
        let out = graph.graph().edges(node).count().min(4);
        dist[out] += 1;
    }
    dist
}

pub fn incoming_distribution(graph: &Graph) -> HashMap<usize, usize> {
    let mut dist = HashMap::new();
    for node in graph.graph().node_indices() {
        let inc = graph
            .graph()
            .edges_directed(node, Direction::Incoming)
            .count();
        *dist.entry(inc).or_insert(0) += 1;
    }
    dist
}
