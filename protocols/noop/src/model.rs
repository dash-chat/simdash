use simdash_routing_common::{ProtocolNetworkAction, RoutingAction};

use crate::{ProtocolAction, ProtocolFx, action::NoopAction, state::NoopState};

/// The Noop model simply does nothing.
#[derive(Debug, Clone, Default)]
pub struct NoopModel;

impl polestar::Machine for NoopModel {
    type State = NoopState;
    type Action = ProtocolAction;
    type Fx = Vec<ProtocolFx>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        let state = match action {
            RoutingAction::Network(ProtocolNetworkAction::Tick) => state,
            RoutingAction::Network(ProtocolNetworkAction::Receive { data: _ }) => state,
            RoutingAction::Protocol(NoopAction::Noop) => state,
        };
        Ok((state, vec![]))
    }
}
