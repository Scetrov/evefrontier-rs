# Build stage
FROM rust:1.97.0-bookworm@sha256:8fa55b2f3ddf97471ab6a767bfa3f37e6bad0986ba823e75fea57e2a2a5c3073 AS builder

WORKDIR /app

# Install cross-compilation tools for smaller binaries
RUN apt-get update && apt-get install -y \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy workspace root src (required for workspace package)
COPY src/ src/

# Copy all workspace crates
COPY crates/ crates/

# Build the scout-range service with release optimizations
RUN cargo build --release -p evefrontier-service-scout-range

# Strip debug symbols for smaller binary
RUN strip /app/target/release/evefrontier-service-scout-range

# Runtime stage - using Distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:66aa873a4a14fb164aa01296058efd8253744606d72715e45acface073359faa

# Copy the binary from builder
COPY --from=builder /app/target/release/evefrontier-service-scout-range /usr/local/bin/evefrontier-service-scout-range

# Set non-root user (distroless:nonroot is UID 65532)
USER nonroot

# Expose the default service port
EXPOSE 8080

# Set default environment variables
ENV SERVICE_PORT=8080
ENV RUST_LOG=info

# Run the service
ENTRYPOINT ["/usr/local/bin/evefrontier-service-scout-range"]
