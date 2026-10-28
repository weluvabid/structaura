#!/usr/bin/env sh
set -eu

cargo run --release -p structaura --features bench-internals --example benchmark
