#!/bin/bash

set -euo pipefail

echo "--- Checking workspace for MCU target (oxide-firmware) ---"
cargo check --target thumbv7em-none-eabihf -p oxide-firmware

echo "--- Checking workspace for Host target (oxide-host) ---"
cargo check --target x86_64-unknown-linux-gnu -p oxide-host

echo "--- Running all tests ---"
cargo test --workspace

echo "--- Running Clippy with strict warnings ---"
# The `+nightly` is often needed for some clippy features or when using unstable Rust features.
# If not using nightly, remove it.
# For this project, we assume stable Rust is sufficient unless specified.
cargo clippy --workspace -- -D warnings -W clippy::pedantic -W clippy::restriction

echo "--- All checks passed! ---"
