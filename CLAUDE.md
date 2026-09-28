# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Simdash is a deterministic network simulator for [Dash Chat](https://dashchat.org), a p2p encrypted messenger. It models network topologies, node behavior, and routing algorithms as a pure state machine, enabling reproducible exploration of edge cases in message delivery, convergence, and protocol behavior.

## Commands

```bash
# Run all tests
just test

# Generate topology from config.json (outputs to gen/)
just generate
```

Under the hood: `just test` runs `cargo nextest run --workspace`, `just generate` runs `cargo run --bin simdash-runner`.

## Architecture

### Polestar State Machine Pattern

The core architecture uses [polestar](https://github.com/maackle/polestar-rs), a state machine framework. Every model implements `polestar::Machine` with four associated types:

- `State` — the data being transformed
- `Action` — an input that triggers a transition
- `Fx` — side effects produced by a transition (propagated up to the network level)
- `Error`

Transitions are pure: `fn transition(state, action) -> Result<(state, fx)>`. Side effects (like broadcasting ops to neighbors) are applied by the parent model after the transition returns.

### Crate Dependency Layers

```
runner
  |
  +-- net-top-gen, dash-chat-top-gen   (scenario generators)
  |       |
  +-------+-- state, ids, distribution (core data types)

model
  +-- action
  +-- state
  +-- ids
```

**`ids`** — Newtype wrappers: `NodeId`, `OpId`, `Size`, `Ticks`.

**`distribution`** — Configurable random distributions (Fixed/Uniform/Normal) used by generators. Serializable as JSON with `{"type": "normal", "mean": ..., "std_dev": ...}`.

**`state`** — Pure data structures with no transition logic:
- `Graph<R, E>` — petgraph wrapper mapping `NodeId` to `NodeState`, with `update_node` for functional updates.
- `NodeState<R>` — contains `ChatState` (op_store, contacts, groups) and routing state `R` (generic, defaults to `FloodingState`).
- `NetworkState` — wraps `Graph`, provides `neighborhood()` for graph traversal.

**`action`** — Enum hierarchies mirroring the state structure:
- `NetworkAction::Tick | Node(NodeId, NodeAction)`
- `NodeAction::Chat(ChatAction) | Routing(RoutingAction)`
- `ChatAction::StoreOp | AddContact | JoinGroup`
- `FloodingAction::StoreOp | ReceiveOp | TriggerBroadcast`

**`model`** — Polestar `Machine` implementations composing the state machines:
- `NetworkModel` > `NodeModel` > `ChatModel` + `RoutingDispatchModel` (alias for `FloodingModel`)
- `NetworkModel::transition` dispatches actions, then applies effects (`FloodingFx::BroadcastOps` sends ops to all neighbors, `FloodingFx::StoreOp` persists received ops).
- Routing is generic via type aliases (`RoutingDispatchModel = FloodingModel`, `RoutingDispatchState = FloodingState`), so new protocols can be swapped in.

**`net-top-gen`** — Generates `Graph` from a JSON rule list. Rules are applied sequentially: `NodeSpec` adds nodes, `LinkSpec` adds edges (per-node with random/fixed counts).

**`dash-chat-top-gen`** — Generates per-node `ChatState` (contacts, groups, op_store) from distribution-based config. Builds on top of an existing `Graph`.

**`runner`** — CLI that reads `config.json`, runs both generators, serializes the graph to `gen/topology.postcard` and `gen/topology.dot`.

### Key Patterns

- **Determinism**: All randomness flows from a seed. Same seed + config = identical output.
- **Effect propagation**: Child machines produce `Fx` values; parent machines interpret and apply them. `FloodingFx::BroadcastOps` triggers `send_op` to each neighbor. `FloodingFx::StoreOp` writes into the node's `ChatState`.
- **Tick-based time**: `Ticks` is the time unit. `NetworkAction::Tick` advances all nodes. Flooding uses `since_last_send` with a `broadcast_interval` to rate-limit rebroadcasts.
- **Serialization**: State uses `postcard` (binary) for fixtures and `serde_json` for config. The `fixtures/` directory contains a pre-generated topology used by integration tests.

### simviz: the generic visualization seam (parallel track)

`crates/simviz` is a model-agnostic viewer library: a `Simulation<B: Behavior>` (polestar `Scenario` with step forward/back/reset via replay), a `Topology` (petgraph digraph of `NodeId`s), and a `Presenter` trait that is the only place a model touches the viz. `Presenter::topology` is required; `on_action`/`render` default to noop. `render` returns `VizEvent`s (node/edge styles, sidebar fields, links that send `UiAction`s back). `Viewer` pairs a simulation with a presenter and serves `GET /api/frame` and `POST /api/command`.

`crates/simviz-simdash` is the simdash adapter: `SimdashPresenter` (topology only for now) plus a CLI on port 3001 (`just serve`), which can also write a static recording (`just record`). `crates/simviz/web/` is the generic SvelteKit frontend; it interprets events in `src/lib/decorations.ts`, gets frames through the `Backend` interface in `src/lib/backend.ts` (live HTTP, or a recording via `?recording=<url>`), and knows nothing about ops, groups or routing. simviz's `build.rs` runs `npm ci`/`npm run build` and embeds the result, so building simviz needs node; set `SIMVIZ_UI_DIR` to a prebuilt frontend to skip that. `just viewer` runs the Vite dev server for frontend work, `just test-ui` runs its tests. Design notes: `docs/superpowers/specs/2026-09-28-simviz-seam-design.md`. The older `web-old/` + `simdash-simulation::playback` path is unchanged (`just viewer-old`).

### Config Format

`config.json` at the repo root:
- `seed`: u64 for deterministic generation
- `net-top`: array of node/link rules
- `dash-chat-top`: distribution-based chat state config (contacts_per_node, groups_per_node, group_size, ops_per_node, op_payload_size)
