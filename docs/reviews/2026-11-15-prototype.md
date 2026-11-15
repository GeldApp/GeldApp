# Prototype review — 2026-11-15

Reviewed commit: `c09ce02e40ef04c37ec6e01a0bdce105ce40f61e`.
Verdict: **approve as a sample-data demo only**. This is not approval for use with real financial data, a secure vault, or release as banking software.

The native and browser active ledgers are plaintext, as explicitly disclosed in README and the UI. There is no biometric authentication or bank connection in this baseline. Encrypted backup uses authenticated encryption, but does not protect the active ledger. The source review did not establish physical Touch ID, Keychain, Windows, crash recovery, or real bank behavior.

Nonblocking for the explicitly limited demo, but must be resolved before real-data use:

- `src/domain.ts:41-43`: CSV quoting escapes delimiters, not spreadsheet formulas. A merchant or note such as `=1+1` is exported as a formula cell and may execute when opened in spreadsheet software. Neutralize formula-capable text or make the export boundary explicit and safe.
- `src-tauri/src/lib.rs:37-54`: active data, temporary writes, and previous snapshots are plaintext. Vault migration must account for all three files, interrupted writes, verified durable encrypted recovery, and any older copies; do not claim migration protects data while plaintext remnants survive.
- `TODO.md` is missing despite the working agreement requiring implementation status and unverified behavior to be tracked.

Validation evidence: reviewed source and project verification record. The record reports frontend build and 8 tests passing, a macOS debug bundle and limited browser checks; those checks were not independently rerun for this review. No physical biometric or Windows validation was claimed.
