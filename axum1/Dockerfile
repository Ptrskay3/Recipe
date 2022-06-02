FROM lukemathwalker/cargo-chef:latest-rust-1.61.0 AS chef
WORKDIR /app

ENV SQLX_OFFLINE=1

FROM chef as planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim as runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/axum1 /usr/local/bin
ENV APP_ENVIRONMENT production
ENTRYPOINT ["/usr/local/bin/axum1"]
