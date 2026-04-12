pub mod naive_flooding;
pub mod no_routing;

use simdash_model::network::NetworkModel;
use simdash_routing::{RoutingModel, RoutingState};
use simdash_scenario::Scenario;

use self::naive_flooding::NaiveFloodingBehaviorType;
use self::no_routing::NoRoutingBehaviorType;

pub type NaiveFloodingScenario = Scenario<NetworkModel<RoutingModel>, NaiveFloodingBehaviorType>;
pub type NoRoutingScenario = Scenario<NetworkModel<RoutingModel>, NoRoutingBehaviorType>;

pub enum AnyScenario {
    NaiveFlooding(NaiveFloodingScenario),
    NoRouting(NoRoutingScenario),
}

impl AnyScenario {
    pub fn state(&self) -> Option<&simdash_state::network::NetworkState<RoutingState>> {
        match self {
            AnyScenario::NaiveFlooding(s) => s.state(),
            AnyScenario::NoRouting(s) => s.state(),
        }
    }

    pub fn advance(&mut self) -> anyhow::Result<bool> {
        match self {
            AnyScenario::NaiveFlooding(s) => s.advance(),
            AnyScenario::NoRouting(s) => s.advance(),
        }
    }

    pub fn topology(&self) -> &dash_chat_top_gen::ChatTopology {
        match self {
            AnyScenario::NaiveFlooding(s) => &s.model().topology,
            AnyScenario::NoRouting(s) => &s.model().topology,
        }
    }
}

pub const AVAILABLE_BEHAVIORS: &[&str] = &["naive-flooding", "no-routing"];
