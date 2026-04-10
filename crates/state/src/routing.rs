use serde::{Deserialize, Serialize};

#[deprecated]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct RoutingState {}

impl RoutingState {
    #[deprecated]
    pub fn tick(&mut self) {
        // self.flooding.tick();
    }
}
