mod macros;

use serde::{Deserialize, Serialize};
// pub use macros::*;
use simdash_routing_common::{ProtocolAction, ProtocolFx, ProtocolNetworkAction};
use simdash_routing_flooding::*;

// routing_protocols! {
//     Flooding / flooding => {
//         model:  FloodingModel,
//         state:  FloodingState,
//         action: FloodingAction,
//         fx:     FloodingFx,
//         data:   FloodingOp,
//     },
// }

pub struct RoutingModel {
    pub flooding: FloodingModel,
}

// impl RoutingDataExt for RoutingModel {
//     type Data = RoutingDataDispatch;

//     fn fx_to_network_actions(
//         fx: Self::Fx,
//         from: NodeId,
//         network_state: &NetworkState<RoutingState>,
//     ) -> Vec<NetworkAction<Self::Action>> {
//         match fx {
//             RoutingFx::Nothing => vec![],
//             RoutingFx::Network(fx) => match fx {
//                 NetworkRoutingFx::BroadcastData { data } => {
//                     let neighbors = network_state.neighborhood(from).collect::<Vec<_>>();
//                     neighbors
//                         .into_iter()
//                         .map(|neighbor| {
//                             NetworkAction::Node(
//                                 neighbor,
//                                 NodeAction::Routing(ProtocolAction::Receive {
//                                     from,
//                                     data: data.clone(),
//                                 }),
//                             )
//                         })
//                         .collect()
//                 }
//                 NetworkRoutingFx::UnicastData { to, data } => {
//                     vec![NetworkAction::Node(
//                         to,
//                         NodeAction::Routing(ProtocolAction::Receive { from, data }),
//                     )]
//                 }
//             },
//             RoutingFx::Node(fx) => match fx {
//                 NodeRoutingFx::StoreOps { ops } => ops
//                     .into_iter()
//                     .map(|op| NetworkAction::Node(from, NodeAction::Chat(ChatAction::StoreOp(op))))
//                     .collect(),
//             },
//             RoutingFx::Protocol(_) => vec![],
//         }
//     }

//     fn produce_tick() -> Self::Action {
//         ProtocolAction::Tick
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct RoutingState {
    pub flooding: FloodingState,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingDataDispatch {
    Flooding(FloodingWireData),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingActionDispatch {
    Flooding(FloodingAction),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingFxDispatch {
    Flooding(Option<ProtocolFx<FloodingFx, FloodingWireData>>),
}

impl RoutingState {
    pub fn tick(&mut self) {
        self.flooding.tick()
        // TODO: maybe add TickableState back in?
        // <FloodingState as simdash_state::TickableState>::tick(&mut self.flooding)
    }
}

impl polestar::Machine for RoutingModel {
    type State = RoutingState;
    type Action = ProtocolAction<RoutingActionDispatch, RoutingDataDispatch>;
    type Fx = Option<ProtocolFx<RoutingFxDispatch, RoutingDataDispatch>>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            ProtocolAction::Network(ProtocolNetworkAction::Tick) => {
                state.tick();
                Ok((state, None))
            }

            ProtocolAction::Network(ProtocolNetworkAction::Receive { from, data }) => match data {
                RoutingDataDispatch::Flooding(data) => {
                    let (new, fx) = self.flooding.transition(
                        state.flooding,
                        ProtocolAction::Network(ProtocolNetworkAction::Receive { from, data }),
                    )?;
                    state.flooding = new;
                    Ok((
                        state,
                        Some(ProtocolFx::Protocol(RoutingFxDispatch::Flooding(fx))),
                    ))
                }
            },

            ProtocolAction::Protocol(action) => match action {
                RoutingActionDispatch::Flooding(a) => {
                    let (new, fx) = self
                        .flooding
                        .transition(state.flooding, ProtocolAction::Protocol(a))?;
                    state.flooding = new;
                    Ok((
                        state,
                        Some(ProtocolFx::Protocol(RoutingFxDispatch::Flooding(fx))),
                    ))
                }
            },
        }
    }
}
