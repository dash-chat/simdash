# simviz: a generic seam for visualizing replayable network simulations

**Status:** implemented as a parallel track (`crates/simviz` with its frontend in `crates/simviz/web`, `crates/simviz-simdash`; the previous frontend is `web-old/`).
The existing `web/` + `simdash-simulation` playback path is untouched.

## Problem

The current viewer is welded to the simdash model. The playback server reads
`NetworkState` fields directly (op stores, contacts, groups, routing ops, storage),
computes deltas over them, and the Svelte frontend hard-codes node colouring,
op tracing, group highlighting and the sidebar layout against those fields.
Bringing in a different model (dash-router) means rewriting all of it.

## Goal

Cut a seam so that:

- The **viz** (Rust server + browser frontend) knows only that there is a replayable
  simulation of a changing directed graph of nodes. It knows nothing about what a node's
  state looks like.
- Each **model** supplies one small module (a `Presenter`) that says how it should be shown.
  It maps model state to a `Topology` and, later, to decoration events (node colours,
  edge styles, sidebar fields with clickable links).
- To start, the presenter for simdash only extracts the topology. Colouring and node info
  are empty. Reproducing today's simdash UI later means only writing a richer presenter.

Out of scope now: per-op delivery views, switching between behaviours at runtime,
deltas on the wire, porting `web/` to the new architecture.

## Shape

```
crates/simviz            generic library: Topology, Simulation, Presenter, events, HTTP server
crates/simviz-simdash    adapter: Presenter for simdash's NetworkState + CLI that serves it
crates/simviz/web        generic SvelteKit frontend (cytoscape): renders frames, fires commands
```

`simviz` depends on polestar/polestar-sim, petgraph, serde, axum. It does not depend on any
simdash crate. `simviz-simdash` depends on both and is the only place the two meet.

## Core types (`simviz`)

### Topology

```rust
pub struct NodeId(pub u32);              // stable identity across steps
pub struct Topology { DiGraph<NodeId, ()> }
impl Topology {
    fn from_parts(nodes: impl IntoIterator<Item = NodeId>, edges: impl IntoIterator<Item = (NodeId, NodeId)>) -> Self
    fn nodes(&self) -> Vec<NodeId>; fn edges(&self) -> Vec<(NodeId, NodeId)>
    fn graph(&self) -> &DiGraph<NodeId, ()>
}
```

Serialises as `{ "nodes": [0, 1, ...], "edges": [{ "source": 0, "target": 1 }, ...] }`.
Node order and edge order are sorted so frames are deterministic.

### Simulation

```rust
pub struct Simulation<B: Behavior> { scenario: polestar_sim::Scenario<B> }
```

Thin wrapper over `Scenario` giving `state()`, `step()`, `step_forward()`, `step_back()`,
`jump_to(n)`, `reset()`. Stepping back is reset + replay, relying on determinism of the
Behavior. There is no history buffer.

### Presenter — the seam

```rust
pub trait Presenter: Send + Sync + 'static {
    type State;
    fn topology(&self, state: &Self::State) -> Topology;                  // required
    fn on_action(&mut self, state: &Self::State, action: &UiAction) {}     // default noop
    fn render(&self, state: &Self::State) -> Vec<VizEvent> { vec![] }      // default noop
}
```

The presenter is the per-model UI module. It owns any UI-side selection state
(which node/op/group is selected) and may keep whatever caches it wants. `render` is
called after every command and returns the full decoration set for the current state;
the frontend clears decorations and re-applies. Deltas can come later without changing
the trait.

`FnPresenter::new(|state| topology)` builds a presenter from a closure for tests and
minimal adapters.

### Commands and events (the wire)

```rust
pub enum Command { StepForward, StepBack, Reset, Ui(UiAction) }
pub enum UiAction { SelectNode(Option<NodeId>), Link(serde_json::Value) }

pub enum VizEvent {
    NodeStyle { node: NodeId, style: NodeStyle },          // color, border_color, opacity, label
    EdgeStyle { source, target, style: EdgeStyle },        // color, width, opacity
    Sidebar { title: Option<String>, fields: Vec<Field> },
}
pub struct Field { label: String, value: FieldValue }
pub enum FieldValue { Text(String), Link { text: String, action: UiAction }, List(Vec<FieldValue>) }

pub struct Frame { step: usize, topology: Topology, selected: Option<NodeId>, events: Vec<VizEvent> }
```

`Link` carries an opaque JSON payload the presenter defined; when clicked the frontend
sends `Command::Ui(UiAction::Link(payload))` back, the presenter's `on_action` interprets
it, and the next `render` reflects whatever changed. This is how "links that fire other
events" works without the viz knowing what a link means.

### Viewer and server

