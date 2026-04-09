pub mod naive_flooding;

use simdash_action::network::NetworkAction;
use simdash_model::network::NetworkModel;
use simdash_scenario::Scenario;

use self::naive_flooding::NaiveFloodingBehaviorType;

pub type NaiveFloodingScenario = Scenario<NetworkModel, NaiveFloodingBehaviorType>;

/// Advance one round: call advance() until the returned action is a Tick.
pub fn step_until_tick(scenario: &mut NaiveFloodingScenario) -> anyhow::Result<()> {
    loop {
        match scenario.advance()? {
            Some(NetworkAction::Tick) => break,
            Some(_) => continue,
            None => break,
        }
    }
    Ok(())
}
