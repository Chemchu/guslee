FROM lukemathwalker/cargo-chef:latest-rust-1-trixie AS chef
WORKDIR /usr/src/app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin guslee
COPY . .
RUN cargo build --release --bin guslee
# "Last updated" dates come from git, which the runtime image doesn't have.
# Same arguments as search_engine::git_dates::GIT_LOG_ARGS
RUN cd garden && git log --format=%x00%cs --name-only --relative -- . > ../garden-dates.txt || true

# Runtime image - same base as builder
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
COPY --from=builder /usr/src/app/garden-dates.txt ./garden-dates.txt

ENV TEMPLATE_PATH=/app/templates
ENV GARDEN_DATES_FILE=/app/garden-dates.txt
RUN chown -R app:app /app
USER app
EXPOSE 3000
CMD ["./guslee"]
