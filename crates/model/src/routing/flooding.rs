use std::collections::BTreeSet;

use simdash_action::routing::flooding::FloodingAction;
use simdash_ids::{OpId, Ticks};
use simdash_state::{dash::Operation, routing::flooding::FloodingOp};

pub struct FloodingModel {
    /// Don't broadcast an op twice within this interval.
    pub broadcast_interval: Ticks,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct FloodingFx(BTreeSet<OpId>);

impl polestar::Machine for FloodingModel {
    type State = simdash_state::routing::flooding::FloodingState;
    type Action = simdash_action::routing::flooding::FloodingAction;
    type Fx = FloodingFx;
    type Error = ();

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        let (state, fx) = match action {
            FloodingAction::Tick => {
                state.ops.iter_mut().for_each(|op| op.tick());
                (state, BTreeSet::new())
            }
            FloodingAction::StoreOp { op } => {
                state.ops.push(FloodingOp::new(&op));
                (state, BTreeSet::new())
            }
            FloodingAction::ReceiveOp { from, op } => {
                state.ops.push(FloodingOp::new(&op));
                (state, BTreeSet::new())
            }
            FloodingAction::TriggerBroadcast => {
                let broadcasted = state
                    .ops
                    .iter_mut()
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
        Ok((state, FloodingFx(fx)))
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
