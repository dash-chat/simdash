mod routing_table;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use derive_more::Deref;
use serde::{Deserialize, Serialize};
use simdash_types::*;

use crate::{Announce, Op, SourceId, StateVector, state::routing_table::RoutingTable};

/// The Stig protocol's state simply tracks the number of ticks since the last time it was triggered.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// The buffer of operations to sync
    pub op_buffer: OpBuffer,
    /// Cached state_vector received from other nodes
    pub routes: RoutingTable,
}

impl State {
    pub fn from_ops(ops: BTreeSet<Op>) -> Self {
        let mut op_buffer: OpBuffer = Default::default();
        for op in ops {
            op_buffer.store_op(op);
        }

        Self {
            op_buffer,
            routes: Default::default(),
        }
    }
}

impl GetRoutingOpInfo for State {
    fn routing_op_infos(&self) -> impl Iterator<Item = RoutingOpInfo> {
        self.op_buffer.ops.values().flat_map(|topics| {
            topics.iter().flat_map(|(_, ops)| {
                ops.iter().map(|op| RoutingOpInfo {
                    id: op.encrypted.op_id(),
                    size: op.encrypted.size(),
                })
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Deref)]
pub struct StoredOp {
    #[deref]
    pub op: Op,
    pub since_last_send: Ticks,
}

impl StoredOp {
    pub fn new(op: Op) -> Self {
        Self {
            op,
            since_last_send: 0.into(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpBuffer {
    ops: HashMap<TopicId, HashMap<LogId, BTreeSet<StoredOp>>>,
}

impl OpBuffer {
    pub fn store_op(&mut self, op: Op) {
        let log = self
            .ops
            .entry(op.metadata.topic_id)
            .or_default()
            .entry(op.metadata.log_id)
            .or_default();
        let metadata = op.metadata.clone();
        let existing = log.insert(StoredOp::new(op));
        if existing {
            tracing::warn!("Op already exists: metadata={:?}", metadata);
        } else {
            tracing::info!("Stored new op: metadata={:?}", metadata);
        }
    }

    pub fn topics(&self) -> impl Iterator<Item = (TopicId, StateVector)> {
        self.ops
            .iter()
            .map(|(topic_id, _)| (topic_id.clone(), self.state_vector(topic_id.clone())))
    }

    pub fn state_vector(&self, topic_id: TopicId) -> StateVector {
        let Some(ops) = self.ops.get(&topic_id) else {
            return StateVector::new(std::iter::empty());
        };
        StateVector::new(ops.iter().flat_map(|(log_id, ops)| {
            ops.iter()
                .map(|op| (log_id.clone(), LogHeight::from_seq(op.metadata.seq)))
        }))
    }

    pub fn diff(&self, topic_id: TopicId, other: &StateVector) -> BTreeSet<Op> {
        let Some(ops) = self.ops.get(&topic_id) else {
            return BTreeSet::new();
        };
        ops.iter()
            .flat_map(|(log_id, ops)| {
                let height = other.get(log_id).map(|h| **h).unwrap_or(0);
                ops.iter()
                    .map(|op| &op.op)
                    .filter(move |op| *op.metadata.seq >= height)
                    .cloned()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;

    use simdash_types::util::basic_op;

    use crate::Metadata;

    use super::*;

    fn make_op(log_id: LogId, seq: Seq) -> Op {
        let op = basic_op(0);
        Op {
            metadata: Metadata {
                topic_id: 0.into(),
                log_id,
                seq,
            },
            encrypted: EncryptedOp::from(op),
        }
    }

    #[test]
    fn test_state_vector() {
        let mut state = OpBuffer::default();
        state.store_op(make_op(11.into(), 0.into()));
        state.store_op(make_op(11.into(), 1.into()));
        state.store_op(make_op(11.into(), 2.into()));
        state.store_op(make_op(22.into(), 0.into()));
        state.store_op(make_op(22.into(), 1.into()));
        state.store_op(make_op(22.into(), 2.into()));
        assert_eq!(
            *state.state_vector(0.into()),
            BTreeMap::from([
                (11.into(), LogHeight::from(3)),
                (22.into(), LogHeight::from(3))
            ])
        );

        state.store_op(make_op(11.into(), 37.into()));
        assert_eq!(
            *state.state_vector(0.into()),
            BTreeMap::from([
                (11.into(), LogHeight::from(38)),
                (22.into(), LogHeight::from(3))
            ])
        );
    }

    #[test]
    fn test_diff() {
        let mut s1 = OpBuffer::default();
        let mut s2 = OpBuffer::default();

        s1.store_op(make_op(11.into(), 0.into()));
        s1.store_op(make_op(11.into(), 1.into()));
        s1.store_op(make_op(11.into(), 2.into()));
        s1.store_op(make_op(22.into(), 0.into()));
        s1.store_op(make_op(22.into(), 1.into()));
        s1.store_op(make_op(22.into(), 2.into()));
        s1.store_op(make_op(33.into(), 0.into()));

        s2.store_op(make_op(22.into(), 0.into()));
        s2.store_op(make_op(33.into(), 0.into()));
        s2.store_op(make_op(33.into(), 1.into()));
        s2.store_op(make_op(44.into(), 0.into()));

        let v1 = s1.state_vector(0.into());
        let v2 = s2.state_vector(0.into());

        assert_eq!(
            *v1,
            BTreeMap::from([
                (11.into(), LogHeight::from(3)),
                (22.into(), LogHeight::from(3)),
                (33.into(), LogHeight::from(1)),
            ])
        );

        assert_eq!(
            *v2,
            BTreeMap::from([
                (22.into(), LogHeight::from(1)),
                (33.into(), LogHeight::from(2)),
                (44.into(), LogHeight::from(1)),
            ])
        );

        assert_eq!(
            s1.diff(0.into(), &v2),
            BTreeSet::from([
                make_op(11.into(), 0.into()),
                make_op(11.into(), 1.into()),
                make_op(11.into(), 2.into()),
                make_op(22.into(), 1.into()),
                make_op(22.into(), 2.into()),
            ])
        );
        assert_eq!(
            s2.diff(0.into(), &v1),
            BTreeSet::from([make_op(33.into(), 1.into()), make_op(44.into(), 0.into()),])
        );
    }
}
