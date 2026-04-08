use std::{fs, path::PathBuf};

use clap::Parser;
use net_top_gen::TopologyGenerator;

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
    }
}
