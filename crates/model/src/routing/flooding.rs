use std::collections::BTreeSet;

use simdash_action::routing::flooding::FloodingAction;
use simdash_ids::{OpId, Ticks};
use simdash_state::{dash::Operation, routing::flooding::FloodingOp};

pub struct FloodingModel {
    /// Don't broadcast an op twice within this interval.
    pub broadcast_interval: Ticks,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FloodingFx {
    pub broadcast_ops: BTreeSet<OpId>,
}

impl polestar::Machine for FloodingModel {
    type State = simdash_state::routing::flooding::FloodingState;
    type Action = simdash_action::routing::flooding::FloodingAction;
    type Fx = FloodingFx;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        let (state, broadcast_ops) = match action {
            FloodingAction::StoreOp { op } | FloodingAction::ReceiveOp { from: _, op } => {
                let existing = state.ops.insert(op.id, FloodingOp::new(&op));
                if existing.is_some() {
                    anyhow::bail!("Flooding op already exists: {:?}", op.id);
                }
                (state, BTreeSet::new())
            }
            FloodingAction::TriggerBroadcast => {
                let broadcasted = state
                    .ops
                    .values_mut()
                    .filter(|op| {
                        op.since_last_send
                            .map(|t| t <= self.broadcast_interval)
                            .unwrap_or(true)
                    })
                    .map(|op| {
                        op.touch();
                        op.id
                    })
                    .collect();
                (state, broadcasted)
            }
        };
        Ok((state, FloodingFx { broadcast_ops }))
    }
}

#[cfg(test)]
mod tests {
    use simdash_state::routing::flooding::FloodingState;

    use super::*;

    #[test]
    fn test_flooding_model() {
        let model = FloodingModel {
            broadcast_interval: 3.into(),
        };
        let state = FloodingState::default();
    }
}
