# GeldApp

A calmer kind of banking. React + TypeScript + Tauri 2, designed for macOS and Windows, with a browser preview.

## Run

```sh
npm install
npm run dev        # browser at http://localhost:1420
npm run tauri dev  # native desktop window; requires Rust and OS prerequisites
npm run build      # typecheck + frontend production build
npm test
npm run tauri build
```

Tauri prerequisites: <https://v2.tauri.app/start/prerequisites/>. Build macOS installers on macOS; build Windows installers on Windows. Desktop signing and release distribution are not configured.

## Desktop runtime requirements

- macOS 13.3 or newer. The app declares this minimum in Tauri and builds its Objective-C Keychain bridge for the same deployment floor. Safari 16.4 is the web-platform baseline used for production JavaScript and CSS output. This minimum physical OS has not yet been tested.
- Windows requires Microsoft Edge WebView2 Evergreen with Chromium 111 or newer. Tauri's default installer can bootstrap a missing runtime; a network connection is needed for that download. Admin-pinned or offline runtimes can remain older than the required baseline. No Windows runtime validation is claimed.
- Vite targets Safari 16.4 and Chromium 111 for both JavaScript syntax and CSS. This does not polyfill browser APIs. The current source uses standard APIs including `String.replaceAll`, `Array.prototype.at`, Web Crypto, `TextEncoder`/`TextDecoder`, and CSS `:focus-visible`, `backdrop-filter`, and `100dvh`; runtime behavior on the oldest supported OS/device remains unverified.

References: [Apple Safari 16.4 release notes](https://developer.apple.com/documentation/safari-release-notes/safari-16_4-release-notes), [Tauri WebView versions](https://v2.tauri.app/reference/webview-versions/), [Tauri Windows WebView2 setup](https://v2.tauri.app/start/prerequisites/#windows), [Vite build options](https://vite.dev/config/build-options.html).

## Working features

- Overview with account balances, actual monthly income/spending, historical cash-flow chart, budgets, and recent transactions.
- Manual accounts with opening balances; transaction entry adjusts account balance in integer euro cents.
- Transaction search, account/category/month filters, detail editing, CSV import/export, duplicate detection.
- Monthly category budgets and category spending breakdowns.
- IBAN-checksummed payment drafts. No bank connection; no payment execution.
- Native active storage: AES-256-GCM encrypted ledger with a random data key wrapped by an Argon2id passphrase key. Native saves preserve a prior encrypted snapshot; recovery is explicit and passphrase-authenticated.
- Optional macOS Touch ID unlock stores the actual vault key in the Data Protection Keychain with `biometryCurrentSet` access control. Passphrase recovery remains available. Windows exposes biometrics as unsupported.
- Browser preview uses localStorage without encryption. Do not enter personal financial data in the browser preview.
- Explicit legacy migration selects one of `ledger.json`, `ledger.previous.json`, or `ledger.next.json`. It leaves every plaintext copy in place for manual review and cleanup after verifying the vault.
- Portable passphrase-encrypted backup and restore: AES-256-GCM, random 96-bit IV and 128-bit salt, PBKDF2-SHA256 with 600,000 iterations. Authentication failures do not replace the current ledger.
- Keyboard navigation, dialog focus trap, reduced-motion support, responsive layouts.

## Prototype boundaries

Starts with illustrative demo data, not real financial data. Keep using sample data until a signed macOS build and Keychain/Touch ID behavior have been verified, the migration copies have been reviewed, and the security review is complete. The Objective-C Security/LocalAuthentication bridge builds for the selected macOS architecture. A suitable signed app identity and entitlements are required for Data Protection Keychain access; the local development identity and physical Touch ID prompts have not been verified. Windows biometric unlock is not implemented or tested.

The native ledger is encrypted at rest; the browser preview is not. Legacy JSON snapshots remain plaintext after migration until the user verifies the vault and removes them manually. There are no bank connections, provider-password storage, bank API, telemetry, backend, or cloud sync. Encrypted backups remain separate from active-storage encryption. Restore replaces the current workspace after an explicit checkbox. Category edits save immediately.

Account balances are opening balances plus transactions added/imported after account creation. Demo balances are illustrative current balances and demo historical transactions are already reflected in them. For a fresh imported account, choose an opening balance from before the imported period. Imports skip identical account/date/merchant/amount/note tuples; this can collapse legitimate identical purchases, so review CSVs before importing. CSV supports comma delimiters, quoted fields, ISO dates, dot decimals, signed euro amounts, optional category/note. Limit: 10,000 rows and 15 MB per import. Single currency: EUR. Budgets recur monthly and exclude refunds from spending. Internal transfers should use the Transfers category and are excluded from cash-flow totals.

DM Sans and Manrope variable fonts are bundled locally. The UI needs no remote font or image requests.

See [architecture](docs/architecture.md) for vault details and the future incremental sync design.
