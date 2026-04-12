use dash_chat_top_gen::ChatTopology;
use simdash_behavior::network::RoundRobinNodeScheduler;
use simdash_behavior::node::BasicNodeBehavior;
use simdash_behavior::routing::BasicRoutingBehavior;
use simdash_model::network::NetworkModel;
use simdash_model::node::NodeModel;
use simdash_routing::{RoutingModel, RoutingState};
use simdash_routing_flooding::{FloodingOp, FloodingState};
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
                let routing_ops = n
                    .dash
                    .op_store
                    .values()
                    .map(|op| (op.id, FloodingOp::from_op(op.clone())))
                    .collect();
                n.routing = RoutingState {
                    flooding: FloodingState { ops: routing_ops },
                };
                Ok((n, ()))
            })
            .ok();
    }

    let model = NetworkModel::new(
        initial_state
            .graph
            .node_ids()
            .map(|id| {
                (
                    id,
                    NodeModel::new(RoutingModel::standard(broadcast_interval.into())),
                )
            })
            .collect(),
        topology,
    );
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(BasicRoutingBehavior));
    Scenario::initial(model, initial_state, behavior)
}
