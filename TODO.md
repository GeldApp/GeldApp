# GeldApp product backlog

Baseline checked: **2026-11-15**. Goal: practical MoneyMoney feature parity with GeldApp's own professional identity, then encrypted cross-device sync. This is a delivery plan, not a claim that bank connectivity or security has shipped.

`[x]` implemented prototype behavior; `[ ]` incomplete. **Partial** means a usable subset exists. **In progress** is not merged or approved. P0 blocks real-data use; P1 makes a useful everyday banking app; P2 closes broader parity; P3 extends GeldApp. Every delivery needs a Luna implementation worktree, relevant checks, an independent Terra review of the final SHA, and primary-agent integration.

## Current working prototype

- [x] React/TypeScript frontend, Tauri desktop shell, bundled fonts and original vector icon.
- [x] Manual current, savings and cash accounts; integer-cent EUR amounts.
- [x] Search and filter transactions by account, category and month; edit notes/categories.
- [x] Manual transaction entry; CSV import/export with basic duplicate detection.
- [x] Monthly category budgets, spending breakdown and cash-flow chart.
- [x] IBAN checksum validation and payment drafts; **no payment execution**.
- [x] Passphrase-encrypted backup/restore, separate from the active ledger.
- [x] macOS development bundle builds and launches; limited native persistence verified.
- [ ] Windows native build and runtime verification; code portability alone is insufficient.

## P0 — Trustworthy local foundation

