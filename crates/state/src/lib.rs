#![allow(unused)]

use std::collections::BTreeMap;

pub mod dash;
pub mod graph;
pub mod network;
pub mod node;
pub mod routing;

pub use graph::*;

pub trait ExtractMetadata<T> {
    fn extract_metadata(&self) -> T;
}
