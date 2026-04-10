use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

use crate::{TickableState, chat::ChatState};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct NodeState<RS> {
    pub dash: ChatState,
    pub routing: RS,
}

impl<RS: TickableState> NodeState<RS> {
    pub fn dash(&self) -> &ChatState {
        &self.dash
    }

    pub fn tick(&mut self) {
        self.dash.tick();
        self.routing.tick();
    }
}
