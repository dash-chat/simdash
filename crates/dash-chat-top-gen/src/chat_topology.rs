use std::collections::BTreeMap;

use net_topology::NetTopology;
use simdash_ids::NodeId;
use simdash_state::{chat::ChatState, graph::Graph};

/// Intermediate representation of a network topology with dash-chat state added.
#[derive(Clone)]
pub struct ChatTopology {
    pub net: NetTopology,
    pub chat_states: BTreeMap<NodeId, ChatState>,
}

impl ChatTopology {
    pub fn empty() -> Self {
        Self {
            net: NetTopology::new(),
            chat_states: BTreeMap::new(),
        }
    }

    pub fn into_graph(self, seed: u64) -> Graph {
        let mut graph = self.net.into_graph(seed);
        for (node_id, chat_state) in self.chat_states {
            graph
                .update_node(node_id, |mut n| {
                    n.chat = chat_state;
                    Ok((n, ()))
                })
                .ok();
        }
        graph
    }
}
