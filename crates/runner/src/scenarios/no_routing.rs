use simdash_behavior::{
    network::RoundRobinNodeScheduler, node::BasicRoutingBehavior, routing::NoOpRoutingBehavior,
};
use simdash_model::network::NetworkModel;
use simdash_model::node::NodeModelConfig;
use simdash_scenario::Scenario;
use simdash_state::network::NetworkState;

pub type NoRoutingBehaviorType = RoundRobinNodeScheduler<BasicRoutingBehavior<NoOpRoutingBehavior>>;

pub fn no_routing(
    initial_state: NetworkState,
    broadcast_interval: u32,
) -> Scenario<NetworkModel, NoRoutingBehaviorType> {
    let model = NetworkModel::new(NodeModelConfig {
        broadcast_interval: broadcast_interval.into(),
    });
    let behavior = RoundRobinNodeScheduler::new(BasicRoutingBehavior::new(NoOpRoutingBehavior));
    Scenario::initial(model, initial_state, behavior)
}
