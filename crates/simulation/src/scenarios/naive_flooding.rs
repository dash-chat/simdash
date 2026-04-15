use dash_chat_top_gen::ChatTopology;
use simdash_behavior::network::RoundRobinNodeScheduler;
use simdash_behavior::node::BasicNodeBehavior;
use simdash_behavior::routing::BasicRoutingBehavior;
use simdash_model::network::NetworkModel;
use simdash_model::node::NodeModel;
use simdash_routing_dispatch::flooding::{FloodingOp, FloodingState};
use simdash_routing_dispatch::flooding_sync;
use simdash_routing_dispatch::{noop::NoopState, RoutingDispatchModel, RoutingDispatchState};
use simdash_scenario::Scenario;
use simdash_state::network::NetworkState;

pub type NaiveFloodingBehaviorType =
    RoundRobinNodeScheduler<BasicNodeBehavior<BasicRoutingBehavior>>;

pub fn naive_flooding(
    mut initial_state: NetworkState,
    broadcast_interval: u32,
    topology: ChatTopology,
) -> Scenario<NetworkModel, NaiveFloodingBehaviorType> {
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
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(BasicRoutingBehavior));
    Scenario::initial(model, initial_state, behavior)
}
