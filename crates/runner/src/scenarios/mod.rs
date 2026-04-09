pub mod naive_flooding;
pub mod no_routing;

use simdash_action::network::NetworkAction;
use simdash_model::network::NetworkModel;
use simdash_scenario::Scenario;

use self::naive_flooding::NaiveFloodingBehaviorType;
use self::no_routing::NoRoutingBehaviorType;

pub type NaiveFloodingScenario = Scenario<NetworkModel, NaiveFloodingBehaviorType>;
pub type NoRoutingScenario = Scenario<NetworkModel, NoRoutingBehaviorType>;

pub enum AnyScenario {
    NaiveFlooding(NaiveFloodingScenario),
    NoRouting(NoRoutingScenario),
}

impl AnyScenario {
    pub fn state(&self) -> Option<&simdash_state::network::NetworkState> {
        match self {
            AnyScenario::NaiveFlooding(s) => s.state(),
            AnyScenario::NoRouting(s) => s.state(),
        }
    }

    pub fn advance(&mut self) -> anyhow::Result<Option<NetworkAction>> {
        match self {
            AnyScenario::NaiveFlooding(s) => s.advance(),
            AnyScenario::NoRouting(s) => s.advance(),
        }
    }
}

/// Advance one round: call advance() until the returned action is a Tick.
pub fn step_until_tick(scenario: &mut AnyScenario) -> anyhow::Result<()> {
    loop {
        match scenario.advance()? {
            Some(NetworkAction::Tick) => break,
            Some(_) => continue,
            None => break,
        }
    }
    Ok(())
}

pub const AVAILABLE_BEHAVIORS: &[&str] = &["naive-flooding", "no-routing"];
