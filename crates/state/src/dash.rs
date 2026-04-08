use std::collections::{BTreeMap, BTreeSet};

use simdash_ids::{NodeId, OpId, Size};

use crate::FloodingMetadata;

pub type GroupId = u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Header {
    pub author: NodeId,
    pub payload_size: Size,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Operation {
    pub header: Header,
    // pub body: Option,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DashState {
    pub op_store: BTreeMap<OpId, Operation>,
    pub contacts: BTreeSet<NodeId>,
    pub groups: BTreeSet<GroupId>,
}

impl FloodingMetadata for Operation {
    fn size(&self) -> Size {
        self.header.payload_size
    }
}
