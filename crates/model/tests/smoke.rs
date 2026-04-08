use std::collections::BTreeSet;

use polestar::Machine;
use simdash_action::{
    dash::DashAction, network::NetworkAction, node::NodeAction, routing::RoutingAction,
};
use simdash_ids::{NodeId, OpId};
use simdash_model::{network::NetworkModel, node::NodeModelConfig, util::basic_op};
use simdash_state::network::NetworkState;

fn holding_count(state: &NetworkState, op_id: &OpId) -> usize {
    state
        .graph
        .graph()
        .node_weights()
        .filter(|n| n.dash.op_store.contains_key(op_id))
        .count()
}

#[test]
fn flooding_convergence() {
    let initial_state = NetworkState::load_postcard("../../fixtures/topology.postcard");
    let model = NetworkModel::new(NodeModelConfig {
        broadcast_interval: 1.into(),
    });

    let op0 = basic_op(0);
    let store0 = NetworkAction::Node(
        NodeId::from(0),
        NodeAction::Dash(DashAction::StoreOp(op0.clone())),
    );

    let broadcast = |node_id: NodeId| {
        NetworkAction::Node(
            node_id,
            NodeAction::Routing(RoutingAction::TriggerBroadcast),
        )
    };

    // store one op on node 0
    let mut state = model.apply_actions_(initial_state, [store0]).unwrap();

    let mut node_ids = state.graph.node_ids().collect::<Vec<_>>();
    node_ids.sort();
    let broadcasts = node_ids.into_iter().map(broadcast).collect::<Vec<_>>();

    // make everyone broadcast their ops as fast as possible
    let mut prev_count = 0;
    for _ in 0..1000 {
        state = model.apply_actions_(state, broadcasts.clone()).unwrap();
        state = model.apply_actions_(state, [NetworkAction::Tick]).unwrap();
        let count = holding_count(&state, &op0.id);
        if prev_count != count {
            prev_count = count;
            println!("count: {}", count);
        } else {
            println!("count is stable");
            break;
        }
    }
}
