use dash_chat_top_gen::ChatTopology;
use simdash_model_network::{NetworkModel, RoundRobinNodeScheduler};
use simdash_model_node::{BasicNodeBehavior, NodeModel};
use simdash_routing_dispatch::flooding::{FloodingOp, FloodingState};
use simdash_routing_dispatch::flooding_sync;
use simdash_routing_dispatch::{noop::NoopState, RoutingDispatchModel, RoutingDispatchState};
use simdash_model_network::NetworkState;
use simdash_scenario::Scenario;
use simdash_types::Behavior;

pub type FloodingBehaviorType<B> = RoundRobinNodeScheduler<BasicNodeBehavior<B>>;

pub fn flooding_scenario<In: Behavior<RoutingDispatchModel>>(
    mut initial_state: NetworkState,
    broadcast_interval: u32,
    topology: ChatTopology,
    behavior: In,
) -> Scenario<NetworkModel, RoundRobinNodeScheduler<BasicNodeBehavior<In>>> {
    // Seed each node's routing state from its chat op_store so flooding
    // knows about the initial ops and can broadcast them.
    let node_ids: Vec<_> = initial_state.graph.node_ids().collect();
    for node_id in node_ids {
        initial_state
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
                n.routing = RoutingDispatchState {
                    flooding: FloodingState { ops: flooding_ops },
                    flooding_sync: flooding_sync::State::from_ops(flooding_sync_ops),
                    noop: NoopState::default(),
                };
                Ok((n, ()))
            })
            .ok();
    }

    let model = NetworkModel::new(
        NodeModel::new(RoutingDispatchModel::standard(broadcast_interval.into())),
        topology,
    );
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(behavior));
    Scenario::initial(model, initial_state, behavior)
}
