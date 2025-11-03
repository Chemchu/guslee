#!/usr/bin/env bash

cleanup() {
  pkill -P $$ 2>/dev/null
  pkill -f surrealdb 2>/dev/null
  pkill -f "target/debug/guslee" 2>/dev/null
  killall -9 guslee 2>/dev/null || true
}

trap cleanup EXIT SIGTERM SIGINT

pkill -f surrealdb 2>/dev/null || true
pkill -f "target/debug/guslee" 2>/dev/null || true

sleep 0.5

/home/gus/.surrealdb/surreal start --unauthenticated &
sleep 2

cargo run
