_:
    just --list

viewer config="config.json":
    mprocs \
        --names "backend,frontend" \
        "RUST_LOG=info cargo run --bin simdash-runner -- {{config}}" \
        "cd web && npm run dev -- --open"

generate:
    cargo run --bin simdash-runner

test:
    cargo nextest run --workspace
