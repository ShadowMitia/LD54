#!/usr/bin/env bash

# https://github.com/bevyengine/bevy/tree/v0.11.3/examples#wasm

cargo build --profile wasm-release --target wasm32-unknown-unknown
wasm-bindgen --out-name wasm_game \
  --out-dir wasm \
  --target web target/wasm32-unknown-unknown/wasm-release/ld54.wasm
cp restart-audio-context.js wasm
