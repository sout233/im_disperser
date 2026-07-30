#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARM64_DIR="${ROOT_DIR}/target/aarch64-apple-darwin/release"
X86_64_DIR="${ROOT_DIR}/target/x86_64-apple-darwin/release"
DIST_DIR="${ROOT_DIR}/target/bundled"
STAGING_DIR="${DIST_DIR}/im_disperser-macos-universal"
DMG_PATH="${DIST_DIR}/im_disperser-macos-universal.dmg"
VST3_DIR="${STAGING_DIR}/im_disperser.vst3"
CLAP_DIR="${STAGING_DIR}/im_disperser.clap"

if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "This script must run on macOS." >&2
    exit 1
fi

cd "${ROOT_DIR}"
"${ROOT_DIR}/scripts/bundle-au-universal.sh"

rm -rf "${STAGING_DIR}" "${DMG_PATH}"
mkdir -p "${VST3_DIR}/Contents/MacOS" "${CLAP_DIR}/Contents/MacOS"

for bundle_dir in "${VST3_DIR}" "${CLAP_DIR}"; do
    cp "${ROOT_DIR}/packaging/macos/Info.plugin.plist" "${bundle_dir}/Contents/Info.plist"
done

lipo -create \
    "${ARM64_DIR}/libim_disperser.dylib" \
    "${X86_64_DIR}/libim_disperser.dylib" \
    -output "${VST3_DIR}/Contents/MacOS/im_disperser"
cp "${VST3_DIR}/Contents/MacOS/im_disperser" "${CLAP_DIR}/Contents/MacOS/im_disperser"

install_name_tool -id "@rpath/im_disperser" "${VST3_DIR}/Contents/MacOS/im_disperser"
install_name_tool -id "@rpath/im_disperser" "${CLAP_DIR}/Contents/MacOS/im_disperser"
chmod +x "${VST3_DIR}/Contents/MacOS/im_disperser" "${CLAP_DIR}/Contents/MacOS/im_disperser"

cp -R "${DIST_DIR}/IM_DISPERSER.component" "${STAGING_DIR}/IM_DISPERSER.component"
cp "${ROOT_DIR}/packaging/macos/INSTALL.txt" "${STAGING_DIR}/INSTALL.txt"

for bundle_dir in "${VST3_DIR}" "${CLAP_DIR}" "${STAGING_DIR}/IM_DISPERSER.component"; do
    codesign --force --deep --sign - "${bundle_dir}" >/dev/null
    codesign --verify --deep --strict "${bundle_dir}"
done

plutil -lint "${VST3_DIR}/Contents/Info.plist"
plutil -lint "${CLAP_DIR}/Contents/Info.plist"
plutil -lint "${STAGING_DIR}/IM_DISPERSER.component/Contents/Info.plist"
lipo -info "${VST3_DIR}/Contents/MacOS/im_disperser"
lipo -info "${CLAP_DIR}/Contents/MacOS/im_disperser"
lipo -info "${STAGING_DIR}/IM_DISPERSER.component/Contents/MacOS/IM_DISPERSER"

ditto -c -k --keepParent "${STAGING_DIR}" \
    "${DIST_DIR}/im_disperser-macos-universal.zip"
hdiutil create -volname "IM_DISPERSER macOS" -srcfolder "${STAGING_DIR}" \
    -ov -format UDZO "${DMG_PATH}"

echo "macOS staging directory: ${STAGING_DIR}"
echo "macOS archive:           ${DIST_DIR}/im_disperser-macos-universal.zip"
echo "macOS installer DMG:     ${DMG_PATH}"
