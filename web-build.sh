#!/usr/bin/env bash

set -eu

if [ ! -d "fmod/web/" ]; then
    echo "fmod/web/ directory not found - should have fmod 2.02.22 html5 sdk there"
    exit 1
fi

echo "Building..."

rm -rf dist
mkdir dist

cargo build --target wasm32-unknown-unknown

if [ ! -f "target/wasm32-unknown-unknown/debug/fmod-test.wasm" ]; then
    echo "fmod-test.wasm not found"
    exit 1
fi

wasm-bindgen target/wasm32-unknown-unknown/debug/fmod-test.wasm --out-dir dist --target web

cp index.html dist/index.html
cp fmod-web.js dist/fmod-web.js

# fmotstudioL is the large files - see https://www.fmod.com/docs/2.02/api/platforms-html5.html#upstream-wasm
cp fmod/web/api/studio/lib/upstream/wasm/fmodstudioL* dist/
mkdir -p dist/assets
cp fmod/web/api/studio/examples/public/js/* dist/assets/
