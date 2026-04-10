use std::path::Path;

use anyhow::{Context, Result};
use dash_chat_top_gen::ChatStateGenerator;
pub use dash_chat_top_gen::ChatTopology;
use net_topology::TopologyGenerator;
pub use simdash_state::graph::Graph;
use simdash_state::network::{NetworkState, NetworkTopologyState};

/// Reads a JSON config file and generates the chat topology (intermediate representation).
pub fn generate_topology(config_path: &Path) -> Result<(ChatTopology, u64)> {
    let contents = std::fs::read_to_string(config_path).context("Failed to read config file")?;

    let config: serde_json::Value =
        serde_json::from_str(&contents).context("Failed to parse config as JSON")?;

    let seed = config.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);

    let net_top_value = config
        .get("net-top")
        .context("config must have a 'net-top' key")?;

    let net_top_json = serde_json::to_string(net_top_value)?;
    let generator = TopologyGenerator::from_json(&net_top_json)?;
    let topology = generator.generate(seed);

    let dash_topology: ChatTopology = if let Some(dash_chat_value) = config.get("dash-chat-top") {
        let dash_chat_json = serde_json::to_string(dash_chat_value)?;
        let chat_gen = ChatStateGenerator::from_json(&dash_chat_json)?;
        chat_gen.generate(seed, topology)
    } else {
        ChatTopology::from_net_topology(topology)
    };

    Ok((dash_topology, seed))
}

pub fn generate_graph<RS: Default + Clone>(config_path: &Path) -> Result<Graph<RS>> {
    let (topology, seed) = generate_topology(&config_path)?;

    let graph = topology.into_graph(seed);
    Ok(graph)
}

pub fn generate_network_state<RS: Default + Clone>(config_path: &Path) -> Result<NetworkState<RS>> {
    let (topology, seed) = generate_topology(&config_path)?;

    let graph = topology.into_graph(seed);

    Ok(NetworkState {
        graph,
        topology: NetworkTopologyState::new(seed),
    })
}
