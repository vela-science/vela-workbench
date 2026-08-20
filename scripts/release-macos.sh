#!/bin/bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/release-macos.sh [--check]

Build and verify the signed, notarized macOS arm64 Workbench app and DMG,
then emit SPDX SBOM sidecars for the DMG and the source-locked manifests.
--check validates the release host, the pinned syft, clean remote-equal main
checkout, signing identity, and one complete notarization credential family
without building.
EOF
}

fail() {
  printf 'macOS release refused: %s\n' "$1" >&2
  exit 1
}

mode=build
case "${1-}" in
  "") ;;
  --check) mode=check ;;
  -h|--help) usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_root"

[ "$(uname -s)" = Darwin ] || fail "this release is qualified only on macOS"
[ "$(uname -m)" = arm64 ] || fail "this release is qualified only for Apple silicon (arm64)"

for tool in bun git security codesign spctl xcrun shasum hdiutil; do
  command -v "$tool" >/dev/null 2>&1 || fail "required tool is unavailable: $tool"
done
xcrun --find notarytool >/dev/null 2>&1 || fail "Xcode notarytool is unavailable"
xcrun --find stapler >/dev/null 2>&1 || fail "Xcode stapler is unavailable"

syft_version=1.50.0
command -v syft >/dev/null 2>&1 || fail "syft $syft_version is required for the SBOM stage and was not found; install the tagged release from https://github.com/anchore/syft/releases/tag/v$syft_version"
observed_syft=$(syft --version 2>/dev/null | awk '{print $NF}')
observed_syft=${observed_syft#v}
[ "$observed_syft" = "$syft_version" ] || fail "syft $observed_syft is not the pinned $syft_version"

source_commit=$(git rev-parse HEAD)
source_tree=$(git rev-parse 'HEAD^{tree}')
assert_source_state() {
  [ "$(git branch --show-current)" = main ] || fail "check out main before releasing"
  [ -z "$(git status --porcelain)" ] || fail "the worktree is not clean"
  git fetch --quiet origin main
  [ "$(git rev-parse HEAD)" = "$source_commit" ] || fail "HEAD changed during release"
  [ "$(git rev-parse 'HEAD^{tree}')" = "$source_tree" ] || fail "the source tree changed during release"
  [ "$source_commit" = "$(git rev-parse origin/main)" ] || fail "main is not equal to origin/main"
}
assert_source_state

signing_identity=${APPLE_SIGNING_IDENTITY-}
[ -n "$signing_identity" ] || fail "APPLE_SIGNING_IDENTITY is not set"
case "$signing_identity" in
  "Developer ID Application: "*) ;;
  *) fail "APPLE_SIGNING_IDENTITY must name a Developer ID Application identity" ;;
esac
security find-identity -v -p codesigning | grep -F -- "\"$signing_identity\"" >/dev/null || fail "APPLE_SIGNING_IDENTITY is not a valid identity in the current keychain"

api_count=0
for name in APPLE_API_ISSUER APPLE_API_KEY APPLE_API_KEY_PATH; do
  [ -n "${!name-}" ] && api_count=$((api_count + 1))
done
[ "$api_count" -eq 3 ] || fail "set all of APPLE_API_ISSUER, APPLE_API_KEY, and APPLE_API_KEY_PATH"
[ -f "$APPLE_API_KEY_PATH" ] && [ -r "$APPLE_API_KEY_PATH" ] || fail "APPLE_API_KEY_PATH is not a readable file"

api_issuer=$APPLE_API_ISSUER
api_key=$APPLE_API_KEY
api_key_path=$APPLE_API_KEY_PATH
unset APPLE_API_ISSUER APPLE_API_KEY APPLE_API_KEY_PATH APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID APPLE_SIGNING_IDENTITY APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_PROVIDER_SHORT_NAME APPLE_DEVELOPMENT_TEAM API_PRIVATE_KEYS_DIR TAURI_SIGNING_PRIVATE_KEY TAURI_SIGNING_PRIVATE_KEY_PATH TAURI_SIGNING_PRIVATE_KEY_PASSWORD TAURI_PRIVATE_KEY TAURI_PRIVATE_KEY_PATH TAURI_PRIVATE_KEY_PASSWORD

version=$(bun -e 'const p=JSON.parse(await Bun.file("package.json").text()); const t=JSON.parse(await Bun.file("src-tauri/tauri.conf.json").text()); if(p.version!==t.version) process.exit(1); process.stdout.write(p.version)' 2>/dev/null) || fail "package.json and tauri.conf.json versions differ"
cargo_version=$(awk -F '"' '/^version = "/ { print $2; exit }' src-tauri/Cargo.toml)
[ "$version" = "$cargo_version" ] || fail "package.json and Cargo.toml versions differ"
app_path="$repo_root/src-tauri/target/release/bundle/macos/Vela Workbench.app"
dmg_path="$repo_root/src-tauri/target/release/bundle/dmg/Vela Workbench_${version}_aarch64.dmg"
dmg_sbom_path="${dmg_path}.spdx.json"
source_sbom_path="${dmg_path%.dmg}.source.spdx.json"

