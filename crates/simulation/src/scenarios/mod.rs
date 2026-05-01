pub mod no_routing;
pub mod sim_scenario;

use std::sync::Arc;

use simdash_model_network::{NetworkAction, NetworkModel};
use simdash_routing_dispatch::{
    BasicFloodingSyncBehavior, BasicEccoBehavior, NaiveFloodingBehavior,
};
use simdash_scenario::Scenario;

use self::no_routing::NoRoutingBehaviorType;
use self::sim_scenario::SimBehaviorType;

pub type NaiveFloodingScenario = Scenario<SimBehaviorType<NaiveFloodingBehavior>>;
pub type BasicFloodingSyncScenario = Scenario<SimBehaviorType<BasicFloodingSyncBehavior>>;
pub type EccoScenario = Scenario<SimBehaviorType<BasicEccoBehavior>>;
pub type NoRoutingScenario = Scenario<NoRoutingBehaviorType>;

pub enum AnyScenario {
    NaiveFlooding(NaiveFloodingScenario),
    BasicFloodingSync(BasicFloodingSyncScenario),
    Ecco(EccoScenario),
    NoRouting(NoRoutingScenario),
}

impl AnyScenario {
    pub fn state(&self) -> &simdash_model_network::NetworkState {
        match self {
            AnyScenario::NaiveFlooding(s) => s.state(),
            AnyScenario::BasicFloodingSync(s) => s.state(),
            AnyScenario::Ecco(s) => s.state(),
            AnyScenario::NoRouting(s) => s.state(),
        }
    }

    pub fn model(&self) -> &NetworkModel {
        match self {
            AnyScenario::NaiveFlooding(s) => s.model(),
            AnyScenario::BasicFloodingSync(s) => s.model(),
            AnyScenario::Ecco(s) => s.model(),
            AnyScenario::NoRouting(s) => s.model(),
        }
    }

    pub fn advance(&mut self) -> anyhow::Result<Vec<NetworkAction>> {
        match self {
            AnyScenario::NaiveFlooding(s) => s.advance(),
            AnyScenario::BasicFloodingSync(s) => s.advance(),
            AnyScenario::Ecco(s) => s.advance(),
            AnyScenario::NoRouting(s) => s.advance(),
        }
    }

    pub fn topology(&self) -> &simdash_model_network::ChatTopology {
        match self {
            AnyScenario::NaiveFlooding(s) => &s.model().topology,
            AnyScenario::BasicFloodingSync(s) => &s.model().topology,
            AnyScenario::Ecco(s) => &s.model().topology,
            AnyScenario::NoRouting(s) => &s.model().topology,
        }
    }
}

pub const AVAILABLE_BEHAVIORS: &[&str] = &[
    "naive-flooding",
    "basic-flooding-sync",
    "ecco",
    "no-routing",
];
