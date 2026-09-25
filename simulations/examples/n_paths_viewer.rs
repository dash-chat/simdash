//! Runs the n_paths scenario in the visualization server.
//!
//! Start backend:  cargo run --example n_paths_viewer -p simdash-protocol-ecco
//! Start frontend: cd web && npm run dev -- --open

use std::net::SocketAddr;

use named_id::{Nameable, Rename};

use simdash_model_network::RoundRobinNodeScheduler;
use simdash_model_node::BasicNodeBehavior;
use simdash_protocol_ecco::util::make_ecco_op;
use simdash_routing_dispatch::BasicEccoBehavior;
use polestar::StateMachine;
use polestar_sim::Scenario;
use simdash_simulation::scenarios::PrettyOutput;
use simdash_simulation::sim::{NetworkBehavior, Simulation};
use simdash_types::{Ticks, TopicId};
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    const SYNC_INTERVAL: Ticks = Ticks(10);

    // Set up the n_paths scenario exactly as in protocols/ecco/tests/n_paths.rs
    let (graph, (alpha, omega), _paths) = simdash_model_network::generate::n_paths([3, 3, 5, 7]);
    let (model, mut state) = simdash_model_network::generate::make_model(graph, SYNC_INTERVAL);

    let mut node_ids = state.graph.nodes.keys().collect::<Vec<_>>();
    node_ids.sort();
    node_ids.iter().for_each(|node_id| {
        node_id.with_short();
        println!("{}: {}", node_id, node_id.renamed_ref());
    });

    let topic = TopicId(0).with_short();

    let alpha_model = &model.nodes.get(&alpha).unwrap().routing.ecco;
    let omega_model = &model.nodes.get(&omega).unwrap().routing.ecco;

    let op = make_ecco_op(alpha, topic);

    // Node alpha "creates" an op for topic 0
    state
        .graph
        .update_node(alpha, |mut node| {
            node.routing.ecco.store_op(alpha_model, op.clone());
            node.chat
                .store_op(op.encrypted.decrypt(&simdash_types::EncryptionKey))
                .unwrap();
            node.chat.groups.insert(topic);
            Ok((node, ()))
        })
        .unwrap();

    state
        .graph
        .update_node(omega, |mut node| {
            node.routing.ecco.subscribe(omega_model, topic);
            node.chat.groups.insert(topic);
            Ok((node, ()))
        })
        .unwrap();

    // Node omega subscribes to topic 0
    state
        .graph
        .update_node(omega, |mut node| {
            node.routing.ecco.subscribe(omega_model, topic);
            Ok((node, ()))
        })
        .unwrap();

    let behavior = NetworkBehavior::new(RoundRobinNodeScheduler::new(BasicNodeBehavior::new(
        BasicEccoBehavior,
    )));
    let scenario = Scenario::initial(StateMachine::new(model, state.clone()), behavior).pretty_output();
    let sim = Simulation::new(scenario, state, true).expect("failed to create simulation");
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let frontend = simdash_simulation::playback::spawn_playback_frontend(true);
    let backend = simdash_simulation::playback::playback_server(sim, addr);
    backend.await.await.unwrap();
    frontend.abort();
    // frontend.await.unwrap();
}
