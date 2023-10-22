#!/bin/bash

echo "--- Rust version info ---"
rustup --version
rustc --version
cargo --version

echo "--- Building plugin backend ---"
cargo build --profile docker
mkdir -p out

mv target/docker/deck-ds out/backend

echo " --- Cleaning up ---"
# remove root-owned target folder
cargo clean