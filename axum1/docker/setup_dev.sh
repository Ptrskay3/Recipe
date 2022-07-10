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
docker-compose -f docker-compose.dev.yml build
