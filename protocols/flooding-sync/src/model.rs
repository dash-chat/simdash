use maplit::hashset;
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
    type Fx = Option<ProtocolFx>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        dbg!(&action);
        let (state, fx) = match action {
            RoutingAction::Network(ProtocolNetworkAction::Receive { data }) => match data {
                WireData::Announce { state_vector } => {
                    let ops = state.op_buffer.diff(&state_vector);
                    if ops.is_empty() {
                        (state, None)
                    } else {
                        (state, Some(ProtocolFx::broadcast(WireData::Ops { ops })))
                    }
                }
                WireData::Ops { ops } => {
                    for op in ops {
                        state.op_buffer.store_op(op);
                    }
                    (state, None)
                }
            },
            RoutingAction::Protocol(Action::TriggerSync) => {
                state.countdown = self.sync_interval;
                let state_vector = state.op_buffer.state_vector();
                let fx = ProtocolFx::broadcast(WireData::Announce { state_vector });
                (state, Some(fx))
            }
            RoutingAction::Network(ProtocolNetworkAction::Tick) => {
                if state.countdown > 0.into() {
                    state.countdown = state.countdown - 1.into();
                }
                (state, None)
            }
        };
        Ok((state, fx))
    }
}
