//! A naive sample flooding routing protocol.
//!
//! See [`model`] for a description of the protocol's model.

mod action;
mod behavior;
mod fx;
mod model;
mod state;

pub use action::*;
pub use behavior::*;
pub use fx::*;
pub use model::*;
pub use state::*;
