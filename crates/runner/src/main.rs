use std::{collections::BTreeMap, fmt::Write, fs, path::PathBuf};

use clap::Parser;
use dash_chat_top_gen::DashStateGenerator;
use net_top_gen::TopologyGenerator;
use simdash_ids::NodeId;
use simdash_state::{dash::DashState, Graph};

/// simdash simulation runner
#[derive(Parser)]
struct Cli {
    /// Path to the JSON config file
    #[arg(default_value = "config.json")]
    config: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let contents = fs::read_to_string(&cli.config)
        .unwrap_or_else(|e| panic!("Failed to read '{}': {e}", cli.config.display()));

    let config: serde_json::Value = serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("Failed to parse '{}' as JSON: {e}", cli.config.display()));

    println!("Hello, world from simdash runner!");
    println!("Config loaded from '{}'", cli.config.display());

    let seed = config.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);

    if let Some(net_top_value) = config.get("net-top") {
        let net_top_json =
            serde_json::to_string(net_top_value).expect("Failed to serialize net-top config");
        let generator =
            TopologyGenerator::from_json(&net_top_json).expect("Failed to parse net-top config");
        let graph = generator.generate(seed);
        println!(
            "Generated network topology: {} nodes, {} edges",
            graph.graph().node_count(),
            graph.graph().edge_count(),
        );

        let dash_states = if let Some(dash_chat_value) = config.get("dash-chat-top") {
            let dash_chat_json = serde_json::to_string(dash_chat_value)
                .expect("Failed to serialize dash-chat-top config");
            let generator = DashStateGenerator::from_json(&dash_chat_json)
                .expect("Failed to parse dash-chat-top config");
            let states = generator.generate(&graph);
            println!("Generated dash chat state for {} nodes", states.len());
            Some(states)
        } else {
            None
        };

        fs::create_dir_all("out").expect("Failed to create output directory");

        let graph_bytes = postcard::to_stdvec(&graph).expect("Failed to serialize graph");
        let graph_path = "out/topology.postcard";
        fs::write(graph_path, graph_bytes).expect("Failed to write graph dump");
        println!("Serialized graph written to '{graph_path}'");

        let dot = generate_dot(&graph, dash_states.as_ref());
        let dot_path = "out/topology.dot";
        fs::write(dot_path, &dot).expect("Failed to write DOT file");
        println!("Graphviz diagram written to '{dot_path}'");
    }
}

fn generate_dot(graph: &Graph, dash_states: Option<&BTreeMap<NodeId, DashState>>) -> String {
    let mut dot = String::new();
    writeln!(dot, "digraph topology {{").unwrap();
    writeln!(dot, "    rankdir=LR;").unwrap();
    writeln!(dot, "    node [shape=record];").unwrap();

    // Build a reverse map from NodeIndex to NodeId
    let mut index_to_id: BTreeMap<petgraph::graph::NodeIndex, NodeId> = BTreeMap::new();
    for node_id in graph.node_ids() {
        if let Some(idx) = graph.node_index(node_id) {
            index_to_id.insert(idx, node_id);
        }
    }

    // Emit nodes with group labels
    for (&idx, &node_id) in &index_to_id {
        let id: u32 = *node_id;
        let groups_label = if let Some(states) = dash_states {
            if let Some(state) = states.get(&node_id) {
                let groups: Vec<String> = state.groups.iter().map(|g| g.to_string()).collect();
                if groups.is_empty() {
                    String::from("none")
                } else {
                    groups.join(", ")
                }
            } else {
                String::from("none")
            }
        } else {
            let node_state = graph.graph().node_weight(idx).unwrap();
            let groups: Vec<String> = node_state
                .dash()
                .groups
                .iter()
                .map(|g| g.to_string())
                .collect();
            if groups.is_empty() {
                String::from("none")
            } else {
                groups.join(", ")
            }
        };
        writeln!(
            dot,
            "    n{id} [label=\"{{Node {id}|groups: {groups_label}}}\"];",
        )
        .unwrap();
    }

    // Emit edges
    for edge in graph.graph().edge_indices() {
        if let Some((src_idx, dst_idx)) = graph.graph().edge_endpoints(edge) {
            let src_id: u32 = *index_to_id[&src_idx];
            let dst_id: u32 = *index_to_id[&dst_idx];
            writeln!(dot, "    n{src_id} -> n{dst_id};").unwrap();
        }
    }

    writeln!(dot, "}}").unwrap();
    dot
}
