use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use simdash_ids::{NodeId, OpId, Size};

use crate::{ExtractMetadata, routing::flooding::FloodingMetadata};

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
    pub id: OpId,
    pub header: Header,
    pub body: Body,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct DashState {
    pub op_store: BTreeMap<OpId, Operation>,
    pub contacts: BTreeSet<NodeId>,
    pub groups: BTreeSet<GroupId>,
}

impl ExtractMetadata<FloodingMetadata> for Operation {
    fn extract_metadata(&self) -> FloodingMetadata {
        FloodingMetadata {
            size: self.header.payload_size,
        }
    }
}
