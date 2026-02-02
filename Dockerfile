# Build stage
# Requires Rust 1.85+ for edition 2024
FROM rust:slim-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY . .

# Build release binary
RUN cargo build --release --bin mdbook-lint

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (CA certificates for any HTTPS operations)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/mdbook-lint /usr/local/bin/mdbook-lint

# Create non-root user
RUN useradd -m -s /bin/bash linter
USER linter

WORKDIR /workspace

ENTRYPOINT ["mdbook-lint"]
CMD ["--help"]
