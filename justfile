_:
    just --list

viewer config="config.json":
    mprocs \
        --names "backend,frontend" \
        "RUST_LOG=info cargo run --release --bin simdash-runner -- {{ config }}" \
        "cd web && npm run dev -- --open"

generate config="config.local.json":
    cargo run --release --bin simdash-runner -- --no-serve {{ config }}

test:
    cargo nextest run --workspace
