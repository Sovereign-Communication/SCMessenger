# Build stage
FROM rust:latest AS builder

# Optional: Override bootstrap nodes at build time
# Format: comma-separated multiaddrs
# Example: /ip4/1.2.3.4/tcp/9001/p2p/12D3Koo...,/ip4/5.6.7.8/tcp/9001/p2p/12D3Koo...
ARG SC_BOOTSTRAP_NODES=""
ENV SC_BOOTSTRAP_NODES=${SC_BOOTSTRAP_NODES}

WORKDIR /usr/src/scmessenger

# System build dependencies for the workspace's -sys crates: libdbus-sys needs
# libdbus-1-dev, openssl-sys needs libssl-dev, and both need pkg-config to
# locate them. Without these, `cargo build` panics in libdbus-sys/build.rs.
RUN apt-get update && apt-get install -y \
    pkg-config \
    libdbus-1-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests (must include every workspace member from the root
# Cargo.toml: core, cli, wasm, mobile, desktop_bridge - a missing member's
# Cargo.toml makes the workspace build fail with "failed to read
# <member>/Cargo.toml").
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml core/
COPY cli/Cargo.toml cli/
COPY mobile/Cargo.toml mobile/
COPY wasm/Cargo.toml wasm/
COPY desktop_bridge/Cargo.toml desktop_bridge/

# Copy source code
COPY core/src core/src
COPY cli/src cli/src
COPY mobile/src mobile/src
COPY wasm/src wasm/src
COPY desktop_bridge/src desktop_bridge/src

# Copy build scripts
COPY core/build.rs core/
COPY cli/build.rs cli/
COPY mobile/build.rs mobile/
COPY desktop_bridge/build.rs desktop_bridge/

# Build the CLI and stress-test harness.
#
# Overrides the workspace's [profile.release] lto=true (fat)/codegen-units=1
# via env var rather than editing Cargo.toml: fat LTO with a single codegen
# unit is the most memory-hungry link configuration rustc/LLVM has, commonly
# needing 4GB+ RAM at the final link step alone for a dependency graph this
# size (659 locked packages, full libp2p feature set, two web stacks, the
# crypto stack). That's fine on a normal dev machine or CI runner, but this
# image also needs to build unattended on farm-sim test nodes as small as a
# 1GiB-RAM t3.micro -- this override trades a slightly less-optimized binary
# for a build that reliably fits in constrained memory. Does not affect the
# UniFFI mobile/wasm build paths, which build via separate gen_kotlin/
# gen_swift/wasm-pack invocations outside this Dockerfile.
ENV CARGO_PROFILE_RELEASE_LTO=false
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16
# Confirmed via live SSH into a building farm-sim node (2026-07-18): even with
# the LTO override above, multiple rustc processes run concurrently by
# default (observed 2 simultaneous ~270MB-RSS `rustc` processes compiling
# uniffi_bindgen, one of them still showing -C codegen-units=1 -- build-time
# host artifacts like proc-macros/uniffi_bindgen appear to fall outside the
# profile override), and on a 1GiB instance that pushed the box into 2.4GB+
# of active swap usage (load average 2.3 on a single vCPU). Capping the
# number of concurrent compilation units directly bounds peak concurrent
# memory instead of hoping the profile override alone is enough.
ARG CARGO_BUILD_JOBS=1
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
RUN cargo build --release --bin scmessenger-cli --bin stress-test

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies and network tools
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    iproute2 \
    iputils-ping \
    curl \
    jq \
    libdbus-1-3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries from builder
COPY --from=builder /usr/src/scmessenger/target/release/scmessenger-cli /usr/local/bin/scm
COPY --from=builder /usr/src/scmessenger/target/release/stress-test /usr/local/bin/stress-test

# Copy entrypoint script
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Environment variables
ENV RUST_LOG=info
ENV SCM_CONFIG_DIR=/root/.config/scmessenger
ENV SCM_DATA_DIR=/root/.local/share/scmessenger
ENV LISTEN_PORT=9000

# Expose ports
EXPOSE 9000 9001

ENTRYPOINT ["entrypoint.sh"]
CMD ["scm", "start"]
