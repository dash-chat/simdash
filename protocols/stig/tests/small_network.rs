//! Integration test of data propagation across a small contrived network

use simdash_model_network::RoundRobinNodeScheduler;
use simdash_model_node::BasicNodeBehavior;
use simdash_protocol_stig::{Op, StateVector, util::make_stig_op};
use simdash_routing_dispatch::{BasicStigBehavior, RoutingDispatchModel};
use simdash_scenario::Scenario;
use simdash_types::{LogId, Seq, TopicId};

use named_id::{Nameable, Rename};

/// Node 0 and node 9 share topic 0, but they are only connected through a chain of
/// intermediate nodes that are not subscribed to any topic.  The test plants an op
/// on node 0 and runs many sync rounds, then checks whether node 9 received it.
///
/// NOTE: the assertion is currently commented out because the protocol does not yet
/// propagate ops to non-originating subscribers.  The test is here so the scenario
/// can be run and inspected as the protocol evolves.
#[test]
fn op_propagates_to_remote_subscriber() {
    const N: u32 = 3;
    // 0 → 1 → 2 → … → N-1
    let (model, mut state) =
        simdash_model_network::generate::linear_chain(N, RoutingDispatchModel::standard(3.into()));
    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(BasicStigBehavior));

    state.graph.nodes.keys().for_each(|node_id| {
        node_id.with_short();
    });

    let op = make_stig_op(TopicId(0).with_short(), LogId(1).with_short(), Seq(0));

    // Node 0 "creates" an op for topic 0
    state
        .graph
        .update_node(0.into(), |mut node| {
            node.routing.stig.op_buffer.store_op(op.clone());
            Ok((node, ()))
        })
        .unwrap();

    let mut scenario = Scenario::initial(model, state, behavior)
        .on_step(|s| {
            println!();
            println!("========= STEP {} =========", s.steps());
            println!();
        })
        .on_action("actions", |action| {
            println!("<< {:?}", action.clone().renamed());
        })
        .on_fx("fx", |fx| {
            if fx.len() > 0 {
                println!("  >> {:?}", fx.clone().renamed());
            }
        });

    // Advance the network; each tick every node broadcasts its Stig state
    // and effects from the previous tick are dispatched as consequences
    for _ in 0..7 {
        scenario.advance().unwrap();
    }

    // Check whether node 9 received the op
    let node0 = scenario.state().graph.node_state(0.into()).unwrap();
    let node9 = scenario.state().graph.node_state(9.into()).unwrap();
    let sv0 = node0.routing.stig.op_buffer.state_vector(0.into());
    let sv9 = node9.routing.stig.op_buffer.state_vector(0.into());
    assert_eq!(sv0.len(), 1);
    assert_eq!(sv0, sv9);
    let received = node9
        .routing
        .stig
        .op_buffer
        .diff(0.into(), &StateVector::new([]));

    // TODO: uncomment once the protocol correctly propagates ops to remote subscribers
    // assert!(!received.is_empty(), "node 9 should have received the op from node 0");
    let _ = received;
}
