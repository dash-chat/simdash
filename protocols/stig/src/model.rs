use std::sync::atomic::AtomicU32;

use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};

use simdash_routing_common::{ProtocolNetworkAction, RoutingAction};
use simdash_types::Ticks;

use crate::{Announce, ProtocolAction, ProtocolFx, WireData, action::Action, state::State};

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
                WireData::Announce(a) => {
                    state.routes.record(a);
                    todo!();
                    // let ops = state.op_buffer.diff(a.topic_id, &a.state_vector);
                    // if ops.is_empty() {
                    //     (state, vec![])
                    // } else {
                    //     (state, vec![ProtocolFx::broadcast(WireData::Ops { ops })])
                    // }
                }
                WireData::Payload(payload) => {
                    let fx = payload
                        .ops
                        .into_iter()
                        .map(|op| {
                            let encrypted = op.encrypted.clone();
                            state.op_buffer.store_op(op);
                            ProtocolFx::store_op(encrypted)
                        })
                        .collect();
                    (state, fx)
                }
            },
            RoutingAction::Protocol(Action::TriggerSync) => {
                if state.sync_countdown > 0.into() {
                    return Ok((state, vec![]));
                }
                state.sync_countdown = self.sync_interval;
                let announcements = state.op_buffer.announcements();
                let fx = announcements
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
