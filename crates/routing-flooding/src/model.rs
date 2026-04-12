use std::collections::BTreeSet;

use simdash_ids::Ticks;
use simdash_routing_common::{
    ProtocolAction, ProtocolFx, ProtocolNetworkAction, SendFx, StorageFx,
};

use crate::{
    FloodingWireData,
    action::FloodingAction,
    state::{FloodingOp, FloodingState},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub enum FloodingFx {}

pub struct FloodingModel {
    /// Don't broadcast an op twice within this interval.
    pub broadcast_interval: Ticks,
}

impl polestar::Machine for FloodingModel {
    type State = FloodingState;
    type Action = ProtocolAction<FloodingAction, FloodingWireData>;
    type Fx = Option<ProtocolFx<FloodingFx, FloodingWireData>>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        let (state, fx) = match action {
            ProtocolAction::Network(ProtocolNetworkAction::Tick) => {
                state.tick();
                (state, None)
            }

            ProtocolAction::Network(ProtocolNetworkAction::Receive { from: _, data }) => {
                let mut flooding_op = FloodingOp::from_wire(data);
                // Mark as recently sent so the node doesn't re-broadcast
                // the op in the same round it was received.
                flooding_op.touch();
                let id = flooding_op.id();
                let existing = state.ops.insert(id, flooding_op.clone());
                if existing.is_some() {
                    anyhow::bail!("Flooding op already exists: op={:?}", id);
                }
                let fx = ProtocolFx::Node(StorageFx::StoreOps {
                    ops: vec![flooding_op.data.encrypted],
                });
                (state, Some(fx))
            }
            ProtocolAction::Protocol(FloodingAction::StoreOp(data)) => {
                let op = FloodingOp::from_wire(data);
                let existing = state.ops.insert(op.id(), op.clone());
                if existing.is_some() {
                    anyhow::bail!("Flooding op already exists: op={:?}", op.id());
                }
                (state, None)
            }
            ProtocolAction::Protocol(FloodingAction::TriggerBroadcast) => {
                let broadcasted = state
                    .ops
                    .values_mut()
                    .filter(|op| {
                        op.state
                            .since_last_send
                            .map(|t| t >= self.broadcast_interval)
                            .unwrap_or(true)
                    })
                    .map(|op| {
                        op.touch();
                        op.data.clone()
                    })
                    .collect::<BTreeSet<_>>();
                (
                    state,
                    (!broadcasted.is_empty())
                        .then(|| ProtocolFx::Network(SendFx::BroadcastData { data: broadcasted })),
                )
            }
        };
        Ok((state, fx))
    }
}

#[cfg(test)]
mod tests {
    use maplit::btreeset;
    use polestar::Machine;
    use pretty_assertions::assert_eq;
    use simdash_ids::{NodeId, util::basic_op};
    use simdash_routing_common::ProtocolNetworkAction;

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
        let data = FloodingWireData::new(op.clone());
        let (state, _) = m
            .transition(state, FloodingAction::StoreOp(data.clone()).into())
            .unwrap();
        let result = m.transition(state, FloodingAction::StoreOp(data).into());
        assert!(result.is_err());
    }

    /// Ops that have never been broadcast are included in TriggerBroadcast Fx.
    #[test]
    fn trigger_broadcast_sends_new_ops() {
        let m = model();
        let state = FloodingState::default();
        let op = basic_op(1);
        let data = FloodingWireData::new(op);
        let (state, _) = m
            .transition(state, FloodingAction::StoreOp(data.clone()).into())
            .unwrap();
        let (_, fx) = m
            .transition(state, FloodingAction::TriggerBroadcast.into())
            .unwrap();
        assert_eq!(
            fx,
            Some(ProtocolFx::Network(SendFx::BroadcastData {
                data: btreeset![data],
            })),
        );
    }

    /// An op that was just broadcast should NOT be immediately re-broadcast.
    #[test]
    fn trigger_broadcast_suppresses_immediate_resend() {
        let m = model();
        let state = FloodingState::default();
        let op = basic_op(1);
        let (state, _) = m
            .transition(
                state,
                FloodingAction::StoreOp(FloodingWireData::new(op)).into(),
            )
            .unwrap();
        let (state, _) = m
            .transition(state, FloodingAction::TriggerBroadcast.into())
            .unwrap();
        let (_, fx) = m
            .transition(state, FloodingAction::TriggerBroadcast.into())
            .unwrap();
        assert_eq!(fx, None);
    }

    /// After broadcast_interval ticks, an op is eligible for re-broadcast.
    #[test]
    fn trigger_broadcast_resends_after_interval() {
        let m = model();
        let state = FloodingState::default();
        let op = basic_op(1);
        let data = FloodingWireData::new(op);
        let (state, _) = m
            .transitions(
                state,
                [
                    FloodingAction::StoreOp(data.clone()).into(),
                    FloodingAction::TriggerBroadcast.into(),
                    ProtocolAction::Network(ProtocolNetworkAction::Tick),
                    ProtocolAction::Network(ProtocolNetworkAction::Tick),
                    ProtocolAction::Network(ProtocolNetworkAction::Tick),
                ],
            )
            .unwrap();
        let (_, fx) = m
            .transition(state, FloodingAction::TriggerBroadcast.into())
            .unwrap();
        assert_eq!(
            fx,
            Some(ProtocolFx::Network(SendFx::BroadcastData {
                data: btreeset![data],
            })),
        );
    }

    /// ReceiveOp stores the op in state and produces no Fx.
    #[test]
    fn receive_op_is_stored() {
        let m = model();
        let state = FloodingState::default();
        let op = basic_op(42);
        let op_id = op.id;
        let data = FloodingWireData::new(op.clone());
        let (state, fx) = m
            .transition(
                state,
                ProtocolAction::Network(ProtocolNetworkAction::Receive {
                    from: NodeId::from(9),
                    data: data.clone(),
                }),
            )
            .unwrap();
        assert_eq!(
            fx,
            Some(ProtocolFx::Node(StorageFx::StoreOps {
                ops: vec![data.encrypted]
            }))
        );
        assert!(state.ops.contains_key(&op_id));
    }
}
