use serde::Deserialize;

use crate::{ApplyRule, BuildState};

/// Sub-rule that applies a tag to all nodes in the current scope.
#[derive(Debug, Clone, Deserialize)]
pub struct NodeTagSpec {
    pub tag: String,
}

impl ApplyRule for NodeTagSpec {
    fn apply<R: rand::Rng>(&self, state: &mut BuildState<R>) {
        for node in &mut state.topology.nodes {
            if state.all_node_ids.contains(&node.id) {
                node.tags.push(self.tag.clone());
            }
        }
    }
}
