_:
    just --list

viewer config="config.json":
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill 0' EXIT
    cargo run --bin simdash-runner -- {{config}} &
    sleep 2
    cd web && npm run dev -- --open

generate:
    cargo run --bin simdash-runner

test:
    cargo nextest run --workspace
