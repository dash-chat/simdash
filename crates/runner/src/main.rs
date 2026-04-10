mod api;
mod scenarios;
mod sim;

use std::{fs, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use dash_chat_top_gen::ChatStateGenerator;
use net_top_gen::TopologyGenerator;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::api::AppState;
use crate::sim::Simulation;

/// simdash simulation server
#[derive(Parser)]
struct Cli {
    /// Path to the JSON config file
    #[arg(default_value = "config.json")]
    config: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    #[arg(long = "no-serve")]
    no_serve: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let contents = fs::read_to_string(&cli.config)
        .unwrap_or_else(|e| panic!("Failed to read '{}': {e}", cli.config.display()));

    let config: serde_json::Value = serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("Failed to parse '{}' as JSON: {e}", cli.config.display()));

    let seed = config.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);

    let behavior = config
        .get("behavior")
        .and_then(|v| v.as_str())
        .unwrap_or("naive-flooding");

    let broadcast_interval = config
        .get("broadcast-interval")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let net_top_value = config
        .get("net-top")
        .expect("config.json must have a 'net-top' key");

    let net_top_json =
        serde_json::to_string(net_top_value).expect("Failed to serialize net-top config");
    let generator =
        TopologyGenerator::from_json(&net_top_json).expect("Failed to parse net-top config");
    let graph = generator.generate(seed).into_graph();

    println!(
        "Generated network topology: {} nodes, {} edges",
        graph.graph().node_count(),
        graph.graph().edge_count(),
    );

    let graph = if let Some(dash_chat_value) = config.get("dash-chat-top") {
        let dash_chat_json = serde_json::to_string(dash_chat_value)
            .expect("Failed to serialize dash-chat-top config");
        let chat_gen = ChatStateGenerator::from_json(&dash_chat_json)
            .expect("Failed to parse dash-chat-top config");
        let graph = chat_gen.generate(graph);
        println!("Generated dash chat state");
        graph
    } else {
        graph
    };

    {
        std::fs::create_dir_all("gen").unwrap();
        let bytes = postcard::to_stdvec(&graph).unwrap();
        fs::write("gen/topology.postcard", bytes).unwrap();
        println!("Serialized graph to gen/topology.postcard");
    }

    if cli.no_serve {
        return;
    }

    println!("Using behavior: {behavior}");
    let sim = Simulation::new(graph, behavior, broadcast_interval)
        .unwrap_or_else(|e| panic!("Failed to create simulation: {e}"));
    let state: AppState = Arc::new(Mutex::new(sim));

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
