mod routing_table;
pub use routing_table::PeerLogFact;

use std::collections::{BTreeSet, HashMap};

use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};
use simdash_types::*;

use crate::{Model, Op, SourceId, StateVector, state::routing_table::RoutingTable};

/// The Stig protocol's state simply tracks the number of ticks since the last time it was triggered.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// The buffer of operations to sync
    pub op_buffer: OpBuffer,
    /// Cached state_vector received from other nodes
    pub routes: RoutingTable,
}

impl State {
    pub fn subscribe(&mut self, model: &Model, topic_id: TopicId) {
        self.routes.record_local_topic_update(model, topic_id, None);
    }

    pub fn is_subscribed(&self, source_id: SourceId, topic_id: TopicId) -> bool {
        self.routes
            .facts
            .get(&topic_id)
            .and_then(|facts| facts.get(&source_id))
            .is_some()
    }

    pub fn store_op(&mut self, model: &Model, op: Op) {
        let op = StoredOp::new(op);
        self.routes.record_local_topic_update(
            model,
            op.metadata.topic_id,
            Some((op.metadata.log_id, LogHeight::from_seq(op.metadata.seq))),
        );
        println!(
            "### STORED: me={}, sv={:?}",
            model.mesh_id.renamed(),
            self.routes
                .facts
                .get(&op.metadata.topic_id)
                .and_then(|facts| facts.get(&model.mesh_id))
                .map(|fact| fact.state_vector.renamed_ref()),
            // self.op_buffer.state_vector(op.metadata.topic_id).renamed()
        );
        self.op_buffer.store_op(op);
    }

    pub fn from_ops(model: &Model, ops: BTreeSet<Op>) -> Self {
        let mut state = Self::default();
        for op in ops {
            state.store_op(model, op);
        }
        state
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Deref, From)]
pub struct StoredOp {
    #[deref]
    pub op: Op,
}

impl StoredOp {
    pub fn new(op: Op) -> Self {
        Self { op }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpBuffer {
    pub ops: HashMap<TopicId, HashMap<LogId, BTreeSet<StoredOp>>>,
}

impl OpBuffer {
    pub(super) fn store_op(&mut self, op: impl Into<StoredOp>) {
        let op = op.into();
        let log = self
            .ops
            .entry(op.metadata.topic_id)
            .or_default()
            .entry(op.metadata.log_id)
            .or_default();
        let metadata = op.metadata.clone();
        let existing = log.insert(op);
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
