#!/usr/bin/env bash

set -eu

if [ ! -d "fmod" ]; then
    echo "fmod/ directory not found - should have fmod 2.02.22 sdk there"
    exit 1
fi

export LD_LIBRARY_PATH=fmod/api/core/lib/x86_64:fmod/api/studio/lib/x86_64
export FMOD_SYS_FMOD_DIRECTORY=fmod

cargo run
