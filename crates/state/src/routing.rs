pub mod flooding;

use crate::dash::Operation;

/// Only one possible routing state for now.
pub type RoutingState = flooding::FloodingState<Operation>;
