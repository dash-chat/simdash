use polestar::Machine;
use simdash_action::{chat::ChatAction, network::NetworkAction, node::NodeAction};
use simdash_behavior::{
    network::RoundRobinNodeScheduler, node::BasicRoutingBehavior, routing::NaiveFloodingBehavior,
};
use simdash_ids::{NodeId, OpId};
use simdash_model::{network::NetworkModel, node::NodeModelConfig, util::basic_op};
use simdash_scenario::Scenario;
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

fn main() {
    let initial_state = NetworkState::load_postcard("gen/topology.postcard");
    let model = NetworkModel::new(NodeModelConfig {
        broadcast_interval: 3.into(),
    });

    let ops = (0..5).map(|i| basic_op(i as u32));
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

    // store ops on nodes
    let initial_state = model.apply_actions_(initial_state, stores).unwrap();

    let behavior = RoundRobinNodeScheduler::new(BasicRoutingBehavior::new(NaiveFloodingBehavior));
    let mut scenario = Scenario::initial(model, initial_state, behavior);

    let mut prev_count = 0;
    let mut stable_steps = 0;
    let start_time = std::time::Instant::now();

    loop {
        scenario.advance().unwrap();
        let state = scenario.state().unwrap();
        let count = holding_count(&state, ops.clone().map(|op| op.id));
        if prev_count != count {
            prev_count = count;
            println!(
                "step {:<5} | count={:<3} | time={:.2?}",
                scenario.history().len(),
                count,
                start_time.elapsed()
            );
        } else {
            stable_steps += 1;
            if stable_steps > 10000 {
                println!(
                    "count is stable. avg time per step: {:.2?}",
                    start_time.elapsed() / scenario.history().len() as u32
                );
                break;
            }
        }
    }
}
