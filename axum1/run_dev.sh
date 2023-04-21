#!/usr/bin/env bash

set -ex

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install sqlx-cli --version=0.6.2"
  echo >&2 "to install it."
  exit 1
fi

docker compose -f ./docker/docker-compose.dev.yml down
docker compose -f docker/docker-compose.dev.yml up -d

PG_CONTAINER=$(docker ps --filter 'name=postgres' --format '{{.ID}}')
timeout 30s bash -c "until docker exec $PG_CONTAINER pg_isready ; do sleep 5 ; done"

sqlx migrate run
cargo run
