use dash_chat_top_gen::ChatTopology;
use simdash_behavior::{network::RoundRobinNodeScheduler, node::NodeBehavior1};
use simdash_model::network::NetworkModel;
use simdash_model::node::{NodeModel, NodeModelConfig};
use simdash_routing::{RoutingModel, RoutingState};
use simdash_routing_flooding::NaiveFloodingBehavior;
use simdash_scenario::Scenario;
use simdash_state::network::NetworkState;

pub type NaiveFloodingBehaviorType =
    RoundRobinNodeScheduler<NodeBehavior1<NaiveFloodingBehavior, RoutingModel>, RoutingModel>;

pub fn naive_flooding(
    mut initial_state: NetworkState<RoutingState>,
    broadcast_interval: u32,
    topology: ChatTopology,
) -> Scenario<NetworkModel<RoutingModel>, NaiveFloodingBehaviorType> {
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
    let behavior = RoundRobinNodeScheduler::new(NodeBehavior1::new(NaiveFloodingBehavior));
    Scenario::initial(model, initial_state, behavior)
}
