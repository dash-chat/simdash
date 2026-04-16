_:
    just --list

generate config="config.local.yaml":
    cargo run --release --bin simdash-server-runner -- --no-serve {{ config }}

test:
    cargo nextest run --workspace

run config="config.yaml":
    cargo run --release -- {{ config }}

resume snapshot="gen/snapshot.json":
    cargo run --release -- {{ snapshot }}

viewer config="config.yaml":
    mprocs \
        --names "backend,frontend" \
        "RUST_LOG=info cargo run --release -p simdash-server-runner -- {{ config }}" \
        "cd web && npm run dev -- --open"
