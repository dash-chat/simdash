use crate::{DashAction, RoutingAction};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeAction {
    Dash(DashAction),
    Routing(RoutingAction),
}
