use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

use crate::{dash::DashState, routing::RoutingState};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct NodeState<R = RoutingState> {
    pub dash: DashState,
    pub routing: R,
}

impl NodeState<RoutingState> {
    pub fn dash(&self) -> &DashState {
        &self.dash
    }

    pub fn tick(&mut self) {
        self.dash.tick();
        self.routing.tick();
    }
}
