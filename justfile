_:
    just --list

viewer config="config.json":
    cargo run --bin simdash-runner -- {{config}} &
    sleep 2 && cd web && npm run dev -- --open
    kill %1 2>/dev/null || true

generate:
    cargo run --bin simdash-runner

test:
    cargo nextest run --workspace
