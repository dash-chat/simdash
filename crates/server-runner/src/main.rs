mod api;

use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::api::AppState;
use simdash_simulation::snapshot::setup_from_input;

/// simdash simulation server
#[derive(Parser)]
struct Cli {
    /// Path to a JSON config file or a previous simulation snapshot to resume
    #[arg(default_value = "config.json")]
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

    let state: AppState = Arc::new(Mutex::new(setup.sim));

    let app = Router::new()
        .route("/api/state", get(api::get_state))
        .route("/api/step/forward", post(api::step_forward))
        .route("/api/step/back", post(api::step_back))
        .route("/api/reset", post(api::reset))
        .route("/api/behaviors", get(api::get_behaviors))
        .route("/api/behavior", post(api::set_behavior))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.port));
    println!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
