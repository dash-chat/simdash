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

# Serve the simviz viewer: backend plus the frontend it embeds, on :3001.
serve config="config.yaml":
    RUST_LOG=info cargo run --release -p simviz-simdash -- {{ config }}

# Frontend development: backend on :3001, Vite dev server on :5173 proxying /api to it.
viewer config="config.yaml":
    mprocs \
        --names "backend,frontend" \
        "RUST_LOG=info cargo run --release -p simviz-simdash -- {{ config }}" \
        "cd crates/simviz/web && npm run dev -- --open"

# Record `steps` frames to a JSON file for static replay (open the frontend with ?recording=<url>).
record steps="100" out="recording.json" config="config.yaml":
    cargo run --release -p simviz-simdash -- {{ config }} --record {{ steps }} --out {{ out }}

# Frontend unit tests and type check.
test-ui:
    cd crates/simviz/web && npm test && npm run check

# Previous viewer stack: backend on :3000, Vite dev frontend on :5174.
viewer-old config="config.yaml":
    mprocs \
        --names "backend,frontend" \
        "RUST_LOG=info cargo run --release -p simdash-server-runner -- {{ config }}" \
        "cd web-old && npm run dev -- --open"
