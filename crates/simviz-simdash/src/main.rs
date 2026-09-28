//! Serve the simdash model through the generic simviz viewer.

use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use simviz::{Simulation, Viewer};
use simviz_simdash::{SimdashPresenter, read_config, scenario_from_config};

/// simviz server for the simdash model
#[derive(Parser)]
struct Cli {
    /// Path to a YAML or JSON config file
    #[arg(default_value = "config.yaml")]
    input: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Instead of serving, record this many steps to a JSON file and exit
    #[arg(long, value_name = "STEPS")]
    record: Option<usize>,

    /// Where to write the recording
    #[arg(long, default_value = "recording.json")]
    out: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let config = read_config(&cli.input)?;
    let scenario = scenario_from_config(config)?;
    let mut viewer = Viewer::new(Simulation::new(scenario), SimdashPresenter);

    if let Some(steps) = cli.record {
        let recording = viewer.record(steps)?;
        std::fs::write(&cli.out, serde_json::to_vec(&recording)?)?;
        println!("wrote {} frames to {}", recording.frames.len(), cli.out.display());
        return Ok(());
    }

    simviz::serve(viewer, SocketAddr::from(([127, 0, 0, 1], cli.port))).await
}
