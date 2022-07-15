#!/usr/bin/env bash

set -x
set -eo pipefail

directories=(
    "volumes/pgadmin"
    "volumes/redis-data"
    "volumes/postgres"
    "volumes/meili"
)

mkdir -p "${directories[@]}"
sudo chown -R $(id -u):$(id -g) volumes/
docker network inspect axum_net >/dev/null 2>&1 || docker network create --driver bridge axum_net
docker-compose -f docker-compose.dev.yml build
