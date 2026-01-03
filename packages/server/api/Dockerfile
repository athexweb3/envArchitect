# Builder Stage
FROM rust:1.81-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    clang \
    llvm \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifest
COPY Cargo.toml Cargo.lock ./

# Copy ALL member crates
COPY apps/cli/Cargo.toml apps/cli/Cargo.toml
COPY apps/web/package.json apps/web/package.json 
COPY packages/server/api/Cargo.toml packages/server/api/Cargo.toml
# packages/server/core does not exist
COPY packages/server/database/Cargo.toml packages/server/database/Cargo.toml
COPY packages/server/worker/Cargo.toml packages/server/worker/Cargo.toml
COPY packages/plugins/node/Cargo.toml packages/plugins/node/Cargo.toml
# shared is under server/shared
COPY packages/server/shared/Cargo.toml packages/server/shared/Cargo.toml

# Create dummy src files to cache dependencies
RUN mkdir -p apps/cli/src && echo "fn main() {}" > apps/cli/src/main.rs
RUN mkdir -p apps/web 
RUN mkdir -p packages/server/api/src && echo "fn main() {}" > packages/server/api/src/main.rs
RUN mkdir -p packages/server/database/src && echo "fn main() {}" > packages/server/database/src/lib.rs
RUN mkdir -p packages/server/worker/src && echo "fn main() {}" > packages/server/worker/src/lib.rs
RUN mkdir -p packages/server/shared/src && echo "fn main() {}" > packages/server/shared/src/lib.rs
RUN mkdir -p packages/plugins/node/src && echo "fn main() {}" > packages/plugins/node/src/lib.rs

# Build dependencies
RUN cargo build --release -p registry-api || true

# Now copy the actual source code
COPY . .

# Touch the main files to force rebuild of the app packages
RUN touch packages/server/api/src/main.rs

# Build the binary
RUN cargo build --release -p registry-api

# Runtime Stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/registry-api .

# Expose port
EXPOSE 3000

# Run
CMD ["./registry-api"]
