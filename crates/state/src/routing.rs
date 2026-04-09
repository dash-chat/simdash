pub mod flooding;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct RoutingState {
    pub flooding: flooding::FloodingState,
}

impl RoutingState {
    pub fn tick(&mut self) {
        self.flooding.tick();
    }
}
