_:
    just --list

serve:
    cargo run --bin simdash-runner

dev-ui:
    cd web && pnpm dev

generate:
    cargo run --bin simdash-runner

test:
    cargo nextest run --workspace
