use std::sync::atomic::AtomicU32;

use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

use simdash_routing_common::{ProtocolNetworkAction, RoutingAction};
use simdash_types::Ticks;

use crate::{
    Announce, Metadata, Payload, ProtocolAction, ProtocolFx, WireData, action::Action, state::State,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, From, Deref,
)]
pub struct SourceId(u32);

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
                            let addressees =
                                state.routes.targets(ops.iter().map(|op| &op.metadata));
                            if !addressees.is_empty() {
                                fx.push(ProtocolFx::broadcast(WireData::Payload(Payload {
                                    addressees,
                                    ops,
                                })));
                            }
                        }
                    }

                    fx.push(ProtocolFx::broadcast(WireData::Announce(a)));

                    (state, fx)
                }
                WireData::Payload(payload) => {
                    let mut fx = vec![];
                    if payload.addressees.contains(&self.mesh_id) {
                        let addressees = state
                            .routes
                            .targets(payload.ops.iter().map(|op| &op.metadata));
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
                if state.sync_countdown > 0.into() {
                    return Ok((state, vec![]));
                }
                state.sync_countdown = self.sync_interval;
                let topics = state.op_buffer.topics();
                let own_announcement = Announce::new(self.mesh_id, topics.collect());
                let relay_announcements = state.routes.announcements();

                let fx = relay_announcements
                    .chain([own_announcement])
                    .map(|a| ProtocolFx::broadcast(WireData::Announce(a)))
                    .collect();
                (state, fx)
            }
            RoutingAction::Network(ProtocolNetworkAction::Tick) => {
                if state.sync_countdown > 0.into() {
                    state.sync_countdown = state.sync_countdown - 1.into();
                }
                (state, vec![])
            }
        };
        // println!("* STATE:    {:?}", state);
        // println!("* FX:       {:?}", fx);
        // println!();
        Ok((state, fx))
    }
}
