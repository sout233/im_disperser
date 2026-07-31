#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/target/tools"
TEST_BINARY="${OUTPUT_DIR}/test-auv2-gui"

mkdir -p "${OUTPUT_DIR}"
clang++ -Wall -Wextra -Werror -fobjc-arc \
  "${ROOT_DIR}/scripts/test-auv2-gui.mm" \
  -framework AppKit \
  -framework AudioToolbox \
  -framework CoreFoundation \
  -framework AudioUnit \
  -o "${TEST_BINARY}"

exec "${TEST_BINARY}"
