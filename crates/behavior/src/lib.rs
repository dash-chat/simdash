//! A Behavior is used to drive a model by emitting Actions based on the current state.
//!
//! It's basically a Markov chain, although it can also store memory if it needs to.

/// Standard Behaviors for the chat state.
pub mod chat;
/// Standard Behaviors for the network state.
pub mod network;
/// Standard Behaviors for the node state.
pub mod node;
/// Standard Behaviors for the routing state.
pub mod routing;
