use std::collections::HashSet;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use simdash_state::network::NetworkState;
use simdash_types::{NodeId, OpId, Operation};

use crate::sim::Simulation;

#[derive(Serialize, Deserialize)]
pub struct SimulationSnapshot {
    pub config: serde_json::Value,
    pub behavior: String,
    pub ticks: usize,
    pub elapsed_secs: f64,
    pub ticks_per_second: f64,
    pub message_delivery_rate: f64,
    pub total_ops: usize,
    pub nodes: Vec<NodeSnapshot>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeSnapshot {
    pub id: NodeId,
    pub ops_count: usize,
    pub contacts: Vec<NodeId>,
    pub groups: Vec<u64>,
    pub neighbors: Vec<NodeId>,
    pub storage_capacity: Option<u64>,
    pub stored_size: u64,
    pub ops: Vec<Operation>,
}

impl SimulationSnapshot {
    pub fn from_simulation(
        sim: &Simulation,
        all_op_ids: &HashSet<OpId>,
        config: serde_json::Value,
        ticks: usize,
        elapsed_secs: f64,
    ) -> Self {
        let state = sim.current_state();
        let delivery_rate = compute_delivery_rate(state, all_op_ids);

        let mut node_ids: Vec<NodeId> = state.graph.node_ids().collect();
        node_ids.sort();

        let nodes: Vec<NodeSnapshot> = node_ids
            .into_iter()
            .filter_map(|node_id| {
                let ns = state.graph.node_state(node_id)?;
                let neighbors: Vec<NodeId> = state
                    .graph
                    .graph()
                    .neighbors(node_id.into())
                    .map(NodeId::from)
                    .collect();

                let ops: Vec<Operation> = ns.chat.op_store.values().cloned().collect();

                Some(NodeSnapshot {
                    id: node_id,
                    ops_count: ops.len(),
                    contacts: ns.chat.contacts.iter().copied().collect(),
                    groups: ns.chat.groups.iter().copied().collect(),
                    neighbors,
                    storage_capacity: ns.chat.storage_capacity.map(|s| *s),
                    stored_size: *ns.chat.stored_size(),
                    ops,
                })
            })
            .collect();

        Self {
            config,
            behavior: sim.behavior_name().to_string(),
            ticks,
            elapsed_secs,
            ticks_per_second: if elapsed_secs > 0.0 {
                ticks as f64 / elapsed_secs
            } else {
                0.0
            },
            message_delivery_rate: delivery_rate,
            total_ops: all_op_ids.len(),
            nodes,
        }
    }
}

pub fn collect_all_op_ids(state: &NetworkState) -> HashSet<OpId> {
    let mut all_ops = HashSet::new();
    for node_id in state.graph.node_ids() {
        if let Some(node_state) = state.graph.node_state(node_id) {
            all_ops.extend(node_state.chat.op_store.keys().copied());
        }
    }
    all_ops
}

pub fn compute_delivery_rate(state: &NetworkState, all_op_ids: &HashSet<OpId>) -> f64 {
    if all_op_ids.is_empty() {
        return 1.0;
    }

    let node_count = state.graph.node_ids().count() as f64;
    if node_count == 0.0 {
        return 1.0;
    }

    let total_deliveries: usize = all_op_ids
        .iter()
        .map(|op_id| {
            state
                .graph
                .node_ids()
                .filter(|nid| {
                    state
                        .graph
                        .node_state(*nid)
                        .map_or(false, |ns| ns.chat.op_store.contains_key(op_id))
                })
                .count()
        })
        .sum();

    total_deliveries as f64 / (all_op_ids.len() as f64 * node_count)
}

/// Result of setting up a simulation from an input file.
pub struct SimSetup {
    pub sim: Simulation,
    pub config: serde_json::Value,
    pub start_tick: usize,
    pub prior_elapsed: f64,
}

/// Reads a JSON file and returns a `SimSetup`. If the file is a simulation
/// snapshot (has a `"config"` key), the simulation is resumed from the saved
/// tick. Otherwise the file is treated as a config and a fresh simulation is
/// created.
pub fn setup_from_input(path: &Path) -> anyhow::Result<SimSetup> {
    let contents = std::fs::read_to_string(path).context("Failed to read input file")?;
    let json: serde_json::Value =
        serde_json::from_str(&contents).context("Failed to parse input as JSON")?;

    setup_from_json(json)
}

/// Like `setup_from_input` but takes an already-parsed JSON value.
pub fn setup_from_json(json: serde_json::Value) -> anyhow::Result<SimSetup> {
    if json.get("config").is_some() {
        // This is a simulation snapshot — resume
        let prev: SimulationSnapshot =
            serde_json::from_value(json).context("Failed to parse simulation snapshot")?;

        let sim_config: simdash_generator::SimConfig = serde_json::from_value(prev.config.clone())
            .context("Failed to parse config from snapshot")?;
        let (dash_topology, seed) = simdash_generator::generate_topology_from_config(sim_config);

        println!(
            "Resuming from tick {} ({} nodes, {} edges)",
            prev.ticks,
            dash_topology.net.nodes.len(),
            dash_topology.net.edges.len(),
        );

        let mut sim = Simulation::new(dash_topology, seed)?;
        sim.set_behavior(&prev.behavior)?;
        sim.jump_to(prev.ticks)?;

        Ok(SimSetup {
            sim,
            config: prev.config,
            start_tick: prev.ticks,
            prior_elapsed: prev.elapsed_secs,
        })
    } else {
        // This is a config — start fresh
        let sim_config: simdash_generator::SimConfig = serde_json::from_value(json.clone())
            .context("Failed to parse config")?;
        let (dash_topology, seed) = simdash_generator::generate_topology_from_config(sim_config);

        println!(
            "Generated network topology: {} nodes, {} edges",
            dash_topology.net.nodes.len(),
            dash_topology.net.edges.len(),
        );

        let sim = Simulation::new(dash_topology, seed)?;

        Ok(SimSetup {
            sim,
            config: json,
            start_tick: 0,
            prior_elapsed: 0.0,
        })
    }
}
