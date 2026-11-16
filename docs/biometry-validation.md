# macOS biometric validation

Current code stores the actual 32-byte vault data key in a Data Protection Keychain generic-password item. Its access control is `biometryCurrentSet` plus `WhenUnlockedThisDeviceOnly`; passphrase unlock remains the recovery path. The UI must keep hardware availability separate from signed-app capability.

## Read-only capability diagnostic

`geld_keychain_entitlement_status()` inspects only the running process signature through `SecTaskCopyValueForEntitlement`. It does not call `SecItem*`, probe Keychain contents, or trigger biometric UI. Results:

- `1`: `com.apple.application-identifier` entitlement is present in the effective signature. This is a prerequisite signal, not proof that a Keychain operation succeeds.
- `0`: entitlement is absent. Explain that the current build signature does not expose the app identifier needed for its private Data Protection Keychain access group; keep passphrase unlock available and disable enabling Touch ID.
- `-1`: inspection failed. Report capability as unknown and keep passphrase unlock available.

Apple describes the Data Protection Keychain access groups as derived from code-signing entitlements. On macOS the standard app identifier entitlement is `com.apple.application-identifier`; signing and provisioning requirements depend on the claimed entitlements. Do not claim that every developer needs a paid account. Inspect the effective built app with `codesign -d --entitlements :- /path/to/GeldApp.app` and `codesign -dv --verbose=4 /path/to/GeldApp.app`. This diagnostic intentionally does not assume that hardware availability implies app capability.

For a compact report that omits provisioning profile contents, use `scripts/inspect-macos-signing.sh /path/to/GeldApp.app`. It reads the effective signature and reports whether an embedded profile exists; it never queries or changes Keychain state.

Observed on the current local `GeldApp.app` debug bundle: ad-hoc linker signature, no TeamIdentifier, no effective entitlement blob, and no embedded provisioning profile. This is signing metadata only; the running app was not opened and Keychain was not queried. The `SecTaskCopyValueForEntitlement` API distinguishes absent entitlement (no error) from inspection failure (error), so the runtime status can report `false` for this build while preserving `null` for unknown.

## Preparing a unique interactive harness

The harness compiles the same `keychain.m` implementation as production with a newly generated service and account on each preparation. Its constants cannot name `app.geldapp.desktop` / `vault-key-v1`. The bundle ID defaults to a separate validation ID. The `status` command is read-only. `store`, `load`, and `remove` are explicit commands; `store` and `remove` require typed confirmation. The stored value is a random synthetic key, never a vault key.

After independent review, prepare on macOS:

```sh
scripts/prepare-biometry-harness.sh
```

The default uses an ad-hoc signature. A specific installed identity can be selected with `GELD_TEST_CODESIGN_IDENTITY`; an actual profile and matching app-signing entitlements can be supplied through `GELD_TEST_PROFILE` and `GELD_TEST_ENTITLEMENTS`. Set `GELD_TEST_BUNDLE_ID` only to an identifier for which that operator-provided identity/profile is valid. Never insert a fabricated team ID or reuse the production service/account. The script only compiles, signs, verifies, and prints effective entitlements. It does not perform Keychain operations. Record the emitted namespace path and retain it only for this run.

Then, after confirming the exact namespace from `namespace.txt`, run the generated app executable:

```sh
/tmp/geld-biometry.XXXXXX/GeldBiometryHarness.app/Contents/MacOS/GeldBiometryHarness status
/tmp/geld-biometry.XXXXXX/GeldBiometryHarness.app/Contents/MacOS/GeldBiometryHarness store
```

For persistence, close the process after `store`, relaunch the same executable with `load`, and compare the printed SHA-256 digests. Cancel the biometric prompt and confirm access fails. Test with biometrics unavailable; remove/change enrolled fingerprints and confirm the protected key stops loading; restore the original enrollment only if desired. Finish with `remove` using this same executable and verify a subsequent `load` fails. Never run harness commands against the production GeldApp service/account.

## Acceptance record

- [ ] Effective app entitlements inspected; signing identity and profile availability recorded without copying secrets.
- [ ] Harness namespace recorded and checked to differ from production constants.
- [ ] Store with synthetic key succeeded; digest recorded.
- [ ] Relaunched process loaded identical digest after biometric success.
- [ ] Prompt cancellation failed closed.
- [ ] Unavailable biometrics failed closed.
- [ ] Enrollment change invalidated access to the stored key.
- [ ] Passphrase unlock still worked independently in GeldApp.
- [ ] Harness namespace item removed; repeat load failed.
- [ ] Physical device/macOS version and exact app signature recorded.

No box is complete from compilation or source inspection alone. Until the interactive checks pass with a reviewed build, SEC-03 stays partial and physical biometric behavior stays unverified.
