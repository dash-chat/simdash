use dash_chat_top_gen::ChatTopology;
use simdash_behavior::{
    network::RoundRobinNodeScheduler, node::NodeBehavior1, routing::NoOpRoutingBehavior,
};
use simdash_model::network::NetworkModel;
use simdash_model::node::{NodeModel, NodeModelConfig};
use simdash_scenario::Scenario;
use simdash_state::network::NetworkState;

pub type NoRoutingBehaviorType = RoundRobinNodeScheduler<NodeBehavior1<NoOpRoutingBehavior>>;

pub fn no_routing(
    initial_state: NetworkState,
    broadcast_interval: u32,
    topology: ChatTopology,
) -> Scenario<NetworkModel, NoRoutingBehaviorType> {
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
    let behavior = RoundRobinNodeScheduler::new(NodeBehavior1::new(NoOpRoutingBehavior));
    Scenario::initial(model, initial_state, behavior)
}
