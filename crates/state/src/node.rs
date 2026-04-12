use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};
use simdash_routing::RoutingState;

use crate::chat::ChatState;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NodeState<R = simdash_routing::RoutingState> {
    pub chat: ChatState,
    pub routing: R,
}

impl NodeState<RoutingState> {
    pub fn dash(&self) -> &ChatState {
        &self.chat
    }

    pub fn tick(&mut self) {
        self.chat.tick();
        self.routing.tick();
    }
}
