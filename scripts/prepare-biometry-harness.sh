#!/bin/bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
if [[ $# -ne 0 ]]; then
  echo "This script creates a fresh private temporary directory and accepts no output path." >&2
  exit 2
fi
out_dir="$(mktemp -d "${TMPDIR:-/tmp}/geld-biometry.XXXXXX")"
[[ -d "$out_dir" && ! -L "$out_dir" && -z "$(ls -A "$out_dir")" ]] || {
  echo "Refusing non-empty or symlinked temporary directory." >&2
  exit 2
}
out_dir="$(cd "$out_dir" && pwd -P)"
identity="${GELD_TEST_CODESIGN_IDENTITY:--}"
entitlements="${GELD_TEST_ENTITLEMENTS:-}"
profile="${GELD_TEST_PROFILE:-}"
run_id="$(uuidgen | tr '[:upper:]' '[:lower:]')"
service="app.geldapp.biometry.validation.${run_id}"
account="vault-key-test-${run_id}"
bundle_id="${GELD_TEST_BUNDLE_ID:-app.geldapp.biometry.validation}"

if [[ "$service" == "app.geldapp.desktop" || "$account" == "vault-key-v1" ]]; then
  echo "Refusing production Keychain namespace." >&2
  exit 2
fi
app="$out_dir/GeldBiometryHarness.app"
[[ ! -e "$app" && ! -L "$app" ]] || {
  echo "Refusing to overwrite an existing harness bundle." >&2
  exit 2
}
mkdir -m 700 "$app"
mkdir -m 700 "$app/Contents"
mkdir -m 700 "$app/Contents/MacOS"
exe="$app/Contents/MacOS/GeldBiometryHarness"
if [[ -n "$profile" ]]; then
  [[ -f "$profile" ]] || { echo "GELD_TEST_PROFILE is not a file." >&2; exit 2; }
  cp "$profile" "$app/Contents/embedded.provisionprofile"
fi

cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>GeldBiometryHarness</string>
  <key>CFBundleIdentifier</key><string>${bundle_id}</string>
  <key>CFBundleName</key><string>Geld Biometry Validation Harness</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSUIElement</key><true/>
</dict></plist>
EOF

clang -fobjc-arc \
  "-DGELD_KEYCHAIN_SERVICE=\"${service}\"" \
  "-DGELD_KEYCHAIN_ACCOUNT=\"${account}\"" \
  '-DGELD_KEYCHAIN_REASON="Validate GeldApp biometric Keychain access"' \
  "$repo_root/src-tauri/src/keychain.m" \
  "$repo_root/src-tauri/biometry_harness/main.m" \
  -framework Foundation -framework Security -framework LocalAuthentication \
  -framework CoreFoundation \
  -o "$exe"

sign_args=(--sign "$identity" --identifier "$bundle_id")
if [[ -n "$entitlements" ]]; then
  [[ -f "$entitlements" ]] || { echo "GELD_TEST_ENTITLEMENTS is not a file." >&2; exit 2; }
  sign_args+=(--entitlements "$entitlements")
fi
codesign "${sign_args[@]}" "$app"
codesign --verify --deep --strict --verbose=2 "$app"
entitlements_dump="$(codesign --display --entitlements - --xml "$app" 2>/dev/null || true)"
if [[ -n "$entitlements_dump" ]]; then
  printf 'Effective signed entitlements:\n%s\n' "$entitlements_dump"
else
  echo "No effective signed entitlements found."
fi

cat > "$out_dir/namespace.txt" <<EOF
bundle_id=${bundle_id}
service=${service}
account=${account}
EOF
chmod 600 "$out_dir/namespace.txt"
printf 'Harness prepared (no Keychain operation performed).\nApp: %s\nNamespace record: %s/namespace.txt\n' "$app" "$out_dir"
printf 'Status only: %s status\n' "$exe"
printf 'Interactive operations require explicit acceptance: %s store | load | remove\n' "$exe"
