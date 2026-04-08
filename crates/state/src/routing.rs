pub mod flooding;

use crate::Operation;

/// Only one possible routing state for now.
pub type RoutingState = flooding::FloodingState<Operation>;
