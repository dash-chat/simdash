use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use simdash_ids::{NodeId, OpId, Size};

pub type GroupId = u64;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Body {
    GroupMessage { group_id: GroupId },
    DirectMessage { recipient: NodeId },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Header {
    pub author: NodeId,
    pub payload_size: Size,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Operation {
    pub header: Header,
    pub body: Body,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct DashState {
    pub op_store: BTreeMap<OpId, Operation>,
    pub contacts: BTreeSet<NodeId>,
    pub groups: BTreeSet<GroupId>,
}

impl crate::flooding::FloodingMetadata for Operation {
    fn size(&self) -> Size {
        self.header.payload_size
    }
}
