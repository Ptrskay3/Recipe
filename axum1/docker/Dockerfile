FROM lukemathwalker/cargo-chef:latest-rust-1.67.0 AS chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path chef-recipe.json

FROM chef as builder
ENV SQLX_OFFLINE=1
COPY --from=planner /app/chef-recipe.json chef-recipe.json
RUN cargo chef cook --release --recipe-path chef-recipe.json
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim as runtime
ARG APP_ENV
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl wget ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/axum1 /usr/local/bin
COPY configuration configuration
ENV APP_ENVIRONMENT $APP_ENV
ENTRYPOINT ["/usr/local/bin/axum1"]
