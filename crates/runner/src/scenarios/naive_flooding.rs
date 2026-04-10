use dash_chat_top_gen::ChatTopology;
use simdash_behavior::{
    network::RoundRobinNodeScheduler, node::BasicRoutingBehavior, routing::NaiveFloodingBehavior,
};
use simdash_model::network::NetworkModel;
use simdash_model::node::{NodeModel, NodeModelConfig};
use simdash_scenario::Scenario;
use simdash_state::network::NetworkState;
use simdash_state::routing::flooding::{FloodingOp, FloodingState};
use simdash_state::routing::RoutingState;

pub type NaiveFloodingBehaviorType =
    RoundRobinNodeScheduler<BasicRoutingBehavior<NaiveFloodingBehavior>>;

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
                    .map(|op| (op.id, FloodingOp::new(op)))
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
                    NodeModel::new(NodeModelConfig {
                        broadcast_interval: broadcast_interval.into(),
                    }),
                )
            })
            .collect(),
        topology,
    );
    let behavior = RoundRobinNodeScheduler::new(BasicRoutingBehavior::new(NaiveFloodingBehavior));
    Scenario::initial(model, initial_state, behavior)
}
