#!/usr/bin/env bash

ROOT=$(pwd)

# Build modules
cd "$ROOT"/modules
cargo build

# Build engine
cd "$ROOT"/engine
cargo build

# Copy modules to correct directory
cd "$ROOT"
mkdir -p ./bin/mods/

mkdir ./bin/mods/mod_core
cp ./modules/target/cargo/wasm32-wasi/debug/mod_core.wasm ./bin/mods/mod_core/
echo "{ \"id\": \"core\", \"version\": \"0.1\", \"entry\": \"mod_core.wasm\" }" > ./bin/mods/mod_core/manifest.json
