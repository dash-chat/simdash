//! State types for the node and graph level.
//!
//! `NodeState` represents all state for a single node.
//! `Graph` and `EdgeState` live here (rather than in `simdash-model-network`)
//! so that topology generators can depend on them without creating a circular
//! dependency with the network model.

use derive_more::Display;
use serde::{Deserialize, Serialize};
use simdash_model_chat::ChatState;
use simdash_routing_dispatch::RoutingDispatchState;

// ── NodeState ──────────────────────────────────────────────────────────────

/// The state of a node.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NodeState<R = RoutingDispatchState> {
    /// The chat state of the node
    pub chat: ChatState,
    /// The routing state of the node
    pub routing: R,
}

impl NodeState<RoutingDispatchState> {
    /// Get a reference to the chat state of the node
    pub fn chat(&self) -> &ChatState {
        &self.chat
    }
}

// ── EdgeState ──────────────────────────────────────────────────────────────

/// The state of an edge in the graph.
#[derive(Debug, Display, Clone, Default, Serialize, Deserialize)]
#[display("{topology_id}")]
pub struct EdgeState {
    /// The ID of the topology this edge is part of
    pub topology_id: u32,
    /// The number of ticks remaining for this edge
    pub remaining_ticks: Option<u32>,
}

impl EdgeState {
    /// Create a new EdgeState with the given topology ID and longevity.
    pub fn new(topology_id: u32, longevity: u32) -> Self {
        Self {
            topology_id,
            remaining_ticks: if longevity == 0 {
                None
            } else {
                Some(longevity)
            },
        }
    }
}
