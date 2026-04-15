use std::collections::HashSet;
use std::hash::Hash;

use simdash_types::EncryptedOp;

/// Effect type used by a routing protocol's model.
///
/// The types are generic so that this type can be used both by the protocol model itself,
/// using specific Fx and Data types, and also by the higher level network model,
/// which will use enums of all possible Fx and Data types for all known routing protocols.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingFx<Fx, D: Eq + Hash> {
    /// An effect to take place at the Network level (send data)
    Network(SendFx<D>),
    /// An effect to take place at the Node level (modify Chat state)
    Node(StorageFx),
    /// An effect specific to the protocol itself, which cannot be handled by
    /// the Network but may be used to drive a Behavior
    Protocol(Fx),
}

impl<Fx, D: Eq + Hash> RoutingFx<Fx, D> {
    /// Convenience method to for the [`SendFx::UnicastData`] effect.
    pub fn unicast(to: simdash_types::NodeId, data: HashSet<D>) -> Self {
        RoutingFx::Network(SendFx::UnicastData { to, data })
    }

    /// Convenience method for the [`SendFx::BroadcastData`] effect.
    pub fn broadcast(data: HashSet<D>) -> Self {
        RoutingFx::Network(SendFx::BroadcastData { data })
    }

    /// Convenience method for the [`StorageFx::StoreOps`] effect.
    pub fn store_ops(ops: Vec<EncryptedOp>) -> Self {
        RoutingFx::Node(StorageFx::StoreOps { ops })
    }

    /// Convenience method to wrap a Protocol effect.
    pub fn protocol(fx: Fx) -> Self {
        RoutingFx::Protocol(fx)
    }
}

/// An effect which causes chat app state to change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageFx {
    /// An effect to store operations in the Chat state
    StoreOps {
        /// The ops to be stored
        ops: Vec<EncryptedOp>,
    },
}

/// An effect which causes data to be sent to other node(s) in the network
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendFx<D: Eq + Hash> {
    /// An effect to send data to a specific node
    UnicastData {
        /// The node to send the data to
        to: simdash_types::NodeId,
        /// The protocol wire data to send
        data: HashSet<D>,
    },
    /// An effect to send data to all current neighbors of the node
    BroadcastData {
        /// The protocol wire data to send
        data: HashSet<D>,
    },
}
