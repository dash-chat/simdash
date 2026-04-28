pub mod no_routing;
pub mod sim_scenario;

use simdash_model_network::NetworkAction;
use simdash_routing_dispatch::{
    BasicFloodingSyncBehavior, BasicStigBehavior, NaiveFloodingBehavior,
};
use simdash_scenario::Scenario;

use self::no_routing::NoRoutingBehaviorType;
use self::sim_scenario::SimBehaviorType;

pub type NaiveFloodingScenario = Scenario<SimBehaviorType<NaiveFloodingBehavior>>;
pub type BasicFloodingSyncScenario = Scenario<SimBehaviorType<BasicFloodingSyncBehavior>>;
pub type StigScenario = Scenario<SimBehaviorType<BasicStigBehavior>>;
pub type NoRoutingScenario = Scenario<NoRoutingBehaviorType>;

pub enum AnyScenario {
    NaiveFlooding(NaiveFloodingScenario),
    BasicFloodingSync(BasicFloodingSyncScenario),
    Stig(StigScenario),
    NoRouting(NoRoutingScenario),
}

impl AnyScenario {
    pub fn state(&self) -> &simdash_model_network::NetworkState {
        match self {
            AnyScenario::NaiveFlooding(s) => s.state(),
            AnyScenario::BasicFloodingSync(s) => s.state(),
            AnyScenario::Stig(s) => s.state(),
            AnyScenario::NoRouting(s) => s.state(),
        }
    }

    pub fn advance(&mut self) -> anyhow::Result<Vec<NetworkAction>> {
        match self {
            AnyScenario::NaiveFlooding(s) => s.advance(),
            AnyScenario::BasicFloodingSync(s) => s.advance(),
            AnyScenario::Stig(s) => s.advance(),
            AnyScenario::NoRouting(s) => s.advance(),
        }
    }

    pub fn topology(&self) -> &dash_chat_top_gen::ChatTopology {
        match self {
            AnyScenario::NaiveFlooding(s) => &s.model().topology,
            AnyScenario::BasicFloodingSync(s) => &s.model().topology,
            AnyScenario::Stig(s) => &s.model().topology,
            AnyScenario::NoRouting(s) => &s.model().topology,
        }
    }
}

pub const AVAILABLE_BEHAVIORS: &[&str] = &[
    "naive-flooding",
    "basic-flooding-sync",
    "stig",
    "no-routing",
];
