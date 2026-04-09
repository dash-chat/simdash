use crate::{chat::ChatAction, routing::RoutingAction};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeAction {
    Chat(ChatAction),
    Routing(RoutingAction),
}
