# Verification — 2026-11-15

- Frontend: TypeScript and Vite production build pass.
- Unit tests: 8 passing; exact money parsing, IBAN checksum, quoted CSV round-trip, duplicate import/balance handling, invalid dates/schema, authenticated encrypted backup round-trip, wrong passphrase and tamper rejection.
- Dependency audit after upgrading Vitest: 0 reported vulnerabilities.
- Native: Rust compile check passes; debug macOS `.app` bundle builds. Launched via Launch Services; React initialized its demo ledger through native IPC in `~/Library/Application Support/app.geldapp.desktop/ledger.json` (3 accounts, 27 transactions).
- Browser: transaction search narrows to Spotify; a manually entered -€4.50 expense changes Everyday balance from €4,286.50 to €4,282.00; state survives reload. Invalid IBAN is rejected, valid IBAN saves a payment draft without changing balances; draft deletion works.
- Browser layout: overview and transaction pages have no horizontal document overflow at 390px. Navigation remains labelled when collapsed.
- Automated accessibility: zero WCAG A/AA violations on desktop overview, payment drafts, and narrow transaction view. Incomplete checks remain for decorative single characters and a gradient background; automated checks do not establish full accessibility compliance.
- Windows builds, real bank access, signed release installers, native file export dialogs, and physical-device/assistive-technology behavior have not been validated. No real bank or user financial data was used.

The macOS bundle is a development build, not signed/notarized for distribution.
