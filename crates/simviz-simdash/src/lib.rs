//! Shows the simdash network model through the generic `simviz` seam.
//!
//! [`SimdashPresenter`] is the per-model UI module. Today it only extracts the
//! topology; colouring and sidebar content are the trait defaults (none).
//! Reproducing the old viewer's op tracing, group highlighting and storage
//! read-outs means growing this presenter, not the viz.

use anyhow::Context;
use polestar::StateMachine;
use polestar_sim::Scenario;
use simdash_model_network::NetworkState;
use simdash_simulation::scenarios::sim_scenario::{create_model, seed_routing};
use simdash_simulation::sim::NetworkBehavior;
use simviz::{NodeId, Presenter, Topology};

/// Presenter for [`NetworkState`]: topology only, everything else noop.
pub struct SimdashPresenter;

impl Presenter for SimdashPresenter {
    type State = NetworkState;

    fn topology(&self, state: &NetworkState) -> Topology {
        let graph = state.graph.graph();
        let nodes = state.graph.node_ids().map(|id| NodeId(*id));
        let edges = graph.edge_indices().filter_map(|e| {
            let (a, b) = graph.edge_endpoints(e)?;
            Some((NodeId(a.index() as u32), NodeId(b.index() as u32)))
        });
        Topology::from_parts(nodes, edges)
    }
}

/// Build the standard simdash scenario (ecco routing) from a config value in
/// the same format `config.yaml` uses.
pub fn scenario_from_config(config: serde_json::Value) -> anyhow::Result<Scenario<NetworkBehavior>> {
    let sim_config: simdash_generator::SimConfig =
        serde_json::from_value(config).context("failed to parse sim config")?;
    let (chat_topology, seed) = simdash_generator::generate_topology_from_config(sim_config);
    let mut state = chat_topology.into_network_state(seed);
    let model = create_model(&state, 10u32.into());
    seed_routing(&model, &mut state);
    Ok(Scenario::initial(
        StateMachine::new(model, state),
        NetworkBehavior::standard(),
    ))
}

/// Read a YAML or JSON config file into a JSON value.
pub fn read_config(path: &std::path::Path) -> anyhow::Result<serde_json::Value> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let is_yaml = matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("yaml") | Some("yml")
    );
    if is_yaml {
        serde_yml::from_str(&contents).context("failed to parse YAML config")
    } else {
        serde_json::from_str(&contents).context("failed to parse JSON config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simviz::{Command, Simulation, Viewer};

    fn small_config() -> serde_json::Value {
        serde_json::json!({
            "seed": 1,
            "net-top": [{
                "element": "node", "subnet": "mesh", "number": 12,
                "rules": [{
                    "element": "link", "strategy": "per_node",
                    "count": { "min": 1, "max": 2 },
                    "destination": { "selection": "random", "subnet": "mesh" },
                    "ordinality": "bidirectional", "longevity": 0, "spawn_chance": 1
                }]
            }],
            "dash-chat-top": {
                "storage_capacity_per_node_kb": { "type": "fixed", "value": 4000 },
                "rules": [{
                    "type": "groups", "subnets": "all", "group_amount": 1,
                    "group_size": { "type": "fixed", "value": 4 },
                    "operations_count": { "type": "fixed", "value": 1 }
                }]
            }
        })
    }

    #[test]
    fn presenter_topology_mirrors_network_graph() {
        let scenario = scenario_from_config(small_config()).unwrap();
        let state = scenario.state();
        let topology = SimdashPresenter.topology(state);
        assert_eq!(topology.nodes().len(), state.graph.node_ids().count());
        assert_eq!(topology.edges().len(), state.graph.graph().edge_count());
        assert_eq!(topology.nodes().len(), 12);
    }

    #[test]
    fn viewer_steps_the_simdash_model_and_renders_nothing_extra() {
        let scenario = scenario_from_config(small_config()).unwrap();
        let mut viewer = Viewer::new(Simulation::new(scenario), SimdashPresenter);
        let f0 = viewer.frame();
        let f1 = viewer.handle(Command::StepForward).unwrap();
        assert_eq!(f1.step, 1);
        assert_eq!(f1.topology, f0.topology, "static topology in this config");
        assert!(f1.events.is_empty());
        let f0_again = viewer.handle(Command::StepBack).unwrap();
        assert_eq!(f0_again.step, 0);
    }
}