printf 'macOS release preflight passed\n'
printf '  source commit: %s\n' "$source_commit"
printf '  source tree:   %s\n' "$source_tree"
printf '  app version:   %s\n' "$version"
printf '  signing:       %s\n' "$signing_identity"
printf '  notarization:  App Store Connect API\n'
printf '  sbom tool:     syft %s\n' "$syft_version"

[ "$mode" = build ] || exit 0

bun install --frozen-lockfile
bun run check
assert_source_state
CI=true bun run tauri build --no-bundle --no-sign -- --locked
assert_source_state
rm -rf -- "$app_path"
rm -f -- "$dmg_path" "${dmg_path}.sha256" \
  "$dmg_sbom_path" "${dmg_sbom_path}.sha256" \
  "$source_sbom_path" "${source_sbom_path}.sha256"
APPLE_SIGNING_IDENTITY="$signing_identity" APPLE_API_ISSUER="$api_issuer" APPLE_API_KEY="$api_key" APPLE_API_KEY_PATH="$api_key_path" CI=true bun run tauri bundle --bundles app,dmg --ci
assert_source_state

[ -d "$app_path" ] || fail "expected app bundle was not produced"
[ -f "$dmg_path" ] || fail "expected DMG was not produced"

codesign --verify --deep --strict --verbose=2 "$app_path"
spctl --assess --type execute --verbose=4 "$app_path"
xcrun stapler validate "$app_path"
signature_details=$(codesign -dv --verbose=4 "$app_path" 2>&1)
printf '%s\n' "$signature_details" | grep -F -- "Authority=$signing_identity" >/dev/null || fail "app signer does not match APPLE_SIGNING_IDENTITY"
identity_team=${signing_identity##*(}
identity_team=${identity_team%)}
[ -n "$identity_team" ] || fail "could not derive the signing team from APPLE_SIGNING_IDENTITY"
printf '%s\n' "$signature_details" | grep -F -- "TeamIdentifier=$identity_team" >/dev/null || fail "app TeamIdentifier does not match APPLE_SIGNING_IDENTITY"
codesign --verify --verbose=2 "$dmg_path"
spctl --assess --type open --context context:primary-signature --verbose=4 "$dmg_path"
xcrun stapler validate "$dmg_path"
hdiutil verify "$dmg_path"

mount_path=$(mktemp -d /tmp/vela-workbench-release-mount.XXXXXX)
mounted=false
cleanup_mount() {
  if [ "$mounted" = true ]; then
    hdiutil detach "$mount_path" >/dev/null || true
  fi
  rmdir "$mount_path" 2>/dev/null || true
}
trap cleanup_mount EXIT
hdiutil attach -readonly -nobrowse -mountpoint "$mount_path" "$dmg_path" >/dev/null
mounted=true
installed_app="$mount_path/Vela Workbench.app"
[ -d "$installed_app" ] || fail "the DMG does not contain Vela Workbench.app"
codesign --verify --deep --strict --verbose=2 "$installed_app"
spctl --assess --type execute --verbose=4 "$installed_app"
xcrun stapler validate "$installed_app"
installed_signature=$(codesign -dv --verbose=4 "$installed_app" 2>&1)
printf '%s\n' "$installed_signature" | grep -F -- "Authority=$signing_identity" >/dev/null || fail "DMG-contained app signer does not match APPLE_SIGNING_IDENTITY"
printf '%s\n' "$installed_signature" | grep -F -- "TeamIdentifier=$identity_team" >/dev/null || fail "DMG-contained app TeamIdentifier does not match APPLE_SIGNING_IDENTITY"
hdiutil detach "$mount_path" >/dev/null
mounted=false
rmdir "$mount_path"
trap - EXIT

assert_source_state

checksum_path="${dmg_path}.sha256"
(
  cd "$(dirname -- "$dmg_path")"
  shasum -a 256 "$(basename -- "$dmg_path")" > "$(basename -- "$checksum_path")"
)

syft scan "file:$dmg_path" -o "spdx-json=$dmg_sbom_path" --quiet || fail "syft could not generate the DMG SBOM"
[ -s "$dmg_sbom_path" ] || fail "the DMG SBOM is missing or empty"
syft scan "dir:$repo_root" \
  --exclude ./node_modules --exclude ./src-tauri/target --exclude ./dist \
  -o "spdx-json=$source_sbom_path" --quiet || fail "syft could not generate the source-locked SBOM"
[ -s "$source_sbom_path" ] || fail "the source-locked SBOM is missing or empty"
for sbom in "$dmg_sbom_path" "$source_sbom_path"; do
  (
    cd "$(dirname -- "$sbom")"
    shasum -a 256 "$(basename -- "$sbom")" > "$(basename -- "$sbom").sha256"
  )
done

assert_source_state

printf 'signed and notarized macOS release verified\n'
printf '  app:         %s\n' "$app_path"
printf '  dmg:         %s\n' "$dmg_path"
printf '  checksum:    %s\n' "$checksum_path"
printf '  dmg sbom:    %s\n' "$dmg_sbom_path"
printf '  source sbom: %s\n' "$source_sbom_path"
