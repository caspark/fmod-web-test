#!/usr/bin/env bash

set -eu

if [ ! -d "fmod" ]; then
    echo "fmod/ directory not found - should have fmod 2.02.22 sdk there"
    exit 1
fi

export LD_LIBRARY_PATH=fmod/api/core/lib/x86_64:fmod/api/studio/lib/x86_64
export FMOD_SYS_FMOD_DIRECTORY=fmod

echo "Building..."
cargo build --target wasm32-unknown-unknown

if [ ! -f "target/wasm32-unknown-unknown/debug/fmod-test.wasm" ]; then
    echo "fmod-test.wasm not found"
    exit 1
fi

wasm-bindgen target/wasm32-unknown-unknown/debug/fmod-test.wasm --out-dir dist --target web

cp index.html dist/index.html
