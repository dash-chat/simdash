//! Functions for generating models and initial states for testing.

use std::collections::HashMap;

use crate::{NetworkModel, NetworkState};
use dash_chat_top_gen::ChatTopology;
use simdash_model_node::{Graph, NodeModel, NodeState};
use simdash_routing_dispatch::RoutingDispatchModel;
use simdash_types::{NodeId, Ticks};

/// Generate a linear chain of `n` nodes.
pub fn linear_chain(n: u32, sync_interval: Ticks) -> (NetworkModel, NetworkState) {
    let mut graph = Graph::new();
    for i in 0..n {
        graph.add_node(NodeId::from(i), NodeState::default());
    }
    for i in 0..n - 1 {
        graph.add_edge(NodeId::from(i), NodeId::from(i + 1), Default::default());
        graph.add_edge(NodeId::from(i + 1), NodeId::from(i), Default::default());
    }

    let nodes: HashMap<NodeId, NodeModel> = (0..n)
        .map(|i| {
            (
                NodeId::from(i),
                NodeModel::new(RoutingDispatchModel::standard(
                    NodeId::from(i),
                    sync_interval,
                )),
            )
        })
        .collect();

    let model = NetworkModel::new(nodes, ChatTopology::empty());
    let state = NetworkState {
        graph,
        topology: Default::default(),
    };

    (model, state)
}
