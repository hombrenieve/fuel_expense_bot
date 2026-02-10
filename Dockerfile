# Build stage - compile the Rust application
FROM docker.io/library/rust:1.93-bookworm AS builder

WORKDIR /build

# Copy dependency manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ ./src/

# Fetch dependencies to cache them
RUN cargo fetch

# Build with release optimizations and strip debug symbols
RUN cargo build --release && \
    strip target/release/telegram-fuel-bot

# Runtime stage - minimal production image
FROM gcr.io/distroless/cc-debian12:latest

# Copy the compiled binary from build stage
COPY --from=builder /build/target/release/telegram-fuel-bot /usr/local/bin/telegram-fuel-bot

# Copy database configuration for containerized deployment
COPY .env.container /app/.env

# Set working directory
WORKDIR /app

# Configure graceful shutdown signal
STOPSIGNAL SIGTERM

# Set entrypoint to the bot binary
ENTRYPOINT ["/usr/local/bin/telegram-fuel-bot"]
