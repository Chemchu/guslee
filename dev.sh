#!/usr/bin/env bash

cleanup() {
  echo "Cleaning up..."
  pkill -f surrealdb
}

trap cleanup EXIT

pkill -f surrealdb || true
/home/gus/.surrealdb/surreal start --unauthenticated &
sleep 2
cargo run
