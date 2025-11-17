# Boundless BLS Blockchain - Multi-stage Docker Build
# This Dockerfile creates an optimized production image for the blockchain node

# Stage 1: Builder
FROM rustlang/rust:nightly-bookworm as builder

# Install system dependencies including ninja-build for CMake
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    ninja-build \
    libssl-dev \
    pkg-config \
    git \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Install liboqs for post-quantum cryptography
WORKDIR /tmp
RUN git clone --depth 1 --branch 0.9.0 https://github.com/open-quantum-safe/liboqs.git && \
    cd liboqs && \
    mkdir build && cd build && \
    cmake -GNinja -DCMAKE_INSTALL_PREFIX=/usr/local .. && \
    ninja && ninja install && \
    ldconfig && \
    cd / && rm -rf /tmp/liboqs

# Set working directory
WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY node/Cargo.toml ./node/
COPY core/Cargo.toml ./core/
COPY consensus/Cargo.toml ./consensus/
COPY crypto/Cargo.toml ./crypto/
COPY wasm-runtime/Cargo.toml ./wasm-runtime/
COPY p2p/Cargo.toml ./p2p/
COPY rpc/Cargo.toml ./rpc/
COPY storage/Cargo.toml ./storage/
COPY cli/Cargo.toml ./cli/

# Create dummy source files to cache dependencies
RUN mkdir -p node/src core/src consensus/src crypto/src wasm-runtime/src \
    p2p/src rpc/src storage/src cli/src && \
    echo "fn main() {}" > node/src/main.rs && \
    echo "fn main() {}" > cli/src/main.rs && \
    echo "pub fn dummy() {}" > core/src/lib.rs && \
    echo "pub fn dummy() {}" > consensus/src/lib.rs && \
    echo "pub fn dummy() {}" > crypto/src/lib.rs && \
    echo "pub fn dummy() {}" > wasm-runtime/src/lib.rs && \
    echo "pub fn dummy() {}" > p2p/src/lib.rs && \
    echo "pub fn dummy() {}" > rpc/src/lib.rs && \
    echo "pub fn dummy() {}" > storage/src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release --bin boundless-node && \
    cargo build --release --bin boundless-cli && \
    rm -rf node/src core/src consensus/src crypto/src wasm-runtime/src \
    p2p/src rpc/src storage/src cli/src \
    target/release/boundless* target/release/deps/boundless* \
    target/release/build/boundless* target/release/.fingerprint/boundless* \
    target/release/incremental

# Copy actual source code
COPY . .

# Build the actual application
RUN cargo build --release --bin boundless-node && \
    cargo build --release --bin boundless-cli && \
    strip /build/target/release/boundless-node && \
    strip /build/target/release/boundless-cli

# Stage 2: Runtime
FROM debian:bookworm-slim

# Build argument to bust cache
ARG CACHE_BUST=1

# Install runtime dependencies including curl for health checks
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy liboqs from builder
COPY --from=builder /usr/local/lib/liboqs.so* /usr/local/lib/
RUN ldconfig

# Create app user
RUN useradd -m -u 1000 -s /bin/bash boundless

# Create data directory
RUN mkdir -p /data && chown boundless:boundless /data

# Copy binaries from builder
COPY --from=builder /build/target/release/boundless-node /usr/local/bin/
COPY --from=builder /build/target/release/boundless-cli /usr/local/bin/

# Set permissions
RUN chmod +x /usr/local/bin/boundless-node && \
    chmod +x /usr/local/bin/boundless-cli

# Switch to app user
USER boundless
WORKDIR /data

# Expose ports
# 30333 - P2P port
# 9933 - RPC port
EXPOSE 30333 9933

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:9933/health || exit 1

# Default command
ENTRYPOINT ["/usr/local/bin/boundless-node"]
CMD ["--base-path", "/data"]
