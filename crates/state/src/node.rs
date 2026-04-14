//! NodeState mainly exists to unify ChatState and RoutingState.

use serde::{Deserialize, Serialize};
use simdash_routing::RoutingState;
use simdash_types::Tickable;

use crate::chat::ChatState;

/// The state of a node.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NodeState<R = simdash_routing::RoutingState> {
    /// The chat state of the node
    pub chat: ChatState,
    /// The routing state of the node
    pub routing: R,
}

impl NodeState<RoutingState> {
    /// Get a reference to the chat state of the node
    pub fn chat(&self) -> &ChatState {
        &self.chat
    }

    /// Tick the internal states of the node
    pub fn tick(&mut self) {
        self.chat.tick();
        self.routing.tick();
    }
}
