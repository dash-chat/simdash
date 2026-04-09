use simdash_state::network::NetworkState;
use simdash_state::Graph;

use crate::scenarios::{AnyScenario, AVAILABLE_BEHAVIORS, naive_flooding, no_routing, step_until_tick};

pub struct Simulation {
    initial_state: NetworkState,
    current_step: usize,
    behavior_name: String,
    broadcast_interval: u32,
    scenario: AnyScenario,
}

impl Simulation {
    pub fn new(graph: Graph, behavior: &str, broadcast_interval: u32) -> anyhow::Result<Self> {
        let state = NetworkState { graph };
        let scenario = Self::make_scenario(behavior, &state, broadcast_interval)?;
        Ok(Self {
            initial_state: state,
            current_step: 0,
            behavior_name: behavior.to_string(),
            broadcast_interval,
            scenario,
        })
    }

    fn make_scenario(
        behavior: &str,
        state: &NetworkState,
        broadcast_interval: u32,
    ) -> anyhow::Result<AnyScenario> {
        match behavior {
            "naive-flooding" => Ok(AnyScenario::NaiveFlooding(
                naive_flooding::naive_flooding(state.clone(), broadcast_interval),
            )),
            "no-routing" => Ok(AnyScenario::NoRouting(
                no_routing::no_routing(state.clone(), broadcast_interval),
            )),
            other => anyhow::bail!("Unknown behavior: {other}"),
        }
    }

    pub fn current_state(&self) -> &NetworkState {
        self.scenario.state().expect("scenario should have state")
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }

    pub fn behavior_name(&self) -> &str {
        &self.behavior_name
    }

    pub fn available_behaviors(&self) -> &[&str] {
        AVAILABLE_BEHAVIORS
    }

    pub fn set_behavior(&mut self, behavior: &str) -> anyhow::Result<()> {
        // Validate behavior name before changing
        let scenario = Self::make_scenario(behavior, &self.initial_state, self.broadcast_interval)?;
        self.behavior_name = behavior.to_string();
        self.scenario = scenario;
        self.current_step = 0;
        Ok(())
    }

    pub fn step_forward(&mut self) -> anyhow::Result<()> {
        step_until_tick(&mut self.scenario)?;
        self.current_step += 1;
        Ok(())
    }

    pub fn step_back(&mut self) -> anyhow::Result<()> {
        if self.current_step == 0 {
            anyhow::bail!("Already at step 0");
        }
        let target = self.current_step - 1;
        self.jump_to(target)
    }

    pub fn jump_to(&mut self, step: usize) -> anyhow::Result<()> {
        self.scenario = Self::make_scenario(
            &self.behavior_name,
            &self.initial_state,
            self.broadcast_interval,
        )?;
        for _ in 0..step {
            step_until_tick(&mut self.scenario)?;
        }
        self.current_step = step;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.scenario = Self::make_scenario(
            &self.behavior_name,
            &self.initial_state,
            self.broadcast_interval,
        )
        .expect("scenario was valid at creation");
        self.current_step = 0;
    }
}
