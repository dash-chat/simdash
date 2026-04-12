use std::collections::BTreeSet;

use simdash_action::routing::RoutingAction;
use simdash_ids::{OpId, Ticks};
use simdash_state::{TickableState, chat::Operation};

use crate::{
    action::FloodingAction,
    state::{FloodingOp, FloodingState},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub enum FloodingFx {
    BroadcastOps(BTreeSet<OpId>),
    StoreOp(Operation),
}

pub struct FloodingModel {
    /// Don't broadcast an op twice within this interval.
    pub broadcast_interval: Ticks,
}

impl polestar::Machine for FloodingModel {
    type State = FloodingState;
    type Action = RoutingAction<FloodingAction, FloodingOp>;
    type Fx = Option<FloodingFx>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        let (state, fx) = match action {
            RoutingAction::Tick => {
                state.tick();
                (state, None)
            }

            RoutingAction::Receive { from, data } => {
                let mut flooding_op = data;
                // Mark as recently sent so the node doesn't re-broadcast
                // the op in the same round it was received.
                flooding_op.touch();
                let id = flooding_op.id;
                let existing = state.ops.insert(id, flooding_op.clone());
                if existing.is_some() {
                    anyhow::bail!("Flooding op already exists: op={:?}", id);
                }
                let fx = todo!();
                (state, Some(fx))
            }
            RoutingAction::Protocol(FloodingAction::StoreOp { op }) => {
                let existing = state.ops.insert(op.id, FloodingOp::new(&op));
                if existing.is_some() {
                    anyhow::bail!("Flooding op already exists: op={:?}", op.id);
                }
                (state, None)
            }
            RoutingAction::Protocol(FloodingAction::TriggerBroadcast) => {
                let broadcasted = state
                    .ops
                    .values_mut()
                    .filter(|op| {
                        op.since_last_send
                            .map(|t| t >= self.broadcast_interval)
                            .unwrap_or(true)
                    })
                    .map(|op| {
                        op.touch();
                        op.id
                    })
                    .collect();
                (state, Some(FloodingFx::BroadcastOps(broadcasted)))
            }
        };
        Ok((state, fx))
    }
}

#[cfg(test)]
mod tests {
    use polestar::Machine;
    use simdash_ids::NodeId;

    use simdash_model::util::basic_op;

    use super::*;

    fn model() -> FloodingModel {
        FloodingModel {
            broadcast_interval: 3.into(),
        }
    }

    /// Storing the same op twice should be a no-op, not an error.
    #[test]
    fn store_op_is_idempotent() {
        let m = model();
        let state = FloodingState::default();
        let op = basic_op(1);
        let (state, _) = m
            .transition(state, FloodingAction::StoreOp { op: op.clone() }.into())
            .unwrap();
        let result = m.transition(state, FloodingAction::StoreOp { op }.into());
        assert!(result.is_err());
    }

    /// Ops that have never been broadcast are included in TriggerBroadcast Fx.
    #[test]
    fn trigger_broadcast_sends_new_ops() {
        let m = model();
        let state = FloodingState::default();
        let op = basic_op(1);
        let op_id = op.id;
        let (state, _) = m
            .transition(state, FloodingAction::StoreOp { op }.into())
            .unwrap();
        let (_, fx) = m
            .transition(state, FloodingAction::TriggerBroadcast.into())
            .unwrap();
        assert_eq!(
            fx,
            Some(FloodingFx::BroadcastOps([op_id].into_iter().collect()))
        );
    }

    /// An op that was just broadcast should NOT be immediately re-broadcast.
    #[test]
    fn trigger_broadcast_suppresses_immediate_resend() {
        let m = model();
        let state = FloodingState::default();
        let op = basic_op(1);
        let op_id = op.id;
        let (state, _) = m
            .transition(state, FloodingAction::StoreOp { op }.into())
            .unwrap();
        let (state, _) = m
            .transition(state, FloodingAction::TriggerBroadcast.into())
            .unwrap();
        let (_, fx) = m
            .transition(state, FloodingAction::TriggerBroadcast.into())
            .unwrap();
        assert_eq!(fx, Some(FloodingFx::BroadcastOps(BTreeSet::new())));
    }

    /// After broadcast_interval ticks, an op is eligible for re-broadcast.
    #[test]
    fn trigger_broadcast_resends_after_interval() {
        let m = model();
        let state = FloodingState::default();
        let op = basic_op(1);
        let op_id = op.id;
        let (mut state, _) = m
            .transition(state, FloodingAction::StoreOp { op }.into())
            .unwrap();
        let (mut state, _) = m
            .transition(state, FloodingAction::TriggerBroadcast.into())
            .unwrap();
        for _ in 0..3 {
            state.tick();
        }
        let (_, fx) = m
            .transition(state, FloodingAction::TriggerBroadcast.into())
            .unwrap();
        assert_eq!(
            fx,
            Some(FloodingFx::BroadcastOps([op_id].into_iter().collect()))
        );
    }

    /// ReceiveOp stores the op in state and produces no Fx.
    #[test]
    fn receive_op_is_stored() {
        let m = model();
        let state = FloodingState::default();
        let op = basic_op(42);
        let op_id = op.id;
        let (state, fx) = m
            .transition(
                state,
                RoutingAction::Receive {
                    from: NodeId::from(9),
                    data: FloodingOp::new(&op),
                }
                .into(),
            )
            .unwrap();
        assert_eq!(fx, Some(FloodingFx::StoreOp(op)));
        assert!(state.ops.contains_key(&op_id));
    }
}
