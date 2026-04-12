use std::collections::HashSet;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use simdash_ids::{Body, NodeId, Operation};
use simdash_state::network::NetworkState;
use tokio::sync::Mutex;

use simdash_simulation::sim::{OpSnapshot, Simulation};

pub type AppState = Arc<Mutex<Simulation>>;

pub async fn get_state(State(sim): State<AppState>) -> Json<SimResponse> {
    let mut sim = sim.lock().await;
    Json(build_full_response(&mut sim))
}

pub async fn step_forward(State(sim): State<AppState>) -> Result<Json<SimResponse>, AppError> {
    let mut sim = sim.lock().await;
    let prev_seq = sim.seq();
    let prev_ops = sim.snapshot_ops();
    let prev_edges = snapshot_edges(sim.current_state());
    let t = std::time::Instant::now();
    sim.step_forward()?;
    tracing::info!("{:>6}ms", t.elapsed().as_millis());
    let seq = sim.bump_seq();
    Ok(Json(build_delta_response(
        &sim,
        prev_seq,
        seq,
        &prev_ops,
        &prev_edges,
    )))
}

pub async fn step_back(State(sim): State<AppState>) -> Result<Json<SimResponse>, AppError> {
    let mut sim = sim.lock().await;
    sim.step_back()?;
    Ok(Json(build_full_response(&mut sim)))
}

pub async fn reset(State(sim): State<AppState>) -> Json<SimResponse> {
    let mut sim = sim.lock().await;
    sim.reset();
    Json(build_full_response(&mut sim))
}

pub async fn get_behaviors(State(sim): State<AppState>) -> Json<BehaviorsResponse> {
    let sim = sim.lock().await;
    Json(BehaviorsResponse {
        current: sim.behavior_name().to_string(),
        available: sim
            .available_behaviors()
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
}

pub async fn set_behavior(
    State(sim): State<AppState>,
    Json(body): Json<SetBehaviorRequest>,
) -> Result<Json<SimResponse>, AppError> {
    let mut sim = sim.lock().await;
    sim.set_behavior(&body.behavior)?;
    Ok(Json(build_full_response(&mut sim)))
}

// --- Response building ---

fn build_full_response(sim: &mut Simulation) -> SimResponse {
    let seq = sim.bump_seq();
    let state = sim.current_state();
    SimResponse::Full(SimSnapshot {
        seq,
        step: sim.current_step(),
        behavior: sim.behavior_name().to_string(),
        available_behaviors: sim
            .available_behaviors()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        graph: build_graph_structure(state),
        nodes: build_node_infos(state),
    })
}

fn snapshot_edges(state: &NetworkState) -> HashSet<(u32, u32)> {
    state
        .graph
        .graph()
        .edge_indices()
        .filter_map(|e| {
            let (src, dst) = state.graph.graph().edge_endpoints(e)?;
            Some((src.index() as u32, dst.index() as u32))
        })
        .collect()
}

fn build_delta_response(
    sim: &Simulation,
    prev_seq: u64,
    seq: u64,
    prev_ops: &OpSnapshot,
    prev_edges: &HashSet<(u32, u32)>,
) -> SimResponse {
    let state = sim.current_state();
    let mut node_deltas = Vec::new();

    for node_id in state.graph.node_ids() {
        let Some(node_state) = state.graph.node_state(node_id) else {
            continue;
        };

        let current_op_ids: HashSet<u32> = node_state.chat.op_store.keys().map(|id| **id).collect();
        let empty = HashSet::new();
        let prev_op_ids = prev_ops.get(&*node_id).unwrap_or(&empty);

        let ops_added: Vec<OpInfo> = current_op_ids
            .difference(prev_op_ids)
            .filter_map(|&id| {
                node_state
                    .chat
                    .op_store
                    .get(&id.into())
                    .map(|op| build_op_info(op))
            })
            .collect();

        let ops_removed: Vec<u32> = prev_op_ids.difference(&current_op_ids).copied().collect();

        // Skip nodes with no changes
        if ops_added.is_empty() && ops_removed.is_empty() {
            continue;
        }

        let routing_ops: Vec<RoutingOpInfo> = node_state
            .routing
            .flooding
            .ops
            .values()
            .map(|op| RoutingOpInfo {
                id: *op.id(),
                size: *op.data.metadata.size,
                since_last_send: op.state().since_last_send.map(|t| *t),
            })
            .collect();

        node_deltas.push(NodeDelta {
            id: *node_id,
            ops_added,
            ops_removed,
            routing_ops,
            stored_size: *node_state.chat.stored_size(),
        });
    }

    let current_edges = snapshot_edges(state);

    let edges_added: Vec<GraphEdge> = current_edges
        .difference(prev_edges)
        .map(|&(s, t)| GraphEdge {
            source: s,
            target: t,
        })
        .collect();

    let edges_removed: Vec<GraphEdge> = prev_edges
        .difference(&current_edges)
        .map(|&(s, t)| GraphEdge {
            source: s,
            target: t,
        })
        .collect();

    SimResponse::Delta(SimDelta {
        seq,
        prev_seq,
        step: sim.current_step(),
        node_deltas,
        edges_added,
        edges_removed,
    })
}

fn build_graph_structure(state: &NetworkState) -> GraphStructure {
    let nodes: Vec<GraphNode> = state
        .graph
        .node_ids()
        .map(|id| GraphNode { id: *id })
        .collect();

    let edges: Vec<GraphEdge> = state
        .graph
        .graph()
        .edge_indices()
        .filter_map(|e| {
            let (src, dst) = state.graph.graph().edge_endpoints(e)?;
            Some(GraphEdge {
                source: src.index() as u32,
                target: dst.index() as u32,
            })
        })
        .collect();

    let subnets: std::collections::HashMap<String, Vec<u32>> = state
        .graph
        .subnets()
        .iter()
        .map(|(name, ids)| (name.clone(), ids.iter().map(|id| **id).collect()))
        .collect();

    GraphStructure {
        nodes,
        edges,
        subnets,
    }
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
                .neighbors(node_id.into())
                .map(|id| *NodeId::from(id))
                .collect();

            let ops: Vec<OpInfo> = node_state
                .chat
                .op_store
                .values()
                .map(|op| build_op_info(op))
                .collect();

            let routing_ops: Vec<RoutingOpInfo> = node_state
                .routing
                .flooding
                .ops
                .values()
                .map(|op| RoutingOpInfo {
                    id: *op.data.id,
                    size: *op.data.metadata.size,
                    since_last_send: op.state.since_last_send.map(|t| *t),
                })
                .collect();

            Some(NodeInfo {
                id: *node_id,
                ops,
                contacts: node_state.chat.contacts.iter().map(|c| **c).collect(),
                groups: node_state.chat.groups.iter().copied().collect(),
                routing_ops,
                neighbors,
                storage_capacity: node_state.chat.storage_capacity.map(|s| *s),
                stored_size: *node_state.chat.stored_size(),
            })
        })
        .collect()
}

