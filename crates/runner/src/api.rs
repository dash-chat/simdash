use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use simdash_ids::NodeId;
use simdash_state::network::NetworkState;
use tokio::sync::Mutex;

use crate::sim::Simulation;

pub type AppState = Arc<Mutex<Simulation>>;

pub async fn get_state(State(sim): State<AppState>) -> Json<SimSnapshot> {
    let sim = sim.lock().await;
    Json(build_snapshot(&sim))
}

pub async fn step_forward(State(sim): State<AppState>) -> Result<Json<SimSnapshot>, AppError> {
    let mut sim = sim.lock().await;
    sim.step_forward()?;
    Ok(Json(build_snapshot(&sim)))
}

pub async fn step_back(State(sim): State<AppState>) -> Result<Json<SimSnapshot>, AppError> {
    let mut sim = sim.lock().await;
    sim.step_back()?;
    Ok(Json(build_snapshot(&sim)))
}

pub async fn reset(State(sim): State<AppState>) -> Json<SimSnapshot> {
    let mut sim = sim.lock().await;
    sim.reset();
    Json(build_snapshot(&sim))
}

fn build_snapshot(sim: &Simulation) -> SimSnapshot {
    let state = sim.current_state();
    SimSnapshot {
        step: sim.current_step(),
        graph: build_graph_structure(state),
        nodes: build_node_infos(state),
    }
}

fn build_graph_structure(state: &NetworkState) -> GraphStructure {
    let mut index_to_id = std::collections::BTreeMap::new();
    for node_id in state.graph.node_ids() {
        if let Some(idx) = state.graph.node_index(node_id) {
            index_to_id.insert(idx, node_id);
        }
    }

    let nodes: Vec<GraphNode> = index_to_id
        .values()
        .map(|&id| GraphNode { id: *id })
        .collect();

    let edges: Vec<GraphEdge> = state
        .graph
        .graph()
        .edge_indices()
        .filter_map(|e| {
            let (src, dst) = state.graph.graph().edge_endpoints(e)?;
            Some(GraphEdge {
                source: **index_to_id.get(&src)?,
                target: **index_to_id.get(&dst)?,
            })
        })
        .collect();

    GraphStructure { nodes, edges }
}

fn build_node_infos(state: &NetworkState) -> Vec<NodeInfo> {
    let mut node_ids: Vec<NodeId> = state.graph.node_ids().collect();
    node_ids.sort();

    node_ids
        .into_iter()
        .filter_map(|node_id| {
            let node_state = state.graph.node_state(node_id)?;
            let neighbors: Vec<u32> = state
                .graph
                .graph()
                .neighbors(state.graph.node_index(node_id)?)
                .filter_map(|idx| state.graph.node_id(idx))
                .map(|id| *id)
                .collect();

            let ops: Vec<OpInfo> = node_state
                .dash
                .op_store
                .values()
                .map(|op| OpInfo {
                    id: *op.id,
                    author: *op.header.author,
                    payload_size: *op.header.payload_size,
                    body: match &op.body {
                        simdash_state::chat::Body::GroupMessage { group_id } => {
                            OpBody::GroupMessage {
                                group_id: *group_id,
                            }
                        }
                        simdash_state::chat::Body::DirectMessage { recipient } => {
                            OpBody::DirectMessage {
                                recipient: **recipient,
                            }
                        }
                    },
                })
                .collect();

            let routing_ops: Vec<RoutingOpInfo> = node_state
                .routing
                .flooding
                .ops
                .values()
                .map(|op| RoutingOpInfo {
                    id: *op.id,
                    size: *op.metadata.size,
                    since_last_send: op.since_last_send.map(|t| *t),
                })
                .collect();

            Some(NodeInfo {
                id: *node_id,
                ops,
                contacts: node_state.dash.contacts.iter().map(|c| **c).collect(),
                groups: node_state.dash.groups.iter().copied().collect(),
                routing_ops,
                neighbors,
            })
        })
        .collect()
}

// --- Response types ---

#[derive(Serialize)]
pub struct SimSnapshot {
    pub step: usize,
    pub graph: GraphStructure,
    pub nodes: Vec<NodeInfo>,
}

#[derive(Serialize)]
pub struct GraphStructure {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Serialize)]
pub struct GraphNode {
    pub id: u32,
}

#[derive(Serialize)]
pub struct GraphEdge {
    pub source: u32,
    pub target: u32,
}

#[derive(Serialize)]
pub struct NodeInfo {
    pub id: u32,
    pub ops: Vec<OpInfo>,
    pub contacts: Vec<u32>,
    pub groups: Vec<u64>,
    pub routing_ops: Vec<RoutingOpInfo>,
    pub neighbors: Vec<u32>,
}

#[derive(Serialize)]
pub struct OpInfo {
    pub id: u32,
    pub author: u32,
    pub payload_size: u64,
    pub body: OpBody,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum OpBody {
    GroupMessage { group_id: u64 },
    DirectMessage { recipient: u32 },
}

#[derive(Serialize)]
pub struct RoutingOpInfo {
    pub id: u32,
    pub size: u64,
    pub since_last_send: Option<u32>,
}

// --- Error handling ---

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.0.to_string()).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}
