use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use simdash_types::*;

use crate::{Op, StateVector};

/// The Noop protocol's state simply tracks the number of ticks since the last time it was triggered.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// The number of ticks until the next sync round may begin
    pub countdown: Ticks,
    /// The buffer of operations to sync
    pub op_buffer: OpBuffer,
}

impl Tickable for State {
    fn tick(&mut self) {
        self.countdown = self.countdown + 1.into();
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpBuffer {
    ops: HashMap<LogId, BTreeSet<Op>>,
}

impl OpBuffer {
    pub fn store_op(&mut self, op: Op) {
        let log = self.ops.entry(op.metadata.log_id).or_default();
        let metadata = op.metadata.clone();
        let existing = log.insert(op);
        if existing {
            tracing::warn!("Op already exists: metadata={:?}", metadata);
        }
    }

    pub fn state_vector(&self) -> StateVector {
        self.ops
            .iter()
            .map(|(log_id, ops)| {
                (
                    log_id.clone(),
                    ops.iter()
                        .map(|op| op.metadata.seq)
                        .max()
                        .map(|seq| LogHeight::from(*seq + 1u32))
                        .unwrap_or_default(),
                )
            })
            .collect()
    }

    pub fn diff(&self, other: &StateVector) -> BTreeSet<Op> {
        self.ops
            .iter()
            .flat_map(|(log_id, ops)| {
                let height = other.get(log_id).map(|h| **h).unwrap_or(0);
                ops.iter()
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
            metadata: Metadata { log_id, seq },
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
            state.state_vector(),
            BTreeMap::from([
                (11.into(), LogHeight::from(3)),
                (22.into(), LogHeight::from(3))
            ])
        );

        state.store_op(make_op(11.into(), 37.into()));
        assert_eq!(
            state.state_vector(),
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

        let v1 = s1.state_vector();
        let v2 = s2.state_vector();

        assert_eq!(
            v1,
            BTreeMap::from([
                (11.into(), LogHeight::from(3)),
                (22.into(), LogHeight::from(3)),
                (33.into(), LogHeight::from(1)),
            ])
        );

        assert_eq!(
            v2,
            BTreeMap::from([
                (22.into(), LogHeight::from(1)),
                (33.into(), LogHeight::from(2)),
                (44.into(), LogHeight::from(1)),
            ])
        );

        assert_eq!(
            s1.diff(&v2),
            BTreeSet::from([
                make_op(11.into(), 0.into()),
                make_op(11.into(), 1.into()),
                make_op(11.into(), 2.into()),
                make_op(22.into(), 1.into()),
                make_op(22.into(), 2.into()),
            ])
        );
        assert_eq!(
            s2.diff(&v1),
            BTreeSet::from([make_op(33.into(), 1.into()), make_op(44.into(), 0.into()),])
        );
    }
}
