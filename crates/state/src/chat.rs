use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use simdash_ids::{GroupId, NodeId, OpId, Operation, Size};

use simdash_ids::ExtractMetadata;

use crate::routing::flooding::FloodingMetadata;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ChatState {
    pub op_store: BTreeMap<OpId, Operation>,
    pub contacts: BTreeSet<NodeId>,
    pub groups: BTreeSet<GroupId>,
    /// Maximum total payload size this node can store. None means unlimited.
    #[serde(default)]
    pub storage_capacity: Option<Size>,
}

impl ChatState {
    /// ChatState is timeless
    pub fn tick(&mut self) {}

    pub fn stored_size(&self) -> Size {
        self.op_store
            .values()
            .map(|op| op.header.payload_size)
            .fold(Size::from(0u64), |acc, s| Size::from(*acc + *s))
    }

    pub fn store_op(&mut self, op: Operation) -> anyhow::Result<()> {
        if self.op_store.contains_key(&op.id) {
            anyhow::bail!("Op already exists: {:?}", op.id);
        }
        if let Some(capacity) = self.storage_capacity {
            let new_size = *self.stored_size() + *op.header.payload_size;
            if new_size > *capacity {
                anyhow::bail!(
                    "Storage capacity exceeded: {} + {} > {}",
                    *self.stored_size(),
                    *op.header.payload_size,
                    *capacity
                );
            }
        }
        self.op_store.insert(op.id, op);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use simdash_ids::{Body, Header};

    use super::*;

    fn make_op(id: u32, size: u64) -> Operation {
        Operation {
            id: OpId::from(id),
            header: Header {
                author: NodeId::from(1),
                payload_size: Size::from(size),
            },
            body: Body::DirectMessage {
                recipient: NodeId::from(2),
            },
        }
    }

    #[test]
    fn store_op_within_capacity() {
        let mut state = ChatState {
            storage_capacity: Some(Size::from(200u64)),
            ..Default::default()
        };
        assert!(state.store_op(make_op(1, 100)).is_ok());
        assert!(state.store_op(make_op(2, 100)).is_ok());
        assert_eq!(state.op_store.len(), 2);
    }

    #[test]
    fn store_op_exceeding_capacity_is_rejected() {
        let mut state = ChatState {
            storage_capacity: Some(Size::from(150u64)),
            ..Default::default()
        };
        assert!(state.store_op(make_op(1, 100)).is_ok());
        assert!(state.store_op(make_op(2, 100)).is_err());
        assert_eq!(state.op_store.len(), 1);
    }

    #[test]
    fn store_op_exactly_at_capacity() {
        let mut state = ChatState {
            storage_capacity: Some(Size::from(200u64)),
            ..Default::default()
        };
        assert!(state.store_op(make_op(1, 100)).is_ok());
        assert!(state.store_op(make_op(2, 100)).is_ok());
        assert!(state.store_op(make_op(3, 1)).is_err());
        assert_eq!(state.op_store.len(), 2);
    }

    #[test]
    fn store_op_unlimited_capacity() {
        let mut state = ChatState::default();
        assert!(state.storage_capacity.is_none());
        for i in 0..100 {
            assert!(state.store_op(make_op(i, 1000)).is_ok());
        }
        assert_eq!(state.op_store.len(), 100);
    }

    #[test]
    fn store_op_duplicate_is_rejected() {
        let mut state = ChatState {
            storage_capacity: Some(Size::from(500u64)),
            ..Default::default()
        };
        assert!(state.store_op(make_op(1, 100)).is_ok());
        assert!(state.store_op(make_op(1, 100)).is_err());
        assert_eq!(state.op_store.len(), 1);
    }
}

impl ExtractMetadata<FloodingMetadata> for Operation {
    fn extract_metadata(&self) -> FloodingMetadata {
        FloodingMetadata {
            size: self.header.payload_size,
        }
    }
}
