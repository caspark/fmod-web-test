#!/usr/bin/env bash

set -eu

if [ ! -d "fmod/linux/" ]; then
    echo "fmod/linux/ directory not found - should have fmod 2.02.22 linux sdk there"
    exit 1
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

export LD_LIBRARY_PATH="$SCRIPT_DIR/fmod/linux/api/core/lib/x86_64:$SCRIPT_DIR/fmod/linux/api/studio/lib/x86_64"
export FMOD_SYS_FMOD_DIRECTORY="$SCRIPT_DIR/fmod/linux"

cargo run -- fmod/linux/api/studio/examples/
