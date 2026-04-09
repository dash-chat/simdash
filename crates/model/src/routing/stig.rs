use std::collections::BTreeSet;

use simdash_ids::{NodeId, OpId};
use simdash_state::{
    chat::{GroupId, Operation},
    routing::flooding::FloodingOp,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StigAction {
    Receive { from: NodeId, msg: StigMessage },
    Send { want: BTreeSet<GroupId> },
    Trigger,
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StigMessage {
    Want { groups: BTreeSet<GroupId> },
    Data { group: GroupId, op: Operation },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct StigState {}

pub enum StigFx {
    Send { to: NodeId, msg: StigMessage },
}

pub struct StigModel {}

impl polestar::Machine for StigModel {
    type State = StigState;
    type Action = StigAction;
    type Fx = StigFx;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        Ok((state, todo!()))
    }
}

#[cfg(test)]
mod tests {
    use simdash_state::{network::NetworkState, node::NodeState};

    use crate::network::NetworkModel;

    use super::*;

    #[test]
    fn test_stig_model() {
        let nodes = vec![NodeState::<StigState>::default(); 10];
        let graph = net_top_gen::scc::strongly_connected_graph(nodes, &mut rand::rng(), 1.5);
        let state: NetworkState<StigState> = graph.into();

        // let model = StigModel::new();
    }
}
