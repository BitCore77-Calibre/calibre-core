# Runtime-only image for the Calibre node.
#
# The release binary is built on the host (cargo build --release) and
# copied in. Rebuilding in-container would need 20+ GB of disk and
# 30+ minutes per build; not worth it for local testnets.
#
# Build:  cargo build --release -p solochain-template-node
#         docker compose -f deploy/docker-compose.yml build
# Run:    docker compose -f deploy/docker-compose.yml up -d

FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
        libstdc++6 libgcc-s1 libc6 ca-certificates \
        curl iproute2 iputils-ping \
    && rm -rf /var/lib/apt/lists/*

COPY deploy/bin/node /usr/local/bin/node
COPY deploy/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

WORKDIR /data
EXPOSE 30333 9944 9615
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
