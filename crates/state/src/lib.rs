#![allow(unused)]

use std::collections::BTreeMap;

pub mod chat;
pub mod graph;
pub mod network;
pub mod node;

pub use graph::*;

pub trait ExtractMetadata<T> {
    fn extract_metadata(&self) -> T;
}

/// Routing state types that can be advanced by one tick.
pub trait TickableState {
    fn tick(&mut self);
}