fn build_op_info(op: &Operation) -> OpInfo {
    OpInfo {
        id: *op.id,
        author: *op.header.author,
        payload_size: *op.header.payload_size,
        body: match &op.body {
            Body::GroupMessage { group_id } => OpBody::GroupMessage {
                group_id: *group_id,
            },
            Body::DirectMessage { recipient } => OpBody::DirectMessage {
                recipient: **recipient,
            },
        },
    }
}

// --- Response types ---

#[derive(Serialize)]
#[serde(tag = "kind")]
pub enum SimResponse {
    Full(SimSnapshot),
    Delta(SimDelta),
}

#[derive(Serialize)]
pub struct SimSnapshot {
    pub seq: u64,
    pub step: usize,
    pub behavior: String,
    pub available_behaviors: Vec<String>,
    pub graph: GraphStructure,
    pub nodes: Vec<NodeInfo>,
}

#[derive(Serialize)]
pub struct SimDelta {
    pub seq: u64,
    pub prev_seq: u64,
    pub step: usize,
    pub node_deltas: Vec<NodeDelta>,
    pub edges_added: Vec<GraphEdge>,
    pub edges_removed: Vec<GraphEdge>,
}

#[derive(Serialize)]
pub struct NodeDelta {
    pub id: u32,
    pub ops_added: Vec<OpInfo>,
    pub ops_removed: Vec<u32>,
    pub routing_ops: Vec<RoutingOpInfo>,
    pub stored_size: u64,
}

#[derive(Serialize)]
pub struct BehaviorsResponse {
    pub current: String,
    pub available: Vec<String>,
}

#[derive(Deserialize)]
pub struct SetBehaviorRequest {
    pub behavior: String,
}

#[derive(Serialize)]
pub struct GraphStructure {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub subnets: std::collections::HashMap<String, Vec<u32>>,
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
    pub storage_capacity: Option<u64>,
    pub stored_size: u64,
}

#[derive(Serialize, Clone)]
pub struct OpInfo {
    pub id: u32,
    pub author: u32,
    pub payload_size: u64,
    pub body: OpBody,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum OpBody {
    GroupMessage { group_id: u64 },
    DirectMessage { recipient: u32 },
}

#[derive(Serialize, Clone)]
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
