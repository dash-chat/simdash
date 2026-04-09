use std::collections::BTreeSet;

use polestar::Machine;
use simdash_action::{
    chat::ChatAction, network::NetworkAction, node::NodeAction, routing::RoutingAction,
};
use simdash_ids::{NodeId, OpId};
use simdash_model::{network::NetworkModel, node::NodeModelConfig, util::basic_op};
use simdash_state::network::NetworkState;

fn holding_count(state: &NetworkState, ops: impl IntoIterator<Item = OpId>) -> usize {
    let ops = ops.into_iter().collect::<Vec<_>>();
    state
        .graph
        .graph()
        .node_weights()
        .filter(|n| ops.iter().any(|op_id| n.dash.op_store.contains_key(&op_id)))
        .count()
}

/// Test that the flooding algorithm converges to the entire (connected) network holding an op
#[test]
fn flooding_convergence() {
    let initial_state = NetworkState::load_postcard("../../fixtures/topology.postcard");
    let model = NetworkModel::new(NodeModelConfig {
        broadcast_interval: 1.into(),
    });

    let ops = (0..5).map(|i| basic_op(i as u32));
    println!(
        "initial count: {:?}",
        holding_count(&initial_state, ops.clone().map(|op| op.id))
    );
    let stores = ops
        .clone()
        .enumerate()
        .map(|(i, op)| {
            NetworkAction::Node(
                NodeId::from(i as u32),
                NodeAction::Chat(ChatAction::StoreOp(op.clone())),
            )
        })
        .collect::<Vec<_>>();

    let broadcast = |node_id: NodeId| {
        NetworkAction::Node(
            node_id,
            NodeAction::Routing(RoutingAction::TriggerBroadcast),
        )
    };

    // store ops on nodes
    let mut state = model.apply_actions_(initial_state, stores).unwrap();

    let mut node_ids = state.graph.node_ids().collect::<Vec<_>>();
    node_ids.sort();
    let broadcasts = node_ids.into_iter().map(broadcast).collect::<Vec<_>>();

    // make everyone broadcast their ops as fast as possible
    let mut prev_count = 0;
    for _ in 0..100 {
        let count = holding_count(&state, ops.clone().map(|op| op.id));
        if prev_count != count {
            prev_count = count;
            println!("count: {}", count);
        } else {
            println!("count is stable");
            break;
        }
        state = model.apply_actions_(state, broadcasts.clone()).unwrap();
        state = model.apply_actions_(state, [NetworkAction::Tick]).unwrap();
    }
}
