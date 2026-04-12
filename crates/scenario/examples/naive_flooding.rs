use dash_chat_top_gen::ChatTopology;
use polestar::Machine;
use simdash_action::{chat::ChatAction, network::NetworkAction, node::NodeAction};
use simdash_behavior::{
    network::RoundRobinNodeScheduler, node::BasicNodeBehavior, routing::BasicRoutingBehavior,
};
use simdash_ids::{NodeId, OpId};
use simdash_model::{network::NetworkModel, node::NodeModel};
use simdash_routing::RoutingModel;
use simdash_scenario::Scenario;
use simdash_state::network::NetworkState;

const STABLE_STEP_LIMIT: usize = 100;

fn holding_count(state: &NetworkState, ops: impl IntoIterator<Item = OpId>) -> usize {
    let ops = ops.into_iter().collect::<Vec<_>>();
    state
        .graph
        .node_ids()
        .filter_map(|id| state.graph.node_state(id))
        .filter(|n| ops.iter().any(|op_id| n.chat.op_store.contains_key(&op_id)))
        .count()
}

fn main() {
    let initial_state = NetworkState::load_postcard("gen/topology.postcard");
    let model = NetworkModel::new(
        NodeModel::new(RoutingModel::standard(3.into())),
        ChatTopology::empty(),
    );

    let ops = (0..5).map(|i| simdash_ids::util::basic_op(i as u32));
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

    let behavior = RoundRobinNodeScheduler::new(BasicNodeBehavior::new(BasicRoutingBehavior));
    let mut scenario = Scenario::initial(model, initial_state, behavior);

    let mut prev_count = 0;
    let mut stable_steps = 0;
    let start_time = std::time::Instant::now();
    println!("starting scenario");
    let mut step = 0;

    println!(" STEP | COUNT | ACTIONS  | TIME");
    println!("------|-------|----------|----------");
    loop {
        let advanced = scenario.advance().unwrap();
        if !advanced {
            println!("scenario couldn't advance");
            break;
        };
        step += 1;
        let state = scenario.state().unwrap();
        let count = holding_count(&state, ops.clone().map(|op| op.id));
        let num_actions = scenario.history().len();
        if num_actions % state.graph.graph().node_count() == 0 {
            println!("...");
        }
        if prev_count != count {
            stable_steps = 0;
            prev_count = count;
            println!(
                "{:<5} | {:<5} | {:<8} | {:.2?}",
                step,
                count,
                num_actions,
                start_time.elapsed()
            );
        } else {
            stable_steps += 1;
            if stable_steps > STABLE_STEP_LIMIT {
                println!(
                    "count is stable in {} steps, {num_actions} actions. avg time per step: {:.2?}",
                    step - stable_steps,
                    start_time.elapsed() / step as u32
                );
                break;
            }
        }
    }
}
