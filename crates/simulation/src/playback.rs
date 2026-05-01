mod server;
use server::*;

use crate::sim::Simulation;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn playback_server(
    sim: Simulation,
    addr: core::net::SocketAddr,
) -> axum::serve::Serve<tokio::net::TcpListener, axum::Router<()>, axum::Router<()>> {
    use axum::{
        routing::{get, post},
        Router,
    };
    use tower_http::cors::CorsLayer;

    let state: PlaybackState = Arc::new(Mutex::new(sim));

    let app = Router::new()
        .route("/api/state", get(get_state))
        .route("/api/step/forward", post(step_forward))
        .route("/api/step/back", post(step_back))
        .route("/api/reset", post(reset))
        .route("/api/behaviors", get(get_behaviors))
        .route("/api/behavior", post(set_behavior))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
}

pub fn spawn_playback_frontend(open: bool) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        use std::process::{Command, Stdio};

        let output = false;

        let _child = Command::new("npm")
            .arg("run")
            .arg("dev")
            .arg("--")
            .arg(if open { "--open" } else { "" })
            .current_dir("web")
            .stdout(if output {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .stderr(if output {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .spawn()
            .expect("failed to start web dev server");
    })
}
