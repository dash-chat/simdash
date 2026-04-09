use simdash_behavior::{network::RoundRobinNodeScheduler, node::BasicRoutingBehavior, routing::NaiveFloodingBehavior};
use simdash_model::network::NetworkModel;
use simdash_model::node::NodeModelConfig;
use simdash_scenario::Scenario;
use simdash_state::network::NetworkState;

pub fn naive_flooding(
    initial_state: NetworkState,
    broadcast_interval: u32,
) -> Scenario<NetworkModel, RoundRobinNodeScheduler<BasicRoutingBehavior<NaiveFloodingBehavior>>> {
    let model = NetworkModel::new(NodeModelConfig {
        broadcast_interval: broadcast_interval.into(),
    });
    let behavior =
        RoundRobinNodeScheduler::new(BasicRoutingBehavior::new(NaiveFloodingBehavior));
    Scenario::initial(model, initial_state, behavior)
}
