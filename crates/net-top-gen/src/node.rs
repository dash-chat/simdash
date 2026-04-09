use serde::Deserialize;
use simdash_ids::NodeId;
use simdash_state::node::NodeState;

use crate::{ApplyRule, BuildState, Rule};

fn default_strategy() -> String {
    "default".to_string()
}

/// Specification for a subnet of nodes to create.
#[derive(Debug, Clone, Deserialize)]
pub struct NodeSpec {
    #[serde(default = "default_strategy")]
    pub strategy: String,
    #[serde(default)]
    pub subnet: Option<String>,
    pub number: u32,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl ApplyRule for NodeSpec {
    fn apply<R: rand::Rng>(&self, state: &mut BuildState<R>) {
        let node_ids: Vec<NodeId> = (*state.next_id..*state.next_id + self.number)
            .map(NodeId::from)
            .collect();
        *state.next_id += self.number;

        for &node_id in &node_ids {
            state.graph.add_node(node_id, NodeState::default());
        }
        state.all_node_ids.extend_from_slice(&node_ids);
        if let Some(subnet) = &self.subnet {
            state
                .named_subnets
                .entry(subnet.clone())
                .or_default()
                .extend_from_slice(&node_ids);
        }

        if !self.rules.is_empty() {
            let mut scoped_node_ids = node_ids;
            for rule in &self.rules {
                rule.apply(&mut BuildState {
                    graph: state.graph,
                    next_id: state.next_id,
                    all_node_ids: &mut scoped_node_ids,
                    named_subnets: state.named_subnets,
                    rng: state.rng,
                });
            }
        }
    }
}
