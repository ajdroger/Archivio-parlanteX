# ============================================================================
# Archivio Parlante — Rust Engine Dockerfile
# ============================================================================
# Multi-stage build for minimal production image

# === Stage 1: Build ===
# Using rust:nightly for edition2024 support (time-core 0.1.8 requires Rust 1.88+)
# TODO: Switch to rust:1.88-bookworm when released (estimated Q2 2025)
FROM rust:nightly-bookworm AS builder

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Copy source
COPY src ./src

# Build release binary
RUN cargo build --release

# === Stage 2: Runtime ===
FROM debian:bookworm-slim

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/archivio-parlante-rust-engine /app/rust-engine

# Expose port
EXPOSE 8090

# Run as non-root user (security)
RUN useradd -m -u 1000 rustuser && \
    chown -R rustuser:rustuser /app
USER rustuser

# Start server
CMD ["/app/rust-engine"]
