use simdash_routing_common::{ProtocolNetworkAction, RoutingAction};
use simdash_types::Ticks;

use crate::{ProtocolAction, ProtocolFx, WireData, action::Action, state::State};

/// The FloodingSync model implements the flooding-sync protocol.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Ticks to wait between sync rounds
    pub sync_interval: Ticks,
}

impl Model {
    /// Constructor
    pub fn new(sync_interval: Ticks) -> Self {
        Self { sync_interval }
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
                WireData::Announce(state_vector) => {
                    let ops = state.op_buffer.diff(&state_vector);
                    // println!("* STATEVEC: {:?}", state.op_buffer.state_vector());
                    if ops.is_empty() {
                        (state, vec![])
                    } else {
                        (state, vec![ProtocolFx::broadcast(WireData::Ops { ops })])
                    }
                }
                WireData::Ops { ops } => {
                    let fx = ops
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
                if state.countdown > 0.into() {
                    return Ok((state, vec![]));
                }
                state.countdown = self.sync_interval;
                let state_vector = state.op_buffer.state_vector();
                let fx = ProtocolFx::broadcast(WireData::Announce(state_vector));
                (state, vec![fx])
            }
            RoutingAction::Network(ProtocolNetworkAction::Tick) => {
                if state.countdown > 0.into() {
                    state.countdown = state.countdown - 1.into();
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
