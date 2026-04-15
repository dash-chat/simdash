use simdash_routing_dispatch::RoutingDispatchAction;

use crate::chat::ChatAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeAction {
    Chat(ChatAction),
    Routing(RoutingDispatchAction),
    Tick,
}
