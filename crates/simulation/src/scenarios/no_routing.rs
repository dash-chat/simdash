use dash_chat_top_gen::ChatTopology;
use simdash_behavior::node::BasicNodeBehavior;
use simdash_behavior::{network::RoundRobinNodeScheduler, routing::NoOpRoutingBehavior};
use simdash_model::network::NetworkModel;
use simdash_model::node::NodeModel;
use simdash_routing_dispatch::RoutingDispatchModel;
use simdash_scenario::Scenario;
use simdash_state::network::NetworkState;

pub type NoRoutingBehaviorType = RoundRobinNodeScheduler<BasicNodeBehavior<NoOpRoutingBehavior>>;

pub fn no_routing(
    initial_state: NetworkState,
    broadcast_interval: u32,
    topology: ChatTopology,
) -> Scenario<NetworkModel, NoRoutingBehaviorType> {
    let model = NetworkModel::new(
        NodeModel::new(RoutingDispatchModel::standard(broadcast_interval.into())),
        topology,
    );
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(NoOpRoutingBehavior));
    Scenario::initial(model, initial_state, behavior)
}
