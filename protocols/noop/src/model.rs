use simdash_routing_common::{ProtocolNetworkAction, RoutingAction};

use crate::{ProtocolAction, ProtocolFx, action::NoopAction, state::NoopState};

/// The Noop model simply does nothing.
#[derive(Debug, Clone, Default)]
pub struct NoopModel;

impl polestar::Machine for NoopModel {
    type State = NoopState;
    type Action = ProtocolAction;
    type Fx = Option<ProtocolFx>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        let (state, fx) = match action {
            RoutingAction::Network(ProtocolNetworkAction::Tick) => (state, None),

            RoutingAction::Network(ProtocolNetworkAction::Receive { from: _, data: _ }) => {
                (state, None)
            }
            RoutingAction::Protocol(NoopAction::Noop) => (state, None),
        };
        Ok((state, fx))
    }
}
