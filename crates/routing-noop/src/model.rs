use simdash_routing_common::{ProtocolAction, ProtocolFx, ProtocolNetworkAction};

use crate::{NoopFx, WireData, action::NoopAction, state::NoopState};

/// The Noop model simply does nothing.
#[derive(Debug, Clone, Default)]
pub struct NoopModel;

impl polestar::Machine for NoopModel {
    type State = NoopState;
    type Action = ProtocolAction<NoopAction, WireData>;
    type Fx = Option<ProtocolFx<NoopFx, WireData>>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        let (state, fx) = match action {
            ProtocolAction::Network(ProtocolNetworkAction::Tick) => (state, None),

            ProtocolAction::Network(ProtocolNetworkAction::Receive { from: _, data: _ }) => {
                (state, None)
            }
            ProtocolAction::Protocol(NoopAction::Noop) => (state, None),
        };
        Ok((state, fx))
    }
}
