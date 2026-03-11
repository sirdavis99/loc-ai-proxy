# Build stage
FROM rust:1.78-slim as builder

WORKDIR /usr/src/loc-ai-proxy

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/loc-ai-proxy/target/release/locaiproxy /usr/local/bin/locaiproxy

# Create config directory
RUN mkdir -p /root/.config/loc-ai-proxy

# Expose port
EXPOSE 9110

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9110/health || exit 1

# Set environment
ENV RUST_LOG=info
ENV LOC_AI_PROXY_HOST=0.0.0.0
ENV LOC_AI_PROXY_PORT=9110

# Run the proxy
ENTRYPOINT ["locaiproxy"]
CMD ["--host", "0.0.0.0", "--port", "9110"]
