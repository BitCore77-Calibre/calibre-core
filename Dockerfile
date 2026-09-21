# ─── Stage 1: Rust builder (node + pq-signer) ─────────────────────────
FROM rust:1.85-bookworm AS rust-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential clang libclang-dev protobuf-compiler pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add wasm32-unknown-unknown

WORKDIR /build
COPY . .
RUN cargo build --release -p solochain-template-node -p pq-signer \
    && strip target/release/solochain-template-node target/release/pq-signer

# ─── Stage 2: Node.js deps for the faucet's signer helper ─────────────
FROM node:20-bookworm-slim AS node-builder
WORKDIR /build
COPY tools/faucet/package*.json ./
RUN npm ci --omit=dev

# ─── Stage 3: Runtime image ───────────────────────────────────────────
FROM node:20-bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl python3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /build/target/release/solochain-template-node /usr/local/bin/calibre-node
COPY --from=rust-builder /build/target/release/pq-signer /usr/local/bin/pq-signer
COPY --from=node-builder /build/node_modules /app/tools/faucet/node_modules
COPY tools/faucet/faucet.py /app/tools/faucet/faucet.py
COPY tools/faucet/sign_mint.js /app/tools/faucet/sign_mint.js
COPY tools/faucet/package.json /app/tools/faucet/package.json
COPY chain-specs/calibre-testnet-raw.json /app/chain-specs/

WORKDIR /app
RUN mkdir -p /data
VOLUME /data

EXPOSE 9944 30333 9615 8090

# entrypoint: generate node key on first run, then launch the validator
ENTRYPOINT ["sh", "-c", "\
  test -f /data/chains/calibre_testnet_1/network/secret_ed25519 || \
    calibre-node key generate-node-key --chain /app/chain-specs/calibre-testnet-raw.json --base-path /data >/dev/null 2>&1; \
  exec \"$@\"", "--"]
CMD ["calibre-node", \
     "--chain", "/app/chain-specs/calibre-testnet-raw.json", \
     "--alice", \
     "--base-path", "/data", \
     "--port", "30333", \
     "--rpc-external", "--rpc-methods", "unsafe", "--rpc-cors", "all", \
     "--prometheus-external"]
