pub mod flooding;

use crate::chat::Operation;

/// Only one possible routing state for now.
pub type RoutingState = flooding::FloodingState;
