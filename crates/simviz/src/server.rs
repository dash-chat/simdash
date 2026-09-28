//! HTTP server exposing a [`Viewer`] to the frontend.
//!
//! - `GET /api/frame` — the current [`Frame`].
//! - `POST /api/command` — apply a [`Command`] and return the new frame.
//! - anything else — the frontend, built from `crates/simviz/web` by `build.rs`
//!   and embedded at compile time, with an SPA fallback to `index.html`.
//!
//! Because the frontend is embedded, a crate that depends on `simviz` can run
//! the whole viz from its own binary with no node toolchain at runtime.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use include_dir::{Dir, include_dir};
use polestar::{Behavior, FxOf, StateOf};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::{Command, Frame, Presenter, Viewer};

type Shared<B, P> = Arc<Mutex<Viewer<B, P>>>;

/// The frontend, baked in at compile time. `SIMVIZ_UI_DIR` is set by `build.rs`.
static UI: Dir<'static> = include_dir!("$SIMVIZ_UI_DIR");

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
        .fallback(get(serve_ui))
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

/// Serve a file from the embedded frontend, falling back to `index.html` so
/// client-side routes and a bare `/` both load the app.
async fn serve_ui(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let file = match UI.get_file(path) {
        Some(file) if !path.is_empty() => file,
        _ => match UI.get_file("index.html") {
            Some(index) => index,
            None => return (StatusCode::NOT_FOUND, "frontend not built").into_response(),
        },
    };
    let mime = mime_guess::from_path(file.path()).first_or_octet_stream();
    (
        [(header::CONTENT_TYPE, mime.as_ref().to_string())],
        file.contents(),
    )
        .into_response()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::toy::{Ring, ring_simulation};
    use crate::{FnPresenter, Topology};
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn app() -> Router {
        let presenter: FnPresenter<Ring, fn(&Ring) -> Topology> = FnPresenter::new(Ring::topology);
        router(Viewer::new(ring_simulation(3), presenter))
    }

    async fn get(path: &str) -> (StatusCode, String, String) {
        let res = app()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = res.status();
        let ctype = res
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        let body = String::from_utf8_lossy(&res.into_body().collect().await.unwrap().to_bytes())
            .to_string();
        (status, ctype, body)
    }

    #[tokio::test]
    async fn root_serves_embedded_index() {
        let (status, ctype, body) = get("/").await;
        assert_eq!(status, StatusCode::OK);
        assert!(ctype.starts_with("text/html"), "{ctype}");
        assert!(body.contains("<html"), "{body}");
    }

    #[tokio::test]
    async fn unknown_paths_fall_back_to_index_for_spa_routing() {
        let (status, ctype, _) = get("/some/client/route").await;
        assert_eq!(status, StatusCode::OK);
        assert!(ctype.starts_with("text/html"));
    }

    fn find_js<'a>(dir: &'a Dir<'a>) -> Option<String> {
        dir.files()
            .find(|f| f.path().extension().is_some_and(|e| e == "js"))
            .map(|f| f.path().to_string_lossy().to_string())
            .or_else(|| dir.dirs().find_map(find_js))
    }

    #[tokio::test]
    async fn app_bundle_is_served_with_js_mime() {
        let js = find_js(UI.get_dir("_app").expect("built frontend has _app/"))
            .expect("some js file under _app/");
        let (status, ctype, _) = get(&format!("/{js}")).await;
        assert_eq!(status, StatusCode::OK);
        assert!(ctype.contains("javascript"), "{ctype}");
    }

    #[tokio::test]
    async fn api_still_answers() {
        let (status, ctype, body) = get("/api/frame").await;
        assert_eq!(status, StatusCode::OK);
        assert!(ctype.starts_with("application/json"));
        assert!(body.contains("\"topology\""));
    }
}
