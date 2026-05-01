use std::path::Path;

use anyhow::{Context, Result};
use dash_chat_top_gen::{ChatStateGenerator, ChatTopologyConfig};
use net_topology::{TopologyConfig, TopologyGenerator};
use serde::Deserialize;
use simdash_model_network::ChatTopology;
pub use simdash_model_network::Graph;
use simdash_model_network::{NetworkState, NetworkTopologyState};

/// Top-level config struct that mirrors the full config file format.
#[derive(Debug, Deserialize)]
pub struct SimConfig {
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(rename = "net-top")]
    pub net_top: TopologyConfig,
    #[serde(rename = "dash-chat-top")]
    pub dash_chat_top: ChatTopologyConfig,
}

/// Parses a `SimConfig` from a file, selecting JSON or YAML based on the file extension.
pub fn parse_sim_config(config_path: &Path) -> Result<SimConfig> {
    let contents = std::fs::read_to_string(config_path).context("Failed to read config file")?;
    let ext = config_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "yaml" | "yml" => serde_yml::from_str(&contents).context("Failed to parse config as YAML"),
        _ => serde_json::from_str(&contents).context("Failed to parse config as JSON"),
    }
}

/// Reads a config file and generates the chat topology (intermediate representation).
pub fn generate_topology(config_path: &Path) -> Result<(ChatTopology, u64)> {
    let config = parse_sim_config(config_path)?;
    Ok(generate_topology_from_config(config))
}

/// Generates the chat topology from an already-parsed config.
pub fn generate_topology_from_config(config: SimConfig) -> (ChatTopology, u64) {
    let SimConfig {
        seed,
        net_top,
        dash_chat_top,
    } = config;

    let seed = seed.unwrap_or_else(|| rand::random());

    let topology = TopologyGenerator::new(net_top).generate(seed);
    let chat_gen = ChatStateGenerator {
        config: dash_chat_top,
    };
    let dash_topology = chat_gen.generate(seed, topology);

    (dash_topology, seed)
}

pub fn generate_graph(config_path: &Path) -> Result<Graph> {
    let (topology, seed) = generate_topology(&config_path)?;

    let graph = topology.into_graph(seed);
    Ok(graph)
}

pub fn generate_network_state(config_path: &Path) -> Result<NetworkState> {
    let (topology, seed) = generate_topology(&config_path)?;

    let graph = topology.into_graph(seed);

    Ok(NetworkState {
        graph,
        topology: NetworkTopologyState::new(seed),
    })
}
