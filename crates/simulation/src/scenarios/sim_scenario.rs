use std::collections::HashMap;
use std::sync::Arc;

use simdash_model_network::ChatTopology;
use simdash_model_network::NetworkState;
use simdash_model_network::{NetworkModel, RoundRobinNodeScheduler};
use simdash_model_node::{BasicNodeBehavior, NodeModel};
use simdash_routing_dispatch::flooding::{FloodingOp, FloodingState};
use simdash_routing_dispatch::{ecco, flooding_sync};
use simdash_routing_dispatch::{noop::NoopState, RoutingDispatchModel, RoutingDispatchState};
use polestar::StateMachine;
use polestar_sim::Scenario;
use simdash_types::{Behavior, NodeId, Ticks};

pub type SimBehaviorType<B> = RoundRobinNodeScheduler<BasicNodeBehavior<B>>;

/// Build a `NetworkModel` from the node IDs in `state`, using `broadcast_interval` and `topology`.
pub fn create_model(
    state: &NetworkState,
    broadcast_interval: Ticks,
    topology: ChatTopology,
) -> NetworkModel {
    let nodes: HashMap<NodeId, NodeModel> = state
        .graph
        .node_ids()
        .map(|id| {
            (
                id,
                NodeModel::new(RoutingDispatchModel::standard(id, broadcast_interval)),
            )
        })
        .collect();
    NetworkModel::new(nodes, topology)
}

/// Seed each node's routing state from its `chat.op_store`.
///
/// Call once after constructing model + initial state. Do NOT call again on reset —
/// `Simulation` stores the post-seeded state as `initial_state`.
pub fn seed_routing(model: &NetworkModel, state: &mut NetworkState) {
    let node_ids: Vec<NodeId> = state.graph.node_ids().collect();
    for node_id in node_ids {
        let node_model = model.nodes.get(&node_id).unwrap();
        state
            .graph
            .update_node(node_id, |mut n| {
                let flooding_ops = n
                    .chat
                    .op_store
                    .values()
                    .map(|op| (op.id, FloodingOp::from_op(op.clone())))
                    .collect();
                let flooding_sync_ops = n
                    .chat
                    .op_store
                    .values()
                    .map(|op| flooding_sync::Op::from_op(op))
                    .collect();
                let ecco_ops = n
                    .chat
                    .op_store
                    .values()
                    .map(|op| ecco::Op::from_op(op))
                    .collect();
                let groups = model
                    .topology
                    .chat_states
                    .get(&node_id)
                    .map(|s| s.groups.clone())
                    .unwrap_or_default();
                n.routing = RoutingDispatchState {
                    flooding: FloodingState { ops: flooding_ops },
                    flooding_sync: flooding_sync::State::from_ops(flooding_sync_ops),
                    noop: NoopState::default(),
                    ecco: ecco::State::initialize(&node_model.routing.ecco, ecco_ops, groups),
                };
                Ok((n, ()))
            })
            .ok();
    }
}

/// Create a scenario from a pre-built (and pre-seeded) model and state.
pub fn sim_scenario<In: Behavior<Model = RoutingDispatchModel>>(
    model: NetworkModel,
    initial_state: NetworkState,
    behavior: In,
) -> Scenario<RoundRobinNodeScheduler<BasicNodeBehavior<In>>> {
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(behavior));
    Scenario::initial(StateMachine::new(model, initial_state), behavior)
}
