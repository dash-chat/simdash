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
}

#[cfg(test)]
mod tests {
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
            BTreeSet::from([
                (11.into(), LogHeight::from(3)),
                (22.into(), LogHeight::from(3))
            ])
        );

        state.store_op(make_op(11.into(), 37.into()));
        assert_eq!(
            state.state_vector(),
            BTreeSet::from([
                (11.into(), LogHeight::from(38)),
                (22.into(), LogHeight::from(3))
            ])
        );
    }
}
