use std::collections::HashMap;

use simdash_action::routing::RoutingAction;

pub mod flooding;

// crate::routing_protocols! {
//     Flooding / flooding => {
//         model:  flooding::FloodingModel,
//         state:  flooding::FloodingState,
//         action: flooding::FloodingAction,
//         fx:     flooding::FloodingFx,
//         data:   flooding::FloodingData,
//     },
// }

pub enum RoutingProtocols {
    Flooding,
}

pub struct RoutingModel {
    pub flooding: flooding::FloodingModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingFx {
    Flooding(Option<flooding::FloodingFx>),
}

impl polestar::Machine for RoutingModel {
    type State = simdash_state::routing::RoutingState;
    type Action = simdash_action::routing::RoutingAction;
    type Fx = RoutingFx;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            RoutingAction::Flooding(action) => {
                let (flooding, fx) = self.flooding.transition(state.flooding, action)?;
                state.flooding = flooding;
                Ok((state, RoutingFx::Flooding(fx)))
            }
        }
    }
}
