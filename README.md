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

## Working features

- Overview with account balances, actual monthly income/spending, historical cash-flow chart, budgets, and recent transactions.
- Manual accounts with opening balances; transaction entry adjusts account balance in integer euro cents.
- Transaction search, account/category/month filters, detail editing, CSV import/export, duplicate detection.
- Monthly category budgets and category spending breakdowns.
- IBAN-checksummed payment drafts. No bank connection; no payment execution.
- Local persistence: browser localStorage or app-data JSON via narrowly scoped Tauri commands. Native writes preserve a previous snapshot.
- Portable passphrase-encrypted backup and restore: AES-256-GCM, random 96-bit IV and 128-bit salt, PBKDF2-SHA256 with 600,000 iterations. Authentication failures do not replace the current ledger.
- Keyboard navigation, dialog focus trap, reduced-motion support, responsive layouts.

## Prototype boundaries

Starts with illustrative demo data, not real financial data. Working storage is **not yet encrypted**. Encrypted backups do not encrypt the active ledger. Use sample data until the encrypted vault is implemented. Restore replaces the workspace and requires an explicit checkbox. No credentials, bank API, telemetry, backend, or cloud sync is included. Category edits save immediately.

Account balances are opening balances plus transactions added/imported after account creation. Demo balances are illustrative current balances and demo historical transactions are already reflected in them. For a fresh imported account, choose an opening balance from before the imported period. Imports skip identical account/date/merchant/amount/note tuples; this can collapse legitimate identical purchases, so review CSVs before importing. CSV supports comma delimiters, quoted fields, ISO dates, dot decimals, signed euro amounts, optional category/note. Limit: 10,000 rows and 15 MB per import. Single currency: EUR. Budgets recur monthly and exclude refunds from spending. Internal transfers should use the Transfers category and are excluded from cash-flow totals.

DM Sans and Manrope variable fonts are bundled locally. The UI needs no remote font or image requests.

See [architecture](docs/architecture.md) for the future encrypted vault and incremental sync design.
