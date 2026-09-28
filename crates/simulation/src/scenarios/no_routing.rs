use polestar::StateMachine;
use polestar_sim::Scenario;
use simdash_model_network::{NetworkModel, NetworkState, RoundRobinNodeScheduler};
use simdash_model_node::BasicNodeBehavior;
use simdash_routing_dispatch::NoOpRoutingBehavior;

pub type NoRoutingBehaviorType = RoundRobinNodeScheduler<BasicNodeBehavior<NoOpRoutingBehavior>>;

pub fn no_routing(
    model: NetworkModel,
    initial_state: NetworkState,
) -> Scenario<NoRoutingBehaviorType> {
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(NoOpRoutingBehavior));
    Scenario::initial(StateMachine::new(model, initial_state), behavior)
}
