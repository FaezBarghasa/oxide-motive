#!/bin/bash
set -e

echo "--- Checking firmware for thumbv7em-none-eabihf ---"
cargo check --target thumbv7em-none-eabihf -p oxide-firmware

echo "--- Checking host for x86_64-unknown-linux-gnu ---"
cargo check --target x86_64-unknown-linux-gnu -p oxide-host

echo "--- Running tests ---"
cargo test --workspace

echo "--- Running clippy ---"
cargo clippy --workspace -- -D warnings

echo "--- All checks passed! ---"
