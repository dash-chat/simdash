mod macros;

use serde::{Deserialize, Serialize};
// pub use macros::*;
use simdash_action::{
    chat::ChatAction,
    network::NetworkAction,
    node::NodeAction,
    routing::{NetworkRoutingFx, NodeRoutingFx, RoutingAction, RoutingDataExt, RoutingFx},
};
use simdash_ids::NodeId;
use simdash_routing_flooding::*;
use simdash_state::network::NetworkState;

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

impl RoutingDataExt for RoutingModel {
    type Data = RoutingDataDispatch;

    fn fx_to_network_actions(
        fx: Self::Fx,
        from: NodeId,
        network_state: &NetworkState<RoutingState>,
    ) -> Vec<NetworkAction<Self::Action>> {
        match fx {
            RoutingFx::Nothing => vec![],
            RoutingFx::Network(fx) => match fx {
                NetworkRoutingFx::BroadcastData { data } => {
                    let neighbors = network_state.neighborhood(from).collect::<Vec<_>>();
                    neighbors
                        .into_iter()
                        .map(|neighbor| {
                            NetworkAction::Node(
                                neighbor,
                                NodeAction::Routing(RoutingAction::Receive {
                                    from,
                                    data: data.clone(),
                                }),
                            )
                        })
                        .collect()
                }
                NetworkRoutingFx::UnicastData { to, data } => {
                    vec![NetworkAction::Node(
                        to,
                        NodeAction::Routing(RoutingAction::Receive { from, data }),
                    )]
                }
            },
            RoutingFx::Node(fx) => match fx {
                NodeRoutingFx::StoreOps { ops } => ops
                    .into_iter()
                    .map(|op| NetworkAction::Node(from, NodeAction::Chat(ChatAction::StoreOp(op))))
                    .collect(),
            },
            RoutingFx::Protocol(_) => vec![],
        }
    }

    fn produce_tick() -> Self::Action {
        RoutingAction::Tick
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct RoutingState {
    pub flooding: FloodingState,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingDataDispatch {
    Flooding(FloodingOp),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingActionDispatch {
    Flooding(FloodingAction),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoutingFxDispatch {
    Flooding(Option<FloodingFx>),
}

impl RoutingState {
    pub fn tick(&mut self) {
        <FloodingState as simdash_state::TickableState>::tick(&mut self.flooding)
    }
}

impl polestar::Machine for RoutingModel {
    type State = RoutingState;
    type Action =
        simdash_action::routing::RoutingAction<RoutingActionDispatch, RoutingDataDispatch>;
    type Fx = simdash_action::routing::RoutingFx<RoutingFxDispatch, RoutingDataDispatch>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            RoutingAction::Tick => {
                state.tick();
                Ok((state, RoutingFx::Nothing))
            }

            RoutingAction::Receive { from, data } => match data {
                RoutingDataDispatch::Flooding(op) => {
                    let (new, fx) = self
                        .flooding
                        .transition(state.flooding, RoutingAction::Receive { from, data: op })?;
                    state.flooding = new;
                    todo!()
                    // Ok((state, RoutingFx::Flooding(fx)))
                }
            },

            RoutingAction::Protocol(action) => match action {
                RoutingActionDispatch::Flooding(a) => {
                    let (new, fx) = self
                        .flooding
                        .transition(state.flooding, RoutingAction::Protocol(a))?;
                    state.flooding = new;
                    todo!()
                    // Ok((state, RoutingFx::Flooding(fx)))
                }
            },
        }
    }
}
