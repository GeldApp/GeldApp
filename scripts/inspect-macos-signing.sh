#!/bin/bash
set -euo pipefail

if [[ $# -ne 1 || ! -d "$1" ]]; then
  echo "Usage: $0 /path/to/GeldApp.app" >&2
  exit 2
fi
app="$1"
echo "Signature metadata (no Keychain access):"
codesign -dv --verbose=4 "$app" 2>&1 | rg '^(Identifier|Format|CodeDirectory|Signature|Authority|TeamIdentifier|Sealed Resources)' || true
echo
echo "Effective signed entitlements:"
entitlements_output="$(codesign --display --entitlements - --xml "$app" 2>&1 || true)"
if [[ "$entitlements_output" == *"<plist"* ]]; then
  printf '%s\n' "$entitlements_output"
else
  echo "No readable entitlement blob was found."
fi
if [[ -f "$app/Contents/embedded.provisionprofile" ]]; then
  echo
  echo "Embedded provisioning profile: present (contents not dumped)."
else
  echo
  echo "Embedded provisioning profile: absent."
fi
