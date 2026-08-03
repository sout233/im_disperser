#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARM64_DIR="${ROOT_DIR}/target/aarch64-apple-darwin/release"
X86_64_DIR="${ROOT_DIR}/target/x86_64-apple-darwin/release"
DIST_DIR="${ROOT_DIR}/target/bundled"
COMPONENT_DIR="${DIST_DIR}/IM_DISPERSER.component"
COMPONENT_BINARY="${COMPONENT_DIR}/Contents/MacOS/IM_DISPERSER"
CARGO_NATIVE=(cargo)
CARGO_CROSS=(cargo)
CROSS_RUSTC=""

# Homebrew Rust and rustup can coexist. Prefer rustup's stable Cargo when it
# is available because the cross targets are installed in that toolchain.
if command -v rustup >/dev/null 2>&1 && rustup run stable cargo --version >/dev/null 2>&1; then
    CARGO_CROSS=(rustup run stable cargo)
    CROSS_RUSTC="$(rustup which rustc)"
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "This script must run on macOS." >&2
    exit 1
fi

cd "${ROOT_DIR}"

clean_bundle_metadata() {
    local bundle_path="$1"

    find "${bundle_path}" -name '._*' -type f -delete
    find "${bundle_path}" -name '__MACOSX' -type d -prune -exec rm -rf {} +
    xattr -cr "${bundle_path}" 2>/dev/null || true
}

echo "Building arm64 AUv2 library..."
"${CARGO_NATIVE[@]}" build --release --lib --target aarch64-apple-darwin

echo "Building x86_64 AUv2 library..."
if [[ -n "${CROSS_RUSTC}" ]]; then
    RUSTC="${CROSS_RUSTC}" "${CARGO_CROSS[@]}" build --release --lib --target x86_64-apple-darwin
else
    "${CARGO_CROSS[@]}" build --release --lib --target x86_64-apple-darwin
fi

rm -rf "${COMPONENT_DIR}"
mkdir -p "${COMPONENT_DIR}/Contents/MacOS" "${COMPONENT_DIR}/Contents/Resources"

lipo -create \
    "${ARM64_DIR}/libim_disperser.dylib" \
    "${X86_64_DIR}/libim_disperser.dylib" \
    -output "${COMPONENT_BINARY}"

# The cdylib's install name otherwise contains the local Cargo build path.
# It is not a runtime dependency, but keeping it bundle-relative makes the
# component self-contained when copied to another Mac.
install_name_tool -id "@rpath/IM_DISPERSER" "${COMPONENT_BINARY}"

cp "${ROOT_DIR}/packaging/macos/Info.auv2.plist" \
    "${COMPONENT_DIR}/Contents/Info.plist"
printf 'BNDL????' > "${COMPONENT_DIR}/Contents/PkgInfo"
chmod +x "${COMPONENT_BINARY}"
clean_bundle_metadata "${COMPONENT_DIR}"

# Seal the final, metadata-free bundle. Ad-hoc signing is sufficient for local
# testing; a public release can replace this with a Developer ID signature.
codesign --force --deep --sign - --timestamp=none "${COMPONENT_DIR}"

plutil -lint "${COMPONENT_DIR}/Contents/Info.plist"
codesign --verify --deep --strict "${COMPONENT_DIR}"
lipo -info "${COMPONENT_BINARY}"

mkdir -p "${DIST_DIR}"
rm -f "${DIST_DIR}/im_disperser-auv2-universal.zip"
ditto --norsrc --noextattr --noqtn -c -k --keepParent "${COMPONENT_DIR}" \
    "${DIST_DIR}/im_disperser-auv2-universal.zip"

echo "AUv2 component: ${COMPONENT_DIR}"
echo "AUv2 archive:   ${DIST_DIR}/im_disperser-auv2-universal.zip"
