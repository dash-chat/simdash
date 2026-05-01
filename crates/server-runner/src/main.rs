use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

use simdash_simulation::snapshot::setup_from_input;

/// simdash simulation server
#[derive(Parser)]
struct Cli {
    /// Path to a JSON config file or a previous simulation snapshot to resume
    #[arg(default_value = "config.yaml")]
    input: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let setup =
        setup_from_input(&cli.input).unwrap_or_else(|e| panic!("Failed to set up simulation: {e}"));

    let server = simdash_simulation::playback::playback_server(
        setup.sim,
        SocketAddr::from(([127, 0, 0, 1], cli.port)),
    );
    server.await.await.unwrap();
}