```rust
pub struct Viewer<B, P> { sim: Simulation<B>, presenter: P, selected: Option<NodeId> }
impl Viewer { fn frame(&self) -> Frame; fn handle(&mut self, Command) -> Result<Frame> }
pub async fn serve<B, P>(viewer: Viewer<B, P>, addr: SocketAddr)
```

HTTP: `GET /api/frame` returns the current frame; `POST /api/command` with a `Command`
body applies it and returns the new frame. Every response is a full frame.
`Viewer` tracks the selected node so a presenter with a noop `on_action` still
gets a generic "Node N" sidebar from the frontend.

## Adapter (`simviz-simdash`)

`SimdashPresenter` implements `Presenter<State = NetworkState>` with `topology` only:
node ids from `graph.node_ids()`, edges from the petgraph edge list. `main.rs` reads a
config YAML/JSON (same format as today), builds the standard ecco scenario via the
existing public generator/simulation functions, and serves on port 3001.

## Frontend (`crates/simviz/web/`)

Same stack as the old `web/` (SvelteKit, cytoscape + fcose), one route:

- `backend.ts` — the `Backend` interface (`frame()`, `send(command)`) with two
  implementations: `HttpBackend` for a live server and `RecordedBackend` for static replay.
  `backendFromLocation` picks one: `?recording=<url>` replays, otherwise HTTP.
- `decorations.ts` — pure `applyEvents(events)` producing node/edge style maps and sidebar
  content. This is the only place events are interpreted.
- `GraphView.svelte` — draws the topology, diffs nodes/edges between frames, applies node
  and edge styles from the decorations, reports taps as `SelectNode`.
- `Sidebar.svelte` — shows the selected node title and presenter fields; link fields
  send `Link` commands.
- `Controls.svelte` — step back/forward, play/pause with speed, reset.

The frontend has no knowledge of ops, groups, contacts, routing or storage.

## Using simviz from another project

The intended shape for a new model is one crate in that model's own repo, e.g.
`dash-router-simviz`, that:

1. depends on `simviz` (and on `polestar` / `polestar-sim`, which simviz re-uses);
2. defines the `Behavior` that drives the model and a `Presenter` for its state;
3. has a `main` that builds a `Scenario`, wraps it in `Viewer`, and calls `simviz::serve`.

Nothing from simdash is needed. simviz's `build.rs` builds the frontend from
`crates/simviz/web` (`npm ci` when the lockfile is newer than `node_modules`, then
`npm run build` into cargo's `OUT_DIR`) and embeds the result, so the consumer's binary
serves the whole UI itself with no node at runtime. Building simviz does need node and npm;
`SIMVIZ_UI_DIR=<dir>` points at a prebuilt frontend and skips that. The Vite dev server is
only for developing the frontend.

## Static recordings

`Viewer::record(steps)` resets, captures a `Frame` for every step from 0 to `steps`, and
resets again. The result serialises as `{ "frames": [...] }`. `simviz-simdash --record N
--out file.json` writes one. Host the JSON next to the built frontend and open
`index.html?recording=file.json`: `RecordedBackend` steps through the frames, clamping at
both ends, tracks node selection locally, and ignores presenter links since there is no
presenter to receive them. This is the "weak" hosted mode; the wasm and multi-session server
modes discussed for the "strong" mode would be further `Backend` implementations.

Until simviz lives in its own repo, consume it as a git dependency on this repo; cargo
finds the `simviz` package inside the workspace:

```toml
[dependencies]
simviz = { git = "https://github.com/<org>/simdash.git" }
polestar = { git = "https://github.com/maackle/polestar-rs.git", branch = "main" }
polestar-sim = { git = "https://github.com/maackle/polestar-rs.git", branch = "main" }

# Same local-development patch simdash itself uses, while polestar is unpublished.
[patch."https://github.com/maackle/polestar-rs.git"]
polestar = { path = "/path/to/polestar-rs/polestar" }
polestar-sim = { path = "/path/to/polestar-rs/polestar-sim" }
```

If simviz outgrows simdash, moving `crates/simviz` and `web/` into their own repository
changes only that git URL. A minimal external consumer (own machine, behavior, presenter,
`serve`) was built and run against a path dependency to confirm this works.

## Testing

- `simviz` unit tests: topology serialisation and ordering; simulation forward/back/reset
  replay on a toy "ring growth" machine; viewer frame contents and command handling,
  including selection tracking and presenter `on_action` reaching the presenter.
- `simviz-simdash`: a test that the adapter's topology matches the network graph for a
  generated config.
- `simviz` recording tests: frame count and step numbering, reset before and after, JSON round trip.
- frontend: vitest tests for `RecordedBackend` (clamping, local selection, immutability) and
  `svelte-check`, both via `just test-ui`; manual run via `just viewer`.
