use derive_more::{Deref, Display, From};
use named_id::RenameAll;
use serde::{Deserialize, Serialize};

use simdash_routing_common::{ProtocolNetworkAction, RoutingAction};
use simdash_types::{NodeId, Ticks};

use crate::{Payload, ProtocolAction, ProtocolFx, WireData, action::Action, state::State};

/// An ephemeral identifier for a node in the mesh.
///
/// For now, it's just the same as NodeId for ease of debugging.
/// When we want to model SourceId changing, then this can become decoupled from NodeId.
#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    From,
    Deref,
    RenameAll,
)]
pub struct SourceId(pub NodeId);

impl From<u32> for SourceId {
    fn from(id: u32) -> Self {
        Self(NodeId::from(id))
    }
}

/// The Stig model implements the flooding-sync protocol.
#[derive(Debug)]
pub struct Model {
    /// Ticks to wait between sync rounds
    pub sync_interval: Ticks,

    /// Identifier used only in this protocol
    pub mesh_id: SourceId,
}

impl Model {
    /// Constructor
    pub fn new(mesh_id: SourceId, sync_interval: Ticks) -> Self {
        Self {
            sync_interval,
            mesh_id,
        }
    }
}

impl polestar::Machine for Model {
    type State = State;
    type Action = ProtocolAction;
    type Fx = Vec<ProtocolFx>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        // println!("* ACTION:   {:?}", action);
        let (state, fx) = match action {
            RoutingAction::Network(ProtocolNetworkAction::Receive { data }) => match data {
                WireData::Announce(mut a) => {
                    state.routes.record_peer_announcement(a.clone());
                    a.hops += 1.into();
                    a.relay = self.mesh_id;

                    let mut fx = vec![];
                    // TODO: decide to also relay a payload if a new route was established

                    for (topic_id, state_vector) in &a.topics {
                        let ops = state.op_buffer.diff(*topic_id, &state_vector);
                        if !ops.is_empty() {
                            let addressees = state
                                .routes
                                .targets(self, ops.iter().map(|op| &op.metadata));
                            if !addressees.is_empty() {
                                fx.push(ProtocolFx::broadcast(WireData::Payload(Payload {
                                    addressees,
                                    ops,
                                })));
                            }
                        }
                    }

                    // fx.push(ProtocolFx::broadcast(WireData::Announce(a)));

                    (state, fx)
                }
                WireData::Payload(payload) => {
                    let mut fx = vec![];
                    if payload.addressees.contains(&self.mesh_id) {
                        let addressees = state
                            .routes
                            .targets(self, payload.ops.iter().map(|op| &op.metadata));
                        if !addressees.is_empty() {
                            fx.push(ProtocolFx::broadcast(WireData::Payload(Payload {
                                addressees,
                                ops: payload.ops.clone(),
                            })));
                        }
                    }

                    fx.extend(payload.ops.into_iter().map(|op| {
                        let encrypted = op.encrypted.clone();
                        state.op_buffer.store_op(op);
                        ProtocolFx::store_op(encrypted)
                    }));
                    (state, fx)
                }
            },
            RoutingAction::Protocol(Action::TriggerSync) => {
                // let topics = state.op_buffer.topics().collect::<BTreeMap<_, _>>();
                let relay_announcements = state.routes.announcements_to_broadcast(self);

                let fx = relay_announcements
                    .map(|a| ProtocolFx::broadcast(WireData::Announce(a)))
                    .collect();
                (state, fx)
            }
            RoutingAction::Network(ProtocolNetworkAction::Tick) => {
                state.routes.tick();
                (state, vec![])
            }
        };
        // println!("* STATE:    {:?}", state);
        // println!("* FX:       {:?}", fx);
        // println!();
        Ok((state, fx))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use polestar::Machine;
    use simdash_types::LogHeight;

    use crate::{Announce, StateVector, util::make_stig_op};

    use super::*;

    #[test]
    fn test_subscribe() {
        let model = Model::new(NodeId(0).into(), 3.into());
        let mut state = State::default();
        let topic = 0.into();
        state.subscribe(&model, topic);
        assert_eq!(
            state
                .routes
                .facts
                .get(&topic)
                .unwrap()
                .get(&model.mesh_id)
                .unwrap()
                .state_vector,
            StateVector::new(None)
        );

        let trigger = ProtocolAction::protocol(Action::TriggerSync);
        let broadcast = ProtocolFx::broadcast(WireData::Announce(Announce::new(
            model.mesh_id,
            BTreeMap::from([(topic, StateVector::new([]))]),
        )));

        // let (state, _) = model.transition(state, ProtocolAction::tick()).unwrap();
        let (_, fx) = model.transition(state, trigger.clone()).unwrap();
        assert_eq!(fx, vec![broadcast]);
    }

    #[test]
    fn test_sync_interval() {
        let model = Model::new(NodeId(0).into(), 3.into());
        let mut state = State::default();
        let topic = 0.into();
        let log = 0.into();
        let seq = 0.into();
        state.store_op(&model, make_stig_op(topic, log, seq));

        let trigger = ProtocolAction::protocol(Action::TriggerSync);
        let broadcast = ProtocolFx::broadcast(WireData::Announce(Announce::new(
            model.mesh_id,
            BTreeMap::from([(topic, StateVector::new([(log, LogHeight::from_seq(seq))]))]),
        )));

        let since_last = |state: &State| -> Option<Ticks> {
            state
                .routes
                .facts
                .get(&topic)
                .unwrap()
                .get(&model.mesh_id)
                .unwrap()
                .since_last_broadcast
        };
        assert_eq!(since_last(&state), None);

        let (state, _) = model.transition(state, ProtocolAction::tick()).unwrap();
        assert_eq!(since_last(&state), None);
        let (state, fx) = model.transition(state, trigger.clone()).unwrap();
        assert_eq!(fx, vec![broadcast.clone()]);
        assert_eq!(since_last(&state), Some(0.into()));

        let (state, _) = model.transition(state, ProtocolAction::tick()).unwrap();
        assert_eq!(since_last(&state), Some(1.into()));
        let (state, fx) = model.transition(state, trigger.clone()).unwrap();
        assert_eq!(fx, vec![]);
        assert_eq!(since_last(&state), Some(1.into()));

        let (state, _) = model.transition(state, ProtocolAction::tick()).unwrap();
        assert_eq!(since_last(&state), Some(2.into()));
        let (state, fx) = model.transition(state, trigger.clone()).unwrap();
        assert_eq!(fx, vec![]);
        assert_eq!(since_last(&state), Some(2.into()));

        let (state, _) = model.transition(state, ProtocolAction::tick()).unwrap();
        assert_eq!(since_last(&state), Some(3.into()));
        let (state, fx) = model.transition(state, trigger.clone()).unwrap();
        assert_eq!(fx, vec![broadcast]);
        assert_eq!(since_last(&state), Some(0.into()));

        let (state, _) = model.transition(state, ProtocolAction::tick()).unwrap();
        assert_eq!(since_last(&state), Some(1.into()));
        let (_, fx) = model.transition(state, trigger.clone()).unwrap();
        assert_eq!(fx, vec![]);
    }
}
