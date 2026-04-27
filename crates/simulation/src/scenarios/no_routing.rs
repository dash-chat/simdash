use dash_chat_top_gen::ChatTopology;
use simdash_model_network::{NetworkModel, NetworkState, RoundRobinNodeScheduler};
use simdash_model_node::{BasicNodeBehavior, NodeModel};
use simdash_routing_dispatch::{NoOpRoutingBehavior, RoutingDispatchModel};
use simdash_scenario::Scenario;

pub type NoRoutingBehaviorType = RoundRobinNodeScheduler<BasicNodeBehavior<NoOpRoutingBehavior>>;

pub fn no_routing(
    initial_state: NetworkState,
    broadcast_interval: u32,
    topology: ChatTopology,
) -> Scenario<NoRoutingBehaviorType> {
    let model = NetworkModel::new(
        initial_state
            .graph
            .node_ids()
            .map(|id| {
                (
                    id,
                    NodeModel::new(RoutingDispatchModel::standard(broadcast_interval.into())),
                )
            })
            .collect(),
        topology,
    );
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(NoOpRoutingBehavior));
    Scenario::initial(model, initial_state, behavior)
}
