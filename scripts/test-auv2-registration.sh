#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/target/tools"
TEST_BINARY="${OUTPUT_DIR}/test-auv2-registration"

mkdir -p "${OUTPUT_DIR}"
cc -Wall -Wextra -Werror \
  "${ROOT_DIR}/scripts/test-auv2-registration.c" \
  -framework AudioToolbox \
  -framework CoreFoundation \
  -o "${TEST_BINARY}"

exec "${TEST_BINARY}" "$@"
