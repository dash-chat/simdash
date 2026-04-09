pub mod flooding;

use derive_more::From;

#[derive(Debug, Clone, PartialEq, Eq, Hash, From)]
pub enum RoutingAction {
    Flooding(flooding::FloodingAction),
}
