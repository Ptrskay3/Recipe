#!/usr/bin/env bash

set -ex

docker-compose down
docker-compose -f docker-compose.dev.yml up -d 
cargo run
