_:
    just --list

generate:
    cargo run --bin simdash-runner

test:
    cargo nextest run --workspace
