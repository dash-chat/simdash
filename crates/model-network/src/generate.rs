//! Functions for generating models and initial states for testing.

use std::collections::BTreeSet;
use std::collections::HashMap;

use crate::NetworkModel;
use crate::NetworkState;
use crate::state::Graph;
use named_id::Nameable;
use simdash_model_node::{NodeModel, NodeState};
use simdash_routing_dispatch::RoutingDispatchModel;
use simdash_types::{NodeId, Ticks};

/// Generate a linear chain of `n` nodes.
pub fn linear_chain(length: u32) -> Graph {
    let mut graph = Graph::new();
    for i in 0..length {
        graph.add_node(NodeId::from(i), NodeState::default());
    }
    for i in 0..length - 1 {
        graph.add_edge(NodeId::from(i), NodeId::from(i + 1), Default::default());
        graph.add_edge(NodeId::from(i + 1), NodeId::from(i), Default::default());
    }

    graph
}

/// For each length, create a bidirection path between the first and last node
/// with the given number of hops.
pub fn n_paths(hops: impl IntoIterator<Item = u32>) -> (Graph, (NodeId, NodeId), Vec<Vec<NodeId>>) {
    let mut graph = Graph::new();
    let hops = hops.into_iter().collect::<Vec<_>>();
    assert!(
        hops.iter().all(|&len| len >= 2),
        "all hop counts must be at least 2"
    );
    // let max_len = lengths.iter().max().unwrap();
    let alpha = NodeId::from(0).with_name("alpha");
    graph.add_node(alpha, NodeState::default());
    let mut paths = vec![];

    for length in hops {
        let mut path = vec![];
        for _ in 0..length - 1 {
            let id = NodeId::from(graph.node_ids().count() as u32);
            graph.add_node(id, NodeState::default());
            path.push(id);
        }
        paths.push(path);
    }
    let omega = NodeId::from(graph.node_ids().count() as u32).with_name("omega");
    graph.add_node(omega, NodeState::default());

    for path in paths.iter() {
        for i in 0..path.len() - 1 {
            graph.add_edge(path[i], path[i + 1], Default::default());
            graph.add_edge(path[i + 1], path[i], Default::default());
        }
        graph.add_edge(alpha, path[0], Default::default());
        graph.add_edge(path[0], alpha, Default::default());
        graph.add_edge(path[path.len() - 1], omega, Default::default());
        graph.add_edge(omega, path[path.len() - 1], Default::default());
    }

    assert_eq!(
        graph.node_ids().collect::<BTreeSet<_>>(),
        (0..=*omega).map(NodeId::from).collect::<BTreeSet<_>>()
    );
    (graph, (alpha, omega), paths)
}

/// Create a network model and state from a graph.
pub fn make_model(graph: Graph, sync_interval: Ticks) -> (NetworkModel, NetworkState) {
    let models: HashMap<NodeId, NodeModel> = graph
        .node_ids()
        .map(|id| {
            (
                id,
                NodeModel::new(RoutingDispatchModel::standard(id, sync_interval)),
            )
        })
        .collect();
    let model = NetworkModel::new(models);
    let state = NetworkState {
        graph,
        topology: Default::default(),
    };
    (model, state)
}

#[test]
fn smoke_test_n_paths() {
    let (graph, _, _) = n_paths([2, 3, 4]);
    let g = graph.graph().map(|n, _| n.index(), |_, _| "");
    let dot = petgraph::dot::Dot::with_config(&g, &[petgraph::dot::Config::EdgeNoLabel]);
    println!("\n\n{}", dot);
}
