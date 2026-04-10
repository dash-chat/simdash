use itertools::Itertools;

use rand::seq::IndexedRandom;
use simdash_ids::NodeId;
use simdash_state::{Graph, node::NodeState};

pub fn strongly_connected_graph<R: Clone>(
    nodes: Vec<NodeState<R>>,
    rng: &mut impl rand::Rng,
    p_extra_edge: f64,
) -> Graph<R> {
    let mut graph = Graph::default();

    for (i, node) in nodes.into_iter().enumerate() {
        graph.add_node(NodeId::from(i as u32), node);
    }

    loop {
        let components = petgraph::algo::kosaraju_scc(graph.graph());

        if components.len() == 1 {
            break;
        }

        components
            .into_iter()
            .chunks(2)
            .into_iter()
            .for_each(|chunk| {
                let sccs: Vec<_> = chunk.collect();
                if sccs.len() == 2 {
                    // if rng.random_range(0..1.0) < p_extra_edge {}
                    graph.add_edge(
                        graph.node_id(*sccs[0].choose(rng).unwrap()).unwrap(),
                        graph.node_id(*sccs[1].choose(rng).unwrap()).unwrap(),
                        (),
                    );
                }
            });
    }
    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strongly_connected_graph() {
        let mut rng = rand::rng();
        let nodes = vec![NodeState::<simdash_state::routing::RoutingState>::default(); 10];
        let graph = strongly_connected_graph(nodes.clone(), &mut rng, 1.5);
        assert_eq!(graph.node_ids().count(), 10);
        assert!(dbg!(graph.graph().edge_count()) > 9);
    }
}
