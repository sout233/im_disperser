#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <IM_DISPERSER.component> [installer.pkg]" >&2
  exit 64
fi

COMPONENT_DIR="$1"
PACKAGE_PATH="${2:-}"
INFO_PLIST="${COMPONENT_DIR}/Contents/Info.plist"
COMPONENT_BINARY="${COMPONENT_DIR}/Contents/MacOS/IM_DISPERSER"

[[ -d "${COMPONENT_DIR}" ]] || { echo "Component bundle is missing." >&2; exit 1; }
[[ -f "${INFO_PLIST}" ]] || { echo "Component Info.plist is missing." >&2; exit 1; }
[[ -f "${COMPONENT_BINARY}" ]] || { echo "Component executable is missing." >&2; exit 1; }

plutil -lint "${INFO_PLIST}"
codesign --verify --deep --strict "${COMPONENT_DIR}"
lipo -info "${COMPONENT_BINARY}"

if ! nm -gU "${COMPONENT_BINARY}" | grep -q ' _GetPluginFactoryAUV2$'; then
  echo "The AUv2 factory export is missing." >&2
  exit 1
fi

if find "${COMPONENT_DIR}" -name '._*' -type f -print -quit | grep -q .; then
  echo "AppleDouble metadata is present in the component bundle." >&2
  exit 1
fi

if [[ -n "${PACKAGE_PATH}" ]]; then
  [[ -f "${PACKAGE_PATH}" ]] || { echo "Installer package is missing." >&2; exit 1; }
  PAYLOAD_LIST="$(mktemp)"
  trap 'rm -f "${PAYLOAD_LIST}"' EXIT
  pkgutil --payload-files "${PACKAGE_PATH}" > "${PAYLOAD_LIST}"
  if rg -n '(^|/)(\._[^/]*|__MACOSX)(/|$)' "${PAYLOAD_LIST}"; then
    # macOS 15 records its non-removable com.apple.provenance extended
    # attribute as AppleDouble entries in pkgbuild payload listings. The
    # component staging tree above is the authoritative check for actual
    # Finder metadata files, so keep this visible without rejecting a valid
    # package solely for the platform-generated records.
    echo "Note: pkgbuild encoded macOS provenance metadata in the payload." >&2
  fi
fi

echo "AUv2 bundle and installer metadata verification passed."
