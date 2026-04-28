use std::collections::HashMap;

use dash_chat_top_gen::ChatTopology;
use simdash_model_network::NetworkState;
use simdash_model_network::{NetworkModel, RoundRobinNodeScheduler};
use simdash_model_node::{BasicNodeBehavior, NodeModel};
use simdash_routing_dispatch::flooding::{FloodingOp, FloodingState};
use simdash_routing_dispatch::{flooding_sync, stig};
use simdash_routing_dispatch::{noop::NoopState, RoutingDispatchModel, RoutingDispatchState};
use simdash_scenario::Scenario;
use simdash_types::{Behavior, NodeId};

pub type SimBehaviorType<B> = RoundRobinNodeScheduler<BasicNodeBehavior<B>>;

pub fn sim_scenario<In: Behavior<Model = RoutingDispatchModel>>(
    mut initial_state: NetworkState,
    broadcast_interval: u32,
    topology: ChatTopology,
    behavior: In,
) -> Scenario<RoundRobinNodeScheduler<BasicNodeBehavior<In>>> {
    // Seed each node's routing state from its chat op_store so flooding
    // knows about the initial ops and can broadcast them.
    let node_ids: Vec<_> = initial_state.graph.node_ids().collect();

    let node_models: HashMap<NodeId, NodeModel> = node_ids
        .iter()
        .map(|id| {
            (
                *id,
                NodeModel::new(RoutingDispatchModel::standard(
                    *id,
                    broadcast_interval.into(),
                )),
            )
        })
        .collect();

    for node_id in node_ids.iter() {
        let model = node_models.get(node_id).unwrap();
        initial_state
            .graph
            .update_node(*node_id, |mut n| {
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
                let stig_ops = n
                    .chat
                    .op_store
                    .values()
                    .map(|op| stig::Op::from_op(op))
                    .collect();
                n.routing = RoutingDispatchState {
                    flooding: FloodingState { ops: flooding_ops },
                    flooding_sync: flooding_sync::State::from_ops(flooding_sync_ops),
                    noop: NoopState::default(),
                    stig: stig::State::from_ops(&model.routing.stig, stig_ops),
                };
                Ok((n, ()))
            })
            .ok();
    }

    let model = NetworkModel::new(node_models, topology);
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(behavior));
    Scenario::initial(model, initial_state, behavior)
}
