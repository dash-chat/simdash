//! Integration test of data propagation across a small contrived network

use rainbow_brackets::{RainbowBrackets, RainbowBracketsConfig};
use simdash_model_network::{NetworkAction, RoundRobinNodeScheduler};
use simdash_model_node::BasicNodeBehavior;
use simdash_protocol_ecco::{StateVector, util::make_ecco_op};
use simdash_routing_dispatch::BasicEccoBehavior;
use polestar_sim::Scenario;
use simdash_types::{LogId, Seq, Ticks, TopicId};

use named_id::{Nameable, Rename};

/// Node 0 and node 9 share topic 0, but they are only connected through a chain of
/// intermediate nodes that are not subscribed to any topic.  The test plants an op
/// on node 0 and runs many sync rounds, then checks whether node 9 received it.
///
/// NOTE: the assertion is currently commented out because the protocol does not yet
/// propagate ops to non-originating subscribers.  The test is here so the scenario
/// can be run and inspected as the protocol evolves.
#[test]
fn n_paths_scenario() {
    const SYNC_INTERVAL: Ticks = Ticks(10);
    const STEPS: usize = 20;

    let (graph, (alpha, omega), paths) = simdash_model_network::generate::n_paths([2, 3, 4]);
    let (model, mut state) =
        simdash_model_network::generate::make_model(graph.clone(), SYNC_INTERVAL);
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(BasicEccoBehavior)).sorted();

    state.graph.nodes.keys().for_each(|node_id| {
        node_id.with_short();
    });

    let topic = TopicId(0).with_short();

    let alpha_model = &model.nodes.get(&alpha).unwrap().routing.ecco;
    let omega_model = &model.nodes.get(&omega).unwrap().routing.ecco;

    let op = make_ecco_op(alpha, topic);

    // Node 0 "creates" an op for topic 0
    state
        .graph
        .update_node(alpha, |mut node| {
            node.routing.ecco.store_op(&alpha_model, op.clone());
            Ok((node, ()))
        })
        .unwrap();

    // Node N-1 subscribes to topic 0
    state
        .graph
        .update_node(omega, |mut node| {
            node.routing.ecco.subscribe(&omega_model, topic);
            Ok((node, ()))
        })
        .unwrap();

    let mut scenario = Scenario::initial(polestar::StateMachine::new(model, state), behavior);
    // .pretty_output();

    // Advance the network; each tick every node broadcasts its Ecco state
    // and effects from the previous tick are dispatched as consequences
    for _ in 0..STEPS {
        scenario.advance().unwrap();
    }

    // Check whether node 9 received the op
    let node_alpha = scenario.state().graph.node_state(alpha).unwrap();
    let node_omega = scenario.state().graph.node_state(omega).unwrap();
    let sv_alpha = node_alpha.routing.ecco.op_buffer.state_vector(topic);
    let sv_omega = node_omega.routing.ecco.op_buffer.state_vector(topic);
    assert_eq!(sv_alpha.len(), 1);
    assert_eq!(sv_alpha, sv_omega);
    let received = node_omega
        .routing
        .ecco
        .op_buffer
        .diff(0.into(), &StateVector::new([]));

    // TODO: uncomment once the protocol correctly propagates ops to remote subscribers
    assert!(
        !received.is_empty(),
        "node N-1 should have received the op from node 0"
    );

    let op_counts = (alpha..=omega)
        .map(|node| {
            scenario
                .state()
                .graph
                .node_state(node)
                .unwrap()
                .routing
                .ecco
                .op_buffer
                .ops
                .get(&topic)
                .map(|m| m.len())
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();

    let mut expected_op_counts = vec![0; op_counts.len()];
    expected_op_counts[*alpha as usize] = 1;
    expected_op_counts[*omega as usize] = 1;
    for path in &paths[0] {
        expected_op_counts[**path as usize] = 1;
    }

    assert_eq!(op_counts, expected_op_counts);
}
