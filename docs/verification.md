# Verification — 2026-11-15

## Desktop runtime floor and bundle targets — 2026-11-25

- Declared macOS minimum: 13.3, aligned across Tauri bundle metadata and the Objective-C bridge deployment target. Vite production JavaScript and CSS targets Safari 16.4 and Chromium 111 (Edge 111).
- Safari 16.4 is available on macOS Ventura 13.3; see [Apple's release notes](https://developer.apple.com/documentation/safari-release-notes/safari-16_4-release-notes) and [Tauri's WebView version mapping](https://v2.tauri.app/reference/webview-versions/). The app deliberately declares Ventura 13.3 as its OS minimum. The oldest supported physical OS/device has not been tested.
- The Windows requirement is an Evergreen WebView2 runtime at Chromium 111 or newer. Tauri's default installer bootstraps a missing runtime; a pinned/offline runtime may not update. Windows runtime behavior has not been tested.
- Source API/CSS inventory: `String.replaceAll`, `Array.prototype.at`, Web Crypto, `TextEncoder`/`TextDecoder`, `:focus-visible`, `backdrop-filter`, and `100dvh`; no source use found of `:has()`, container queries, subgrid, `color-mix()`, or newer Lab/Oklch color functions. Vite targets syntax and CSS minification only; it does not supply API polyfills. This inventory and successful builds do not establish runtime behavior on the declared minimum device.
- The aggregate at `46554675ba38f54d89ac8492583d0c8a07a7181c` passed its hosted required checks, including CodeQL. This later platform-floor follow-up needs its own exact-SHA review and hosted scan.

## Encrypted vault branch verification

- Rust vault tests: 12 passing after provider isolation. Cover setup/read/lock, wrong passphrase, ciphertext tampering, invalid ledger schema, explicit migration with retained plaintext snapshots, explicit recovery with damaged-current preservation, biometric cleanup failure with retryable removal, previous-snapshot rotation failure, failed current replacement, recovery cleanup-failure state, exclusive process ownership, and fresh OS-generated keys/salts/nonces.
- Native: `cargo fmt --check`, `cargo test`, strict Clippy, and `npm run tauri -- build --debug --no-bundle --ci` pass on Apple Silicon macOS. Tests compile with native provider calls disabled.
- Frontend: `npm test` passes (11 tests, including save-before-lock and locked-view cleanup after status refresh failure); `npm run build` passes.
- Vault behavior: native active storage is AES-256-GCM encrypted under a random key wrapped by Argon2id. Browser preview still uses plaintext localStorage, with that limit called out in Settings.
- **Test isolation incident:** an earlier macOS recovery test called the production Keychain delete entry point before this was caught. Its result was ignored by the recovery flow, and we did not inspect Keychain afterward; whether a matching GeldApp item existed or was removed is unknown. The test provider is now disabled under `cfg(test)`, and recovery cleanup has an injected test seam. No further native Keychain calls are planned in tests. Data Protection Keychain requires an appropriate signed app identity/entitlements; x86_64 cross-build and Windows vault runtime/build remain unverified here. Physical biometric behavior needs a user test.
- Not verified: no Keychain item was added by the tests, no Touch ID prompt was invoked, and no signed/notarized macOS build was tested. The earlier deletion attempt's effect is unknown. Do not rely on the Keychain item surviving; if Touch ID unlock was previously enabled for this app, use passphrase recovery and re-enable it only after reviewing this incident. Avoid checking/removing items manually until the owner decides whether to inspect the fixed GeldApp service/account entry.
- Browser UI inspection could not run in this environment: the CUA browser host was unavailable and the default Vite port was already occupied.

The earlier bullets below record the prototype's pre-vault checks from this date. Its plaintext-ledger observation describes that baseline, not this branch's current native storage.

- Frontend: TypeScript and Vite production build pass.
- Unit tests: 8 passing; exact money parsing, IBAN checksum, quoted CSV round-trip, duplicate import/balance handling, invalid dates/schema, authenticated encrypted backup round-trip, wrong passphrase and tamper rejection.
- Dependency audit after upgrading Vitest: 0 reported vulnerabilities.
- Native: Rust compile check passes; debug macOS `.app` bundle builds. Launched via Launch Services; React initialized its demo ledger through native IPC in `~/Library/Application Support/app.geldapp.desktop/ledger.json` (3 accounts, 27 transactions).
- Browser: transaction search narrows to Spotify; a manually entered -€4.50 expense changes Everyday balance from €4,286.50 to €4,282.00; state survives reload. Invalid IBAN is rejected, valid IBAN saves a payment draft without changing balances; draft deletion works.
- Browser layout: overview and transaction pages have no horizontal document overflow at 390px. Navigation remains labelled when collapsed.
- Automated accessibility: zero WCAG A/AA violations on desktop overview, payment drafts, and narrow transaction view. Incomplete checks remain for decorative single characters and a gradient background; automated checks do not establish full accessibility compliance.
- Windows builds, real bank access, signed release installers, native file export dialogs, and physical-device/assistive-technology behavior have not been validated. No real bank or user financial data was used.

The macOS bundle is a development build, not signed/notarized for distribution.
