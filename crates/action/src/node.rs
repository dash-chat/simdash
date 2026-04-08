use crate::{dash::DashAction, routing::RoutingAction};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeAction {
    Dash(DashAction),
    Routing(RoutingAction),
}
