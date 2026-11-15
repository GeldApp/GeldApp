# Verification — 2026-11-15

## Encrypted vault branch verification

- Rust vault tests: 8 passing. Cover setup/read/lock, wrong passphrase, ciphertext tampering, invalid ledger schema, explicit migration with retained plaintext snapshots, explicit recovery with damaged-current preservation, and exclusive process ownership.
- Native: `cargo fmt --check`, `cargo test`, and `cargo check` pass on Apple Silicon macOS. The Objective-C Security/LocalAuthentication bridge compiles and links into the test binary.
- Frontend: `npm test` passes (10 tests, including save-before-lock and locked-view cleanup after status refresh failure); `npm run build` passes.
- Vault behavior: native active storage is AES-256-GCM encrypted under a random key wrapped by Argon2id. Browser preview still uses plaintext localStorage, with that limit called out in Settings.
- Not verified: no live user Keychain item was added or removed, no Touch ID prompt was invoked, and no signed/notarized macOS build was tested. Data Protection Keychain requires an appropriate signed app identity/entitlements; x86_64 cross-build and Windows vault runtime/build remain unverified here. Physical biometric behavior needs a user test.
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
