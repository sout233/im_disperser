#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/target/bundled"
PKG_ROOT="${DIST_DIR}/pkg-root"
PKG_SCRIPTS="${DIST_DIR}/pkg-scripts"
PKG_PATH="${DIST_DIR}/im_disperser-macos-universal.pkg"
CARGO_XTASK=(cargo)
RUSTUP_STABLE_BIN=""

if command -v rustup >/dev/null 2>&1 && rustup run stable cargo --version >/dev/null 2>&1; then
  CARGO_XTASK=(rustup run stable cargo)
  RUSTUP_STABLE_BIN="$(rustup run stable rustc --print sysroot)/bin"
fi

"${ROOT_DIR}/scripts/bundle-au-universal.sh"

if [[ -n "${RUSTUP_STABLE_BIN}" ]]; then
  PATH="${RUSTUP_STABLE_BIN}:${PATH}" "${CARGO_XTASK[@]}" xtask bundle-universal im_disperser --release
else
  "${CARGO_XTASK[@]}" xtask bundle-universal im_disperser --release
fi

UNIVERSAL_DIR="${DIST_DIR}/im_disperser-macos-universal"
if [[ ! -d "${UNIVERSAL_DIR}/im_disperser.vst3" || ! -d "${UNIVERSAL_DIR}/im_disperser.clap" ]]; then
  echo "Universal VST3/CLAP bundle generation failed." >&2
  exit 1
fi

rm -rf "${PKG_ROOT}" "${PKG_SCRIPTS}"
mkdir -p \
  "${PKG_ROOT}/Library/Audio/Plug-Ins/Components" \
  "${PKG_ROOT}/Library/Audio/Plug-Ins/VST3" \
  "${PKG_ROOT}/Library/Audio/Plug-Ins/CLAP" \
  "${PKG_SCRIPTS}"

copy_bundle() {
  local source_path="$1"
  local destination_path="$2"

  ditto --norsrc --noextattr --noqtn "${source_path}" "${destination_path}"
}

copy_bundle "${DIST_DIR}/IM_DISPERSER.component" \
  "${PKG_ROOT}/Library/Audio/Plug-Ins/Components/IM_DISPERSER.component"
copy_bundle "${UNIVERSAL_DIR}/im_disperser.vst3" \
  "${PKG_ROOT}/Library/Audio/Plug-Ins/VST3/im_disperser.vst3"
copy_bundle "${UNIVERSAL_DIR}/im_disperser.clap" \
  "${PKG_ROOT}/Library/Audio/Plug-Ins/CLAP/im_disperser.clap"

# Reject Finder metadata before creating the payload instead of silently
# allowing it to reappear in an installer.
xattr -cr "${PKG_ROOT}" 2>/dev/null || true
find "${PKG_ROOT}" -name '._*' -type f -delete
if find "${PKG_ROOT}" -name '._*' -type f -print -quit | grep -q .; then
  echo "AppleDouble files remain in the package staging area." >&2
  exit 1
fi

cp "${ROOT_DIR}/packaging/macos/install-postinstall" "${PKG_SCRIPTS}/postinstall"
chmod 755 "${PKG_SCRIPTS}/postinstall"

rm -f "${PKG_PATH}"
pkgbuild \
  --root "${PKG_ROOT}" \
  --identifier "top.soout.godiedsp.im-disperser" \
  --version "0.1.4" \
  --install-location / \
  --scripts "${PKG_SCRIPTS}" \
  "${PKG_PATH}"

"${ROOT_DIR}/scripts/verify-macos-auv2.sh" \
  "${DIST_DIR}/IM_DISPERSER.component" "${PKG_PATH}"

echo "Installer package: ${PKG_PATH}"
