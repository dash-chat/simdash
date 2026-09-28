//! The one thing every visualizable model must expose: a directed graph of nodes.

use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stable identity of a node across simulation steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub u32);

impl From<u32> for NodeId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// A directed edge between two nodes, as it appears on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Edge {
    /// Node the edge leaves from.
    pub source: NodeId,
    /// Node the edge arrives at.
    pub target: NodeId,
}

/// A snapshot of the network's shape at one simulation step: nodes and directed edges.
///
/// Backed by a petgraph graph whose node weights are the stable [`NodeId`]s.
/// Serialises as `{ "nodes": [...], "edges": [{ "source", "target" }, ...] }`
/// with both lists sorted, so identical topologies produce identical frames.
#[derive(Debug, Clone, Default)]
pub struct Topology {
    graph: DiGraph<NodeId, ()>,
    index: HashMap<NodeId, NodeIndex>,
}

impl Topology {
    /// Build a topology from node ids and directed edges.
    ///
    /// Edges whose endpoints are not in `nodes` are dropped.
    pub fn from_parts(
        nodes: impl IntoIterator<Item = NodeId>,
        edges: impl IntoIterator<Item = (NodeId, NodeId)>,
    ) -> Self {
        let mut graph = DiGraph::new();
        let mut index = HashMap::new();
        for id in nodes {
            index.entry(id).or_insert_with(|| graph.add_node(id));
        }
        for (a, b) in edges {
            if let (Some(&ia), Some(&ib)) = (index.get(&a), index.get(&b)) {
                graph.add_edge(ia, ib, ());
            }
        }
        Self { graph, index }
    }

    /// The underlying petgraph graph.
    pub fn graph(&self) -> &DiGraph<NodeId, ()> {
        &self.graph
    }

    /// All node ids, sorted.
    pub fn nodes(&self) -> Vec<NodeId> {
        let mut ids: Vec<NodeId> = self.graph.node_weights().copied().collect();
        ids.sort_unstable();
        ids
    }

    /// All directed edges, sorted.
    pub fn edges(&self) -> Vec<Edge> {
        let mut edges: Vec<Edge> = self
            .graph
            .edge_indices()
            .filter_map(|e| {
                let (a, b) = self.graph.edge_endpoints(e)?;
                Some(Edge {
                    source: self.graph[a],
                    target: self.graph[b],
                })
            })
            .collect();
        edges.sort_unstable();
        edges
    }

    /// Whether the topology contains this node.
    pub fn contains(&self, id: NodeId) -> bool {
        self.index.contains_key(&id)
    }
}

#[derive(Serialize, Deserialize)]
struct TopologyWire {
    nodes: Vec<NodeId>,
    edges: Vec<Edge>,
}

impl Serialize for Topology {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TopologyWire {
            nodes: self.nodes(),
            edges: self.edges(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Topology {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = TopologyWire::deserialize(deserializer)?;
        Ok(Self::from_parts(
            wire.nodes,
            wire.edges.into_iter().map(|e| (e.source, e.target)),
        ))
    }
}

impl PartialEq for Topology {
    fn eq(&self, other: &Self) -> bool {
        self.nodes() == other.nodes() && self.edges() == other.edges()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(id: u32) -> NodeId {
        NodeId(id)
    }

    #[test]
    fn nodes_and_edges_are_sorted_regardless_of_insertion_order() {
        let t = Topology::from_parts([n(2), n(0), n(1)], [(n(2), n(0)), (n(0), n(1))]);
        assert_eq!(t.nodes(), vec![n(0), n(1), n(2)]);
        assert_eq!(
            t.edges(),
            vec![
                Edge { source: n(0), target: n(1) },
                Edge { source: n(2), target: n(0) }
            ]
        );
    }

    #[test]
    fn edges_to_unknown_nodes_are_dropped() {
        let t = Topology::from_parts([n(0)], [(n(0), n(9))]);
        assert!(t.edges().is_empty());
        assert!(t.contains(n(0)));
        assert!(!t.contains(n(9)));
    }

    #[test]
    fn serializes_to_plain_json_shape() {
        let t = Topology::from_parts([n(1), n(0)], [(n(1), n(0))]);
        let json = serde_json::to_value(&t).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "nodes": [0, 1],
                "edges": [{ "source": 1, "target": 0 }]
            })
        );
        let back: Topology = serde_json::from_value(json).unwrap();
        assert_eq!(back, t);
    }
}
