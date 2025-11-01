FROM rust:1-bookworm AS builder
WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/guslee ./guslee

# Runtime image
FROM debian:bookworm-slim

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /usr/src/app

# Copy the entire project from builder
COPY --from=builder /usr/src/app /usr/src/app

EXPOSE 3000

CMD ["./guslee"]
