use derive_more::From;

use crate::chat::ChatAction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeAction<RoutingAction> {
    Chat(ChatAction),
    Routing(RoutingAction),
    Tick,
}
