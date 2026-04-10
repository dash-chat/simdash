use dash_chat_top_gen::ChatTopology;
use simdash_state::network::NetworkState;

use crate::scenarios::{naive_flooding, no_routing, AnyScenario, AVAILABLE_BEHAVIORS};

pub struct Simulation {
    initial_state: NetworkState,
    topology: ChatTopology,
    current_step: usize,
    behavior_name: String,
    broadcast_interval: u32,
    scenario: AnyScenario,
}

impl Simulation {
    pub fn new(
        topology: ChatTopology,
        behavior: &str,
        broadcast_interval: u32,
        seed: u64,
    ) -> anyhow::Result<Self> {
        let graph = topology.clone().into_graph(seed);
        let state = NetworkState { graph };
        let scenario = Self::make_scenario(behavior, &state, broadcast_interval, topology.clone())?;
        Ok(Self {
            initial_state: state,
            topology: topology,
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
        topology: ChatTopology,
    ) -> anyhow::Result<AnyScenario> {
        match behavior {
            "naive-flooding" => Ok(AnyScenario::NaiveFlooding(naive_flooding::naive_flooding(
                state.clone(),
                broadcast_interval,
                topology,
            ))),
            "no-routing" => Ok(AnyScenario::NoRouting(no_routing::no_routing(
                state.clone(),
                broadcast_interval,
                topology,
            ))),
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
        let scenario = Self::make_scenario(
            behavior,
            &self.initial_state,
            self.broadcast_interval,
            self.topology.clone(),
        )?;
        self.behavior_name = behavior.to_string();
        self.scenario = scenario;
        self.current_step = 0;
        Ok(())
    }

    pub fn step_forward(&mut self) -> anyhow::Result<()> {
        self.scenario.advance()?;
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
            self.topology.clone(),
        )?;
        for _ in 0..step {
            self.scenario.advance()?;
        }
        self.current_step = step;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.scenario = Self::make_scenario(
            &self.behavior_name,
            &self.initial_state,
            self.broadcast_interval,
            self.topology.clone(),
        )
        .expect("scenario was valid at creation");
        self.current_step = 0;
    }
}
