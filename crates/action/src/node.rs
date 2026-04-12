use simdash_routing::RoutingAction;

use crate::chat::ChatAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeAction {
    Chat(ChatAction),
    Routing(RoutingAction),
    Tick,
}
