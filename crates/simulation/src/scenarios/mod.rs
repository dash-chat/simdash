pub mod no_routing;
pub mod sim_scenario;

use polestar_sim::Scenario;
use simdash_model_network::{NetworkAction, NetworkModel};
use simdash_routing_dispatch::{
    BasicEccoBehavior, BasicFloodingSyncBehavior, NaiveFloodingBehavior,
};
use simdash_types::Behavior;

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

    pub fn advance(&mut self) -> anyhow::Result<Vec<NetworkAction>> {
        match self {
            AnyScenario::NaiveFlooding(s) => s.advance(),
            AnyScenario::BasicFloodingSync(s) => s.advance(),
            AnyScenario::Ecco(s) => s.advance(),
            AnyScenario::NoRouting(s) => s.advance(),
        }
    }
}

pub const AVAILABLE_BEHAVIORS: &[&str] = &[
    "naive-flooding",
    "basic-flooding-sync",
    "ecco",
    "no-routing",
];

/// Console output helpers for NetworkModel scenarios.
pub trait PrettyOutput {
    /// Print some super helpful output to the console for NetworkModel scenarios.
    fn pretty_output(self) -> Self;
}

impl<B> PrettyOutput for Scenario<B>
where
    B: Behavior<Model = NetworkModel>,
{
    fn pretty_output(self) -> Self {
        use rainbow_brackets::{BracketPair, RainbowBrackets, RainbowBracketsConfig};
        use simdash_types::Rename;

        let mut rb = RainbowBracketsConfig::default();
        rb.brackets.push(BracketPair::new('⟪', '⟫'));
        let rb2 = rb.clone();

        self.on_step(|s| {
            println!();
            println!("========= STEP {} =========", s.steps());
            println!();
        })
        .on_action("actions", move |action| match action {
            NetworkAction::Trace(message) => {
                println!("\n{:?}", message);
            }
            action => {
                println!("⟵ {}", action.renamed_ref().rainbow_brackets_with(&rb));
            }
        })
        .on_fx("fx", move |fx| {
            if fx.len() > 0 {
                println!("  ⤷ {}\n", fx.renamed_ref().rainbow_brackets_with(&rb2));
            }
        })
    }
}
