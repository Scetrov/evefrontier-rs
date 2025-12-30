# Build stage
FROM rust:1.91.1-bookworm AS builder

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

# Build the scout-gates service with release optimizations
RUN cargo build --release -p evefrontier-service-scout-gates

# Strip debug symbols for smaller binary
RUN strip /app/target/release/evefrontier-service-scout-gates

# Runtime stage - using Distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy the binary from builder
COPY --from=builder /app/target/release/evefrontier-service-scout-gates /usr/local/bin/evefrontier-service-scout-gates

# Set non-root user (distroless:nonroot is UID 65532)
USER nonroot

# Expose the default service port
EXPOSE 8080

# Set default environment variables
ENV SERVICE_PORT=8080
ENV RUST_LOG=info

# Run the service
ENTRYPOINT ["/usr/local/bin/evefrontier-service-scout-gates"]
