# Simdash

A deterministic network simulator for [Dash Chat](https://dashchat.org), a p2p encrypted messenger. Simdash models network topologies, node behavior, and routing algorithms as a pure state machine, enabling reproducible exploration of edge cases in message delivery, convergence, and protocol behavior.

## Getting Started

### Prerequisites

- Rust toolchain (via `nix develop` or manually)
- [cargo-nextest](https://nexte.st/) for running tests
- [just](https://github.com/casey/just) for task running
- Node.js (for the web viewer)

### Commands

```bash
just run        # Run the CLI simulation, uses config.json as default
just resume     # Resume simulation, uses gen/snapshot.json as default
just viewer     # Launch backend + frontend viewer, uses config.json as default
just test       # Run all tests
just generate   # Generate topology from config
```

## Protocol Development

Protocols are developed in their own separate crates. To develop a new protocol, create a new crate in the `protocols` directory, and: 
- add it to the `Cargo.toml` file
- add it to the `routing_dispatch!` macro in `simdash-routing-dispatch`
- hook up a Scenario with a Behavior that uses your protocol
- TODO: what else?

The protocol crate must depend on `simdash-routing-common` and `simdash-types`, as well as `polestar` for state machine types, and can communicate with the rest of the system
through the [`ProtocolAction`] and [`ProtocolFx`] enums.

