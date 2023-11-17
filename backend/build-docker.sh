#!/bin/bash

echo "--- Rust version info ---"
rustup --version
rustc --version
cargo --version

echo "--- Building plugin backend ---"
mkdir -p schema
cargo run --profile docker  -- schema schema
mkdir -p out

mv target/docker/deck-ds out/backend

echo " --- Cleaning up ---"
# remove root-owned target folder
cargo clean