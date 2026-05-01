use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use simdash_model_network::{NetworkModel, NetworkState};
use simdash_routing_dispatch::{
    BasicEccoBehavior, BasicFloodingSyncBehavior, NaiveFloodingBehavior,
};
use simdash_types::Behavior;

use crate::scenarios::sim_scenario::sim_scenario as make_sim_scenario;
use crate::scenarios::{no_routing, AnyScenario, AVAILABLE_BEHAVIORS};

/// Snapshot of op IDs per node, used to compute deltas.
pub type OpSnapshot = HashMap<u32, HashSet<u32>>;

const DEFAULT_BEHAVIOR: &str = "ecco";

pub struct Simulation {
    model: NetworkModel,
    initial_state: NetworkState,
    behavior_name: String,
    current_step: usize,
    scenario: AnyScenario,
    seq: u64,
    verbose: bool,
}

impl Simulation {
    /// Create a simulation from a pre-built model and initial state.
    ///
    /// `initial_state` should already have routing state seeded if needed
    /// (use `sim_scenario::seed_routing` for config-based setups).
    pub fn new(
        model: NetworkModel,
        initial_state: NetworkState,
        verbose: bool,
    ) -> anyhow::Result<Self> {
        let scenario = Self::make_scenario(
            DEFAULT_BEHAVIOR,
            Arc::clone(&model),
            initial_state.clone(),
            verbose,
        )?;
        Ok(Self {
            model,
            initial_state,
            behavior_name: DEFAULT_BEHAVIOR.to_string(),
            current_step: 0,
            scenario,
            seq: 0,
            verbose,
        })
    }

    fn make_scenario(
        behavior: &str,
        model: NetworkModel,
        state: NetworkState,
        verbose: bool,
    ) -> anyhow::Result<AnyScenario> {
        match behavior {
            "naive-flooding" => Ok(AnyScenario::NaiveFlooding(make_sim_scenario(
                model,
                state,
                NaiveFloodingBehavior,
            ))),
            "basic-flooding-sync" => Ok(AnyScenario::BasicFloodingSync(make_sim_scenario(
                model,
                state,
                BasicFloodingSyncBehavior,
            ))),
            "ecco" => Ok(AnyScenario::Ecco(decorate_scenario(
                verbose,
                make_sim_scenario(model, state, BasicEccoBehavior),
            ))),
            "no-routing" => Ok(AnyScenario::NoRouting(no_routing::no_routing(model, state))),
            other => anyhow::bail!("Unknown behavior: {other}"),
        }
    }

    pub fn current_state(&self) -> &NetworkState {
        self.scenario.state()
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
        let scenario = Self::make_scenario(
            behavior,
            Arc::clone(&self.model),
            self.initial_state.clone(),
            self.verbose,
        )?;
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
        self.scenario = Self::make_scenario(
            &self.behavior_name,
            Arc::clone(&self.model),
            self.initial_state.clone(),
            self.verbose,
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
            Arc::clone(&self.model),
            self.initial_state.clone(),
            self.verbose,
        )
        .expect("scenario was valid at creation");
        self.current_step = 0;
    }
}

fn decorate_scenario<B: Behavior<Model = NetworkModel>>(
    yes: bool,
    scenario: simdash_scenario::Scenario<B>,
) -> simdash_scenario::Scenario<B> {
    use rainbow_brackets::{RainbowBrackets, RainbowBracketsConfig};
    use simdash_model_network::NetworkAction;
    use simdash_types::Rename;

    if !yes {
        return scenario;
    }

    scenario
        .on_step(|s| {
            println!();
            println!("========= STEP {} =========", s.steps());
            println!();
        })
        .on_action("actions", |action| {
            let rb = RainbowBracketsConfig::default()
                .colors(vec![
                    rainbow_brackets::Color::Blue,
                    rainbow_brackets::Color::White,
                ])
                .mode(rainbow_brackets::Mode::OuterText);
            match action {
                NetworkAction::Trace(message) => {
                    println!("\n{:?}", message);
                }
                action => {
                    println!("⟵ {}", action.renamed_ref().rainbow_brackets_with(&rb));
                }
            }
        })
        .on_fx("fx", |fx| {
            let rb = RainbowBracketsConfig::default()
                .colors(vec![
                    rainbow_brackets::Color::Blue,
                    rainbow_brackets::Color::White,
                ])
                .mode(rainbow_brackets::Mode::OuterText);
            if fx.len() > 0 {
                println!("  ⤷ {}\n", fx.renamed_ref().rainbow_brackets_with(&rb));
            }
        })
}
