pub mod flooding_scenario;
pub mod no_routing;

use simdash_model_network::NetworkModel;
use simdash_model_node::{BasicFloodingSyncBehavior, NaiveFloodingBehavior};
use simdash_scenario::Scenario;

use self::flooding_scenario::FloodingBehaviorType;
use self::no_routing::NoRoutingBehaviorType;

pub type NaiveFloodingScenario =
    Scenario<NetworkModel, FloodingBehaviorType<NaiveFloodingBehavior>>;
pub type BasicFloodingSyncScenario =
    Scenario<NetworkModel, FloodingBehaviorType<BasicFloodingSyncBehavior>>;
pub type NoRoutingScenario = Scenario<NetworkModel, NoRoutingBehaviorType>;

pub enum AnyScenario {
    NaiveFlooding(NaiveFloodingScenario),
    BasicFloodingSync(BasicFloodingSyncScenario),
    NoRouting(NoRoutingScenario),
}

impl AnyScenario {
    pub fn state(&self) -> Option<&simdash_model_network::NetworkState> {
        match self {
            AnyScenario::NaiveFlooding(s) => s.state(),
            AnyScenario::BasicFloodingSync(s) => s.state(),
            AnyScenario::NoRouting(s) => s.state(),
        }
    }

    pub fn advance(&mut self) -> anyhow::Result<bool> {
        match self {
            AnyScenario::NaiveFlooding(s) => s.advance(),
            AnyScenario::BasicFloodingSync(s) => s.advance(),
            AnyScenario::NoRouting(s) => s.advance(),
        }
    }

    pub fn topology(&self) -> &dash_chat_top_gen::ChatTopology {
        match self {
            AnyScenario::NaiveFlooding(s) => &s.model().topology,
            AnyScenario::BasicFloodingSync(s) => &s.model().topology,
            AnyScenario::NoRouting(s) => &s.model().topology,
        }
    }
}

pub const AVAILABLE_BEHAVIORS: &[&str] = &["naive-flooding", "basic-flooding-sync", "no-routing"];
