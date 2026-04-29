//! Runs the n_paths scenario in the visualization server.
//!
//! Start backend:  cargo run --example n_paths_viewer -p simdash-protocol-stig
//! Start frontend: cd web && npm run dev -- --open

use std::net::SocketAddr;
use std::sync::Arc;

use named_id::{Nameable, Rename};

use simdash_model_network::RoundRobinNodeScheduler;
use simdash_model_node::BasicNodeBehavior;
use simdash_protocol_stig::util::make_stig_op;
use simdash_routing_dispatch::BasicStigBehavior;
use simdash_scenario::Scenario;
use simdash_simulation::sim::{NetworkBehavior, Simulation};
use simdash_types::{LogId, Seq, Ticks, TopicId};
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    const SYNC_INTERVAL: Ticks = Ticks(10);

    // Set up the n_paths scenario exactly as in protocols/stig/tests/n_paths.rs
    let (graph, (alpha, omega), _paths) = simdash_model_network::generate::n_paths([2, 3, 4]);
    let (model, mut state) = simdash_model_network::generate::make_model(graph, SYNC_INTERVAL);

    let mut node_ids = state.graph.nodes.keys().collect::<Vec<_>>();
    node_ids.sort();
    node_ids.iter().for_each(|node_id| {
        node_id.with_short();
        println!("{}: {}", node_id, node_id.renamed_ref());
    });

    let topic = TopicId(0).with_short();

    let alpha_model = &model.nodes.get(&alpha).unwrap().routing.stig;
    let omega_model = &model.nodes.get(&omega).unwrap().routing.stig;

    let op = make_stig_op(topic, LogId(1).with_short(), Seq(0));

    // Node alpha "creates" an op for topic 0
    state
        .graph
        .update_node(alpha, |mut node| {
            node.routing.stig.store_op(alpha_model, op.clone());
            node.chat
                .store_op(op.encrypted.decrypt(&simdash_types::EncryptionKey))
                .unwrap();
            Ok((node, ()))
        })
        .unwrap();

    // Node omega subscribes to topic 0
    state
        .graph
        .update_node(omega, |mut node| {
            node.routing.stig.subscribe(omega_model, topic);
            Ok((node, ()))
        })
        .unwrap();

    let behavior = NetworkBehavior::new(RoundRobinNodeScheduler::new(BasicNodeBehavior::new(
        BasicStigBehavior,
    )));
    let scenario = Scenario::initial(model, state.clone(), behavior);
    let sim = Simulation::new(scenario, state, true).expect("failed to create simulation");
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let frontend = simdash_simulation::playback::spawn_playback_frontend(true);
    let backend = simdash_simulation::playback::playback_server(sim, addr);
    backend.await.await.unwrap();
    frontend.abort();
    // frontend.await.unwrap();
}
