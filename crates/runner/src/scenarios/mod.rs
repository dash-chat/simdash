mod naive_flooding;

use simdash_action::network::NetworkAction;
use simdash_behavior::{
    network::RoundRobinNodeScheduler, node::BasicRoutingBehavior, routing::NaiveFloodingBehavior,
};
use simdash_model::network::NetworkModel;
use simdash_scenario::Scenario;
use simdash_state::network::NetworkState;

type NaiveFloodingScenario =
    Scenario<NetworkModel, RoundRobinNodeScheduler<BasicRoutingBehavior<NaiveFloodingBehavior>>>;

/// Wraps the concrete scenario types so we can select them at runtime.
pub enum AnyScenario {
    NaiveFlooding(NaiveFloodingScenario),
}

impl AnyScenario {
    pub fn from_config(
        behavior: &str,
        initial_state: NetworkState,
        broadcast_interval: u32,
    ) -> anyhow::Result<Self> {
        match behavior {
            "naive-flooding" => Ok(Self::NaiveFlooding(naive_flooding::naive_flooding(
                initial_state,
                broadcast_interval,
            ))),
            other => anyhow::bail!("Unknown behavior: {other}"),
        }
    }

    /// Advance one "round": keep calling advance() until a Tick action is applied.
    pub fn step(&mut self) -> anyhow::Result<()> {
        match self {
            Self::NaiveFlooding(s) => step_until_tick(s),
        }
    }

    pub fn state(&self) -> &NetworkState {
        match self {
            Self::NaiveFlooding(s) => s.state().expect("scenario should have state"),
        }
    }
}

fn step_until_tick(scenario: &mut NaiveFloodingScenario) -> anyhow::Result<()> {
    loop {
        let prev_len = scenario.history().len();
        scenario.advance()?;
        let history = scenario.history();
        if history.len() > prev_len {
            if matches!(history.last(), Some(NetworkAction::Tick)) {
                break;
            }
        } else {
            // No action produced, behavior exhausted
            break;
        }
    }
    Ok(())
}
