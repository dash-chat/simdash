use simdash_routing::RoutingActionDispatch;

use crate::chat::ChatAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeAction {
    Chat(ChatAction),
    Routing(RoutingActionDispatch),
    Tick,
}
