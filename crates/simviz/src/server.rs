//! HTTP server exposing a [`Viewer`] to the frontend.
//!
//! - `GET /api/frame` — the current [`Frame`].
//! - `POST /api/command` — apply a [`Command`] and return the new frame.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use polestar::{Behavior, FxOf, StateOf};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::{Command, Frame, Presenter, Viewer};

type Shared<B, P> = Arc<Mutex<Viewer<B, P>>>;

/// Build the router for a viewer.
pub fn router<B, P>(viewer: Viewer<B, P>) -> Router
where
    B: Behavior + Send + 'static,
    FxOf<B::Model>: std::fmt::Debug,
    P: Presenter<State = StateOf<B::Model>>,
{
    let shared: Shared<B, P> = Arc::new(Mutex::new(viewer));
    Router::new()
        .route("/api/frame", get(get_frame::<B, P>))
        .route("/api/command", post(post_command::<B, P>))
        .layer(CorsLayer::permissive())
        .with_state(shared)
}

/// Serve a viewer on `addr` until the process exits.
pub async fn serve<B, P>(viewer: Viewer<B, P>, addr: SocketAddr) -> anyhow::Result<()>
where
    B: Behavior + Send + 'static,
    FxOf<B::Model>: std::fmt::Debug,
    P: Presenter<State = StateOf<B::Model>>,
{
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("simviz listening on http://{addr}");
    axum::serve(listener, router(viewer)).await?;
    Ok(())
}

async fn get_frame<B, P>(State(viewer): State<Shared<B, P>>) -> Json<Frame>
where
    B: Behavior + Send + 'static,
    FxOf<B::Model>: std::fmt::Debug,
    P: Presenter<State = StateOf<B::Model>>,
{
    Json(viewer.lock().await.frame())
}

async fn post_command<B, P>(
    State(viewer): State<Shared<B, P>>,
    Json(command): Json<Command>,
) -> Result<Json<Frame>, AppError>
where
    B: Behavior + Send + 'static,
    FxOf<B::Model>: std::fmt::Debug,
    P: Presenter<State = StateOf<B::Model>>,
{
    let frame = viewer.lock().await.handle(command)?;
    Ok(Json(frame))
}

struct AppError(anyhow::Error);

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
