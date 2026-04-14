//! Common Action and Effect types usable by all routing protocols.
//!
//! This models the things that routing protocols have in common:
//! - protocol models can cause data to be sent to other nodes
//! - protocol models can receive data from other nodes
//! - protocol models can cause their host node to store data

mod action;
mod fx;

pub use action::*;
pub use fx::*;
