#!/usr/bin/env bash

set -eu

if [ ! -d "fmod/linux/" ]; then
    echo "fmod/linux/ directory not found - should have fmod 2.02.22 linux sdk there"
    exit 1
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

# Copy fmod libraries to target directories
for profile in "debug" "release"; do
    mkdir -p "target/$profile"
    cp "$SCRIPT_DIR/fmod/linux/api/core/lib/x86_64/"* "target/$profile/"
    cp "$SCRIPT_DIR/fmod/linux/api/studio/lib/x86_64/"* "target/$profile/"
done

cargo run "$@"
