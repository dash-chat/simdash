use derive_more::{Debug, Display};
use named_id::RenameAll;
use simdash_model_chat::ChatAction;
use simdash_routing_dispatch::RoutingDispatchAction;

/// Actions for the node model.
#[derive(Debug, Display, Clone, PartialEq, Eq, RenameAll)]
pub enum NodeAction {
    /// A chat-level action
    // #[debug("{:?}", _0.renamed())]
    Chat(ChatAction),
    /// A routing-level action
    // #[debug("{:?}", _0.renamed())]
    Routing(RoutingDispatchAction),
    /// Advance time by one tick
    Tick,
}
