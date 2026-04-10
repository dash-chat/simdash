use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

/// Generate initial simulation state from a JSON config file.
#[derive(Parser)]
struct Cli {
    /// Path to the JSON config file
    #[arg(default_value = "config.json")]
    config: PathBuf,

    /// Path to the output postcard file
    #[arg(short, long, default_value = "gen/topology.postcard")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let (topology, seed) = simdash_generator::generate(&cli.config)?;

    let graph = topology.into_graph(seed);

    let bytes = postcard::to_stdvec(&graph).context("Failed to serialize graph")?;

    if let Some(parent) = cli.output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    std::fs::write(&cli.output, &bytes)
        .with_context(|| format!("Failed to write {}", cli.output.display()))?;

    println!(
        "Generated {} bytes to {}",
        bytes.len(),
        cli.output.display()
    );

    Ok(())
}
