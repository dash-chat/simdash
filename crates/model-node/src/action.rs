use derive_more::Debug;
use named_id::RenameAll;
use simdash_model_chat::ChatAction;
use simdash_routing_dispatch::RoutingDispatchAction;

/// Actions for the node model.
#[derive(Debug, Clone, PartialEq, Eq, RenameAll)]
pub enum NodeAction {
    /// A chat-level action
    #[debug("{:?}", _0)]
    Chat(ChatAction),
    /// A routing-level action
    #[debug("{:?}", _0)]
    Routing(RoutingDispatchAction),
    /// Advance time by one tick
    Tick,
}
