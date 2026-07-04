#!/bin/bash
set -e

# Compile all targets
cargo build --all-targets

# Run clippy with strict warnings
cargo clippy -- -D warnings
