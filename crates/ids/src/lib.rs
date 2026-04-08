use derive_more::{Deref, From};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref)]
pub struct NodeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref)]
pub struct OpId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref)]
pub struct Size(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Deref)]
pub struct Time(u64);
