use simdash_model_chat::ChatAction;
use simdash_routing_dispatch::RoutingDispatchAction;

/// Actions for the node model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeAction {
    /// A chat-level action
    Chat(ChatAction),
    /// A routing-level action
    Routing(RoutingDispatchAction),
    /// Advance time by one tick
    Tick,
}
