//! The Network model manages the network-level state:
//! - the state of each Node
//! - the network topology (edges between nodes)

use std::collections::{HashMap, HashSet};

use rand::seq::IndexedRandom;
use simdash_model_node::{NodeAction, NodeModel};
use simdash_types::NodeId;

use crate::{NetworkAction, NetworkFx, NetworkState};

/// The Network model manages the network-level state:
/// - the state of each Node
/// - the network topology (edges between nodes)
///
/// The model holds only transition rules. All data, including the edge specs
/// consulted by `EvolveTopology`, lives in `NetworkState`.
pub struct NetworkModel {
    /// The model of each node
    pub nodes: HashMap<NodeId, NodeModel>,
}

impl NetworkModel {
    /// Constructor
    pub fn new(nodes: HashMap<NodeId, NodeModel>) -> Self {
        Self { nodes }
    }
}

impl polestar::Machine for NetworkModel {
    type State = NetworkState;
    type Action = NetworkAction;
    type Fx = Vec<NetworkFx>;
    type Error = anyhow::Error;

    fn transition(
        &self,
        mut state: Self::State,
        action: Self::Action,
    ) -> polestar::TransitionResult<Self> {
        match action {
            NetworkAction::Tick => {
                for edge in state.graph.graph_mut().edge_weights_mut() {
                    if let Some(ref mut ticks) = edge.remaining_ticks {
                        *ticks = ticks.saturating_sub(1);
                    }
                }

                let all_keys = state.graph.node_ids().collect::<Vec<_>>();

                for node_id in all_keys {
                    let fx = state.graph.update_node(node_id, |node_state| {
                        self.nodes
                            .get(&node_id)
                            .unwrap()
                            .transition(node_state, NodeAction::Tick)
                    })?;
                    debug_assert!(fx.is_empty());
                }
                Ok((state, vec![]))
            }

            NetworkAction::EvolveTopology => {
                let NetworkState { graph, topology } = &mut state;

                graph
                    .graph_mut()
                    .retain_edges(|g, idx| g[idx].remaining_ticks != Some(0));

                let existing_ids: HashSet<u32> = graph
                    .graph()
                    .edge_weights()
                    .map(|e| e.topology_id)
                    .collect();

                let mut rng = topology.rng();
                for edge_topo in topology.unspawned_edges(&existing_ids) {
                    if !edge_topo.should_spawn(&mut rng) {
                        continue;
                    }
                    let candidates = edge_topo.destination_candidates(graph);
                    if let Some(&to) = candidates.choose(&mut rng) {
                        edge_topo.add_to_graph(graph, to);
                    }
                }

                Ok((state, vec![]))
            }

            NetworkAction::Node(node_id, node_action) => {
                match state.graph.update_node(node_id, |node_state| {
                    let (node_state, node_fx) = self
                        .nodes
                        .get(&node_id)
                        .unwrap()
                        .transition(node_state, node_action)?;
                    Ok((node_state, node_fx))
                }) {
                    Ok(fx) => Ok((
                        state,
                        fx.into_iter()
                            .map(|fx| NetworkFx::Node(node_id, fx))
                            .collect(),
                    )),
                    Err(e) => {
                        tracing::error!("Failed to transition node {node_id:?}: {e:?}");
                        Ok((state, vec![]))
                    }
                }
            }

            NetworkAction::Trace(message) => {
                tracing::trace!("{:?}", message);
                Ok((state, vec![]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NetworkTopologyState;
    use crate::net_topology::{
        DestinationSpec, EdgeTopology, NetTopology, NodeTopology, Ordinality, Selection,
    };
    use polestar::Machine;
    use simdash_routing_dispatch::RoutingDispatchModel;

    /// Two nodes, one short-lived edge that always respawns.
    fn make_state() -> NetworkState {
        let mut topo = NetTopology::new();
        topo.nodes.push(NodeTopology::new(0.into()));
        topo.nodes.push(NodeTopology::new(1.into()));
        topo.edges.push(EdgeTopology {
            id: 0,
            from: 0.into(),
            destination: DestinationSpec {
                selection: Selection::Random,
                subnet: None,
            },
            ordinality: Ordinality::Unidirectional,
            longevity: 1,
            spawn_chance: Some(1.0),
        });
        topo.into_network_state(7)
    }

    fn make_model(state: &NetworkState) -> NetworkModel {
        NetworkModel::new(
            state
                .graph
                .node_ids()
                .map(|id| {
                    (
                        id,
                        NodeModel::new(RoutingDispatchModel::standard(id, 1.into())),
                    )
                })
                .collect(),
        )
    }

    #[test]
    fn evolve_topology_respawns_edge_from_state_alone() {
        let state = make_state();
        let model = make_model(&state);
        assert_eq!(state.graph.graph().edge_count(), 1);

        // Tick expires the edge, evolve removes it and respawns it.
        let (state, _) = model.transition(state, NetworkAction::Tick).unwrap();
        let (state, _) = model
            .transition(state, NetworkAction::EvolveTopology)
            .unwrap();
        assert_eq!(state.graph.graph().edge_count(), 1);
        let edge = state.graph.graph().edge_weights().next().unwrap();
        assert_eq!(edge.remaining_ticks, Some(1));
    }

    #[test]
    fn evolve_topology_survives_serialization_round_trip() {
        let state = make_state();
        let bytes = postcard::to_stdvec(&state).unwrap();
        let state: NetworkState = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(state.topology.edges.len(), 1);

        let model = make_model(&state);
        let (state, _) = model.transition(state, NetworkAction::Tick).unwrap();
        let (state, _) = model
            .transition(state, NetworkAction::EvolveTopology)
            .unwrap();
        assert_eq!(state.graph.graph().edge_count(), 1);
    }

    #[test]
    fn state_without_edge_specs_never_spawns() {
        let mut state = make_state();
        state.topology = NetworkTopologyState::new(7, vec![]);
        let model = make_model(&state);
        let (state, _) = model.transition(state, NetworkAction::Tick).unwrap();
        let (state, _) = model
            .transition(state, NetworkAction::EvolveTopology)
            .unwrap();
        assert_eq!(state.graph.graph().edge_count(), 0);
    }
}
