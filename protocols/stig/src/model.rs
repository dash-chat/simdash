use std::{collections::BTreeMap, sync::atomic::AtomicU32};

use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

use simdash_routing_common::{ProtocolNetworkAction, RoutingAction};
use simdash_types::Ticks;

use crate::{
    Announce, Payload, ProtocolAction, ProtocolFx, WireData, action::Action, state::State,
};

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
)]
pub struct SourceId(u32);

impl named_id::Nameable for SourceId {
    fn shortener(&self) -> Option<named_id::Shortener> {
        Some(named_id::Shortener {
            length: 3,
            prefix: "S",
        })
    }
}

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

impl SourceId {
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// The Stig model implements the flooding-sync protocol.
#[derive(Debug, Clone)]
pub struct Model {
    /// Ticks to wait between sync rounds
    pub sync_interval: Ticks,

    /// Identifier used only in this protocol
    pub mesh_id: SourceId,
}

impl Model {
    /// Constructor
    pub fn new(sync_interval: Ticks) -> Self {
        Self {
            sync_interval,
            mesh_id: SourceId::new(),
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
                    state.routes.record(a.clone());
                    a.hops += 1.into();
                    a.relay = self.mesh_id;

                    let mut fx = vec![];
                    // TODO: decide to also relay a payload if a new route was established

                    for (topic_id, state_vector) in &a.topics {
                        let ops = state.op_buffer.diff(*topic_id, &state_vector);
                        if !ops.is_empty() {
                            let addressees = state
                                .routes
                                .targets(self.mesh_id, ops.iter().map(|op| &op.metadata));
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
                            .targets(self.mesh_id, payload.ops.iter().map(|op| &op.metadata));
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
                let relay_announcements =
                    state.routes.announcements_to_broadcast(self.sync_interval);

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
    use polestar::Machine;
    use simdash_types::LogHeight;

    use crate::{StateVector, StoredOp, util::make_stig_op};

    use super::*;

    #[test]
    fn test_model() {
        let model = Model::new(3.into());
        let mut state = State::default();
        let topic = 0.into();
        let log = 0.into();
        let seq = 0.into();
        state.store_op(model.mesh_id, make_stig_op(topic, log, seq));

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