Reference: [MoneyMoney security](https://moneymoney.app/security/). These are GeldApp requirements and acceptance conditions, not an assertion of identical internals.

- [x] **SEC-01 · Implemented; prototype reviewed** — Active native ledger uses versioned AES-256-GCM encryption, Argon2id-wrapped random keys, durable snapshots and corruption detection. Browser preview remains unencrypted. macOS/Windows runtime data-path validation is still required.
- [x] **SEC-02 · Implemented; prototype reviewed** — Native create/unlock/lock/recovery flows; wrong-key, tamper, schema, migration, locking and save-before-lock tests. Vault keys stay in native memory and are zeroized on lock.
- [ ] **SEC-03 · Partial** — Optional macOS Keychain item protects the vault key with current-biometry access control. No bank-provider passwords exist or are stored yet; a separate credential boundary is future work. Signed app entitlement and real Keychain behavior need device validation.
- [ ] **SEC-04 · Partial** — Startup Touch ID uses native access control on the actual vault key; cancel/unavailable/enrollment changes fail closed and passphrase recovery remains. Physical Touch ID and signed-app testing remain outstanding.
- [ ] **SEC-05** — Auto-lock after idle, sleep and OS session lock; clear decrypted frontend state and native key material. Test outstanding saves during lock.
- [x] **SEC-06 · Implemented; prototype reviewed** — Explicit migration validates the full ledger and encrypts the selected legacy snapshot; `ledger.json`, `ledger.previous.json` and `ledger.next.json` remain untouched for manual review/cleanup. Authenticated prior-vault recovery preserves damaged evidence. Copies in backups cannot be erased remotely.
- [ ] **SEC-07** — Windows credential protection and optional Windows Hello, with equivalent recovery semantics and platform tests.
- [ ] **SEC-08** — Safe exports: neutralize spreadsheet formulas in merchant/note cells while preserving signed numeric amounts. Untrusted-file size/schema limits and migration tests.
- [ ] **REL-01** — Signed/notarized macOS distribution, signed Windows installer, verified updates and rollback policy.
- [ ] **REL-02 · Partial** — Baseline frontend/Rust checks and native compilation passed GitHub CI on macOS and Windows ([run](https://github.com/GeldApp/GeldApp/actions/runs/43739038652)). Expanded lint, CodeQL, Conventional Commit and draft-release workflows are in progress; this run predates the vault implementation. Keep financial fixtures synthetic.

Release gate: encrypted storage and recovery verified, critical/high review findings closed, platform-specific claims backed by actual execution.

## P1 — Everyday banking

References: [account types and banking features](https://moneymoney.app/), [balances, history and pending entries](https://moneymoney.app/help/).

- [ ] **BANK-01** — Connector interface with supported capabilities, typed errors, retries, cancellation and synthetic connector fixtures.
- [ ] **BANK-02** — FinTS/HBCI account discovery and read-only balances/history. Publish a tested bank matrix instead of a blanket compatibility claim.
- [ ] **BANK-03** — TAN/SCA challenges, decoupled approval, expired consent and user cancellation. Keep provider secrets native.
- [ ] **BANK-04** — Optional PSD2 provider integration: explicit consent, renewal/revocation, account access scope and transparent provider data flow.
- [ ] **BANK-05** — Bank-specific API connectors for unsupported accounts; credentials saved only on request. Start with a small documented bank set.
- [ ] **BANK-06** — Scheduled/background refresh, last successful sync, per-account errors, complete-history fetch and provider date-range limitations.
- [ ] **LEDGER-01 · Partial** — Bank-provided balance separate from calculated ledger balance; opening-balance calibration and reconciliation.
- [ ] **LEDGER-02** — Pending/booked lifecycle, booking/value dates, original provider IDs and idempotent repeated sync. Do not merge legitimate identical purchases.
- [ ] **LEDGER-03 · Partial** — Add/edit/archive accounts, account groups, custom icons, configurable ordering, fresh empty workspace and isolated demo workspace.
- [ ] **LEDGER-04** — Credit cards, deposits, loans, building-society accounts, PayPal and foreign currencies. Account-specific metadata and currency-aware totals.
- [ ] **TX-01 · Partial** — Date-range and multi-account search, counterparty/reference/amount predicates, quick same-counterparty search and running/filter totals.
- [ ] **TX-02** — Split transactions with exact parent/child sum invariants, transfers linked across own accounts and no double-counted cash flow.
- [ ] **TX-03** — Completed/reconciled flags, bulk selection/edit, attachments and undo for destructive edits.
- [ ] **DOC-01** — Download, view and archive electronic PDF statements; manual PDF import and duplicate detection. Explicit document encryption/export policy.

Gate: at least one real bank passes read-only end-to-end sync, SCA, re-sync, history-gap and pending-to-booked tests without duplication or balance drift.

## P1 — Categorization and useful analysis

References: [rules syntax and application behavior](https://moneymoney.app/categorization/), [analysis features](https://moneymoney.app/).

- [ ] **RULE-01** — Editable categories/groups, uncategorized inbox, ordering and rule priority.
- [ ] **RULE-02** — Automatic classification on new imports/sync; fields for name, purpose, account, currency, reference, mandate, creditor ID, note and booking text.
- [ ] **RULE-03** — Boolean conditions, phrases, wildcards, amount comparisons and bounded regex. Preview matches before applying.
- [ ] **RULE-04** — Apply changed rules to uncategorized or all entries explicitly; preserve manual overrides by default and offer undo.
- [ ] **REPORT-01 · Partial** — Month/quarter/year views, category/account-group comparisons, income/expense totals and configurable exclusions.
- [ ] **REPORT-02 · Partial** — Category-group budgets, refund treatment, historical budgets and clear overspend states.
- [ ] **REPORT-03** — Historical balances, cash-flow trends and securities holdings/valuation. Show stale prices, currencies and data coverage.

Gate: fixtures demonstrate correct results for splits, refunds, transfers, rule ordering and period boundaries.

## P2 — Payments and business parity

References: [payment operations](https://moneymoney.app/), [templates, outbox and mandates](https://moneymoney.app/help/).

- [ ] **PAY-01 · Partial** — Editable draft outbox, recipient/address book, reusable templates and create-from-transaction.
- [ ] **PAY-02** — Ordinary SEPA and own-account transfers with review, recipient verification, SCA, submission/status reconciliation and duplicate-send protection.
- [ ] **PAY-03** — Instant and scheduled payments; fetch/create/edit/cancel standing orders where supported.
- [ ] **PAY-04** — Batch payments with per-item validation and submission state; one authorization where the bank permits it.
- [ ] **PAY-05** — International transfers with supported currencies, fees and capability checks.
- [ ] **PAY-06** — Parse copied bank details into a draft; user reviews extracted data. Template placeholders and payment history.
- [ ] **PAY-07** — SEPA core/B2B direct debit, mandate management/groups, single/batch collection, outbox and scheduled collection updates.
- [ ] **PAY-08** — Import payment templates/mandates from CSV/SEPA XML; SRZ approvals and electronic-invoice draft handoff where supported.
- [ ] **AUTH-01** — Broader TAN methods, QR/photo/flicker challenges and hardware reader support. Scope USB/Bluetooth/HBCI cards/key files by platform and tested hardware.

Gate: payment transport remains disabled until authorization, retries, ambiguous submission, cancellation and bank reconciliation have dedicated tests and review.

## P2 — Interoperability and desktop quality

References: [import/export formats](https://moneymoney.app/), [import details](https://moneymoney.app/help/), [extensions and automation](https://moneymoney.app/extensions/).

- [ ] **IO-01 · Partial** — CSV mapping preview, German decimals/delimiters, duplicate review and rollback; QIF, OFX/QFX, CAMT, MT940 and tab-separated imports.
- [ ] **IO-02 · Partial** — Export CSV, Excel, Numbers-compatible tables, OFX, CAMT and MT940; preserve source/reference fields and split semantics.
- [ ] **IO-03** — Documented automation API and macOS scripting integration; controlled data handoff to bookkeeping/receipt tools.
- [ ] **EXT-01** — Sandboxed connector/import/export extension boundary, explicit permissions, versioning and trust/revocation policy.
- [x] **UX-01** — Professional theme, distinctive editable SVG logo/wordmark, desktop icons, design tokens and readable dense tables. Luna implementation and Terra review are recorded in `docs/reviews/2026-11-15-brand.md`.
- [ ] **UX-02** — System light/dark appearance, DE/EN translation, locale-aware date/amount entry, currency formatting and keyboard shortcuts.
- [ ] **UX-03** — Native menus/file dialogs, drag/drop imports, accessible focus and screen-reader flows, persisted window/table preferences.
- [ ] **DATA-01** — Automated encrypted backups, retention, restore verification and schema upgrades; large-history pagination and indexing.

## P3 — GeldApp beyond parity

These are our roadmap, separate from MoneyMoney feature-parity claims. See [architecture](docs/architecture.md).

- [ ] **SYNC-01** — User-controlled encryption keys; separately design recovery, rotation, revocation and key epochs.
- [ ] **SYNC-02** — Opaque encrypted chunks plus encrypted manifest/index; incremental history loading without plaintext server queries.
- [ ] **SYNC-03** — Compare-and-swap manifest updates, offline merges, operation IDs, tombstones and bounded garbage collection.
- [ ] **SYNC-04** — Rollback/fork threat model, quotas and metadata-leakage disclosure; two-client conflict/interrupted-upload tests.
- [ ] **WEB-01** — Web client with encrypted IndexedDB and in-memory unlocked keys. Independent threat-model/cryptography review before real-data release.
- [ ] **MOBILE-01** — Consider a phone companion after desktop reliability. Do not present it as shipped parity.

## Delivery automation requested 2026-11-15

- [x] Remote repository configured and reviewed prototype/brand/quality history pushed.
- [x] Repository merge settings: squash only, resulting title from PR title, body from PR body.
- [x] `v0.1.0` prototype baseline tag; no release published.
- [x] **CI-01** — Frontend lint/build/tests and native formatting/Clippy/build/tests run in required macOS and Windows CI. Workflow syntax was validated with actionlint; existing lint warnings remain visible.
- [x] **CI-02** — Required CodeQL analyses for JavaScript/TypeScript and Rust, plus the findings gate. Live checks passed; three vault findings were addressed through direct OS CSPRNG generation rather than suppressed.
- [x] **CI-03** — Required Conventional PR-title and commit checks. Main receives Conventional Commit squash titles.
- [x] **CI-04 · Configured** — Standard/community actions update the implied next-version draft on main commits. Feature/breaking changes select minor/major; ordinary commits share the next patch draft until a release tag. Monday 08:00 UTC publication is gated on an exact checked SHA and immutable tag; no empty releases. Draft updates were observed live; the first scheduled publication has not yet been observed.
- [x] **CI-05** — Required PR checks and branch enforcement are active in GitHub ruleset `27324053`: strict status checks, PR-only squash merges, Conventional Commit metadata, no force/deletion/bypass, and CodeQL findings thresholds. The repository became public during setup, resolving the original private-plan HTTP 403 errors. CI PR #5 passed all eight checks before it was merged externally.
