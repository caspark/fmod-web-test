#!/usr/bin/env bash

set -eu

watchexec --restart "./web-build.sh && cd dist && python3 -m http.server --bind localhost"
