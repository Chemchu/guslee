FROM lukemathwalker/cargo-chef:latest-rust-1-trixie AS chef
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /usr/src/app/recipe.json recipe.json
# Only the server binary: skips dev-only workspace members such as the e2e crate
RUN cargo chef cook --release --recipe-path recipe.json --bin guslee
COPY . .
RUN cargo build --release --bin guslee

# Runtime image - same Debian release as the builder (glibc must match)
FROM debian:trixie-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash app
WORKDIR /app

COPY --from=builder /usr/src/app/target/release/guslee ./guslee
COPY --from=builder /usr/src/app/garden ./garden
COPY --from=builder /usr/src/app/static ./static
COPY --from=builder /usr/src/app/templates ./templates

ENV TEMPLATE_PATH=/app/templates
RUN chown -R app:app /app
USER app
EXPOSE 3000
CMD ["./guslee"]
