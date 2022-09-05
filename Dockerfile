FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Install cmake
RUN apt-get update && apt-get --yes install cmake
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --package twitch-chat-logger

# We do not need the Rust toolchain to run the binary!
FROM debian:bullseye-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/twitch-chat-logger /usr/local/bin
ENV RUST_LOG=info
VOLUME ["/app/.refreshed-token.json", "/app/config.ron"]
ENTRYPOINT ["/usr/local/bin/twitch-chat-logger"]

