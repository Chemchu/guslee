FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin guslee

# Runtime image
FROM debian:bookworm-slim

RUN useradd -ms /bin/bash app
WORKDIR /app

COPY --from=builder /usr/src/app/target/release/guslee ./guslee
COPY --from=builder /usr/src/app/garden ./garden
COPY --from=builder /usr/src/app/static ./static
COPY --from=builder /usr/src/app/templates ./templates

ENV GARDEN_PATH=/app/garden
ENV TEMPLATE_PATH=/app/templates

RUN chown -R app:app /app
USER app

EXPOSE 3000
CMD ["./guslee"]
