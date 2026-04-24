use std::collections::{HashMap, HashSet};

use dash_chat_top_gen::ChatTopology;
use simdash_model_network::{NetworkState, NetworkTopologyState};
use simdash_routing_dispatch::{BasicFloodingSyncBehavior, NaiveFloodingBehavior};

use crate::scenarios::{flooding_scenario, no_routing, AnyScenario, AVAILABLE_BEHAVIORS};

/// Snapshot of op IDs per node, used to compute deltas.
pub type OpSnapshot = HashMap<u32, HashSet<u32>>;

const DEFAULT_BEHAVIOR: &str = "naive-flooding";
const DEFAULT_BROADCAST_INTERVAL: u32 = 1;

pub struct Simulation {
    seed: u64,
    behavior_name: String,
    current_step: usize,
    scenario: AnyScenario,
    seq: u64,
}

impl Simulation {
    pub fn new(topology: ChatTopology, seed: u64) -> anyhow::Result<Self> {
        let state = Self::build_initial_state(&topology, seed);
        let scenario = Self::make_scenario(DEFAULT_BEHAVIOR, &state, &topology)?;
        Ok(Self {
            seed,
            behavior_name: DEFAULT_BEHAVIOR.to_string(),
            current_step: 0,
            scenario,
            seq: 0,
        })
    }

    fn build_initial_state(topology: &ChatTopology, seed: u64) -> NetworkState {
        let graph = topology.clone().into_graph(seed);
        NetworkState {
            graph,
            topology: NetworkTopologyState::new(seed),
        }
    }

    fn make_scenario(
        behavior: &str,
        state: &NetworkState,
        topology: &ChatTopology,
    ) -> anyhow::Result<AnyScenario> {
        match behavior {
            "naive-flooding" => Ok(AnyScenario::NaiveFlooding(
                flooding_scenario::flooding_scenario(
                    state.clone(),
                    DEFAULT_BROADCAST_INTERVAL,
                    topology.clone(),
                    NaiveFloodingBehavior,
                ),
            )),
            "basic-flooding-sync" => Ok(AnyScenario::BasicFloodingSync(
                flooding_scenario::flooding_scenario(
                    state.clone(),
                    DEFAULT_BROADCAST_INTERVAL,
                    topology.clone(),
                    BasicFloodingSyncBehavior,
                ),
            )),
            // "stig" => Ok(AnyScenario::Stig(
            //     flooding_scenario::flooding_scenario(
            //         state.clone(),
            //         DEFAULT_BROADCAST_INTERVAL,
            //         topology.clone(),
            //         BasicFloodingSyncBehavior,
            //     ),
            // )),
            "no-routing" => Ok(AnyScenario::NoRouting(no_routing::no_routing(
                state.clone(),
                DEFAULT_BROADCAST_INTERVAL,
                topology.clone(),
            ))),
            other => anyhow::bail!("Unknown behavior: {other}"),
        }
    }

    fn topology(&self) -> &ChatTopology {
        self.scenario.topology()
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
        let topology = self.topology().clone();
        let state = Self::build_initial_state(&topology, self.seed);
        let scenario = Self::make_scenario(behavior, &state, &topology)?;
        self.behavior_name = behavior.to_string();
        self.scenario = scenario;
        self.current_step = 0;
        Ok(())
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn bump_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    /// Capture the current op IDs per node before a step.
    pub fn snapshot_ops(&self) -> OpSnapshot {
        let state = self.current_state();
        let mut snap = HashMap::new();
        for node_id in state.graph.node_ids() {
            if let Some(node_state) = state.graph.node_state(node_id) {
                snap.insert(
                    *node_id,
                    node_state.chat.op_store.keys().map(|id| **id).collect(),
                );
            }
        }
        snap
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
        let topology = self.topology().clone();
        let state = Self::build_initial_state(&topology, self.seed);
        self.scenario = Self::make_scenario(&self.behavior_name, &state, &topology)?;
        for _ in 0..step {
            self.scenario.advance()?;
        }
        self.current_step = step;
        Ok(())
    }

    pub fn reset(&mut self) {
        let topology = self.topology().clone();
        let state = Self::build_initial_state(&topology, self.seed);
        self.scenario = Self::make_scenario(&self.behavior_name, &state, &topology)
            .expect("scenario was valid at creation");
        self.current_step = 0;
    }
}
