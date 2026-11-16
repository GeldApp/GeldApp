# Local-first now, encrypted sync later

## Current implementation

React owns a versioned ledger using integer euro cents. Browser preview uses localStorage and is explicitly unencrypted. The Tauri adapter sends ledger JSON only to native commands. Rust stores one encrypted snapshot in the fixed app-data directory; it does not accept arbitrary paths or expose filesystem/network plugins. Saves are serialized in the app and protected by a cross-process exclusive vault lock. Demo rows and payment drafts never contact banks.

The native active ledger uses a random 256-bit data key with AES-256-GCM. An Argon2id-derived key wraps the data key, with KDF parameters versioned in the vault file. Rust keeps the unlocked data key in a zeroizing process session; passphrase command strings are zeroized after use. Writes stage and sync new ciphertext, stage and sync a copy of the current ciphertext, atomically replace the previous snapshot, then rename the new snapshot over the current file. The current file stays in place if either replacement fails. [`std::fs::rename`](https://doc.rust-lang.org/stable/std/fs/fn.rename.html) replaces an existing destination where supported; Windows behavior depends on OS/filesystem support. Invalid or missing current data is never silently replaced. Previous encrypted snapshots have an explicit passphrase-authenticated recovery flow that preserves the current file as a rejected copy. Ledger schema and account/date references are validated before encryption and after decryption.

On macOS, optional biometric unlocking stores the actual data key in the Data Protection Keychain. A native `SecAccessControl` item uses `biometryCurrentSet` and `WhenUnlockedThisDeviceOnly`; a fresh LocalAuthentication context requests access with zero Touch ID reuse. Enrollment changes, cancellation and other Keychain errors fail closed; passphrase recovery remains available. A duplicate Keychain item with an unknown access policy is rejected rather than updated. Windows currently uses the encrypted vault and passphrase; biometric unlock is reported as unsupported. Browser preview remains usable but stores plaintext in localStorage.

Legacy `ledger.json`, `ledger.previous.json` and `ledger.next.json` migration requires explicit snapshot selection. Migration retains every plaintext copy for the user to review and remove manually after verifying the encrypted vault. The encrypted format is one bounded JSON snapshot, not SQLite or incremental storage. Provider passwords and bank connections do not exist in this prototype. No idle locking, OS sleep/session lock handling, vault key rotation, Windows credential protection, or cloud service exists yet.

Data Protection Keychain behavior requires a suitably signed macOS app identity and supported entitlements. This branch has build and isolated cryptographic tests, but has not written to a real user Keychain or exercised biometric prompts; physical Touch ID and signed-release behavior remain unverified. macOS x86_64 cross-compilation and Windows runtime validation also remain open.

## Vault follow-up

Move from the bounded encrypted JSON snapshot to a transactional native repository only after measuring the next required query and recovery behavior. Keep the current envelope as a passphrase-recoverable data key; never store the key in localStorage, logs, URLs, analytics, or the blob server. Before adding provider passwords, define a separate native credential boundary and explicit save/forget flows. Test a signed macOS app identity and Data Protection Keychain entitlements before making release claims. Add idle/sleep locking and platform lifecycle tests. Browser storage needs a separate encrypted IndexedDB design before it can hold real financial data.

BYOK needs a precise product definition: user-generated 256-bit recovery key, passphrase-wrapped vault key, or an external key manager. A key that decrypts finance data is distinct from an AI provider API key. This project assumes user-controlled vault encryption keys. Browser compromise/XSS can read unlocked data; encryption cannot protect an actively compromised client.

## Dumb blob server / incremental loading

Avoid one giant encrypted blob: every transaction would force full re-download and re-encryption. Use immutable encrypted chunks plus an encrypted manifest:

- Random opaque vault and chunk identifiers; no plaintext merchants, dates, accounts or category names in server object names.
- Separate chunks for account metadata, category/rule data, and bounded transaction history (e.g. per account/month, with size-based splitting). The encrypted manifest maps query ranges to chunk IDs. Loading overview fetches manifest + summary chunks, then history pages on demand.
- Independently encrypt every chunk with audited AEAD, a fresh nonce, and authenticated associated data binding vault ID, schema version, chunk ID and key epoch. Do not reuse the export helper as a sync protocol without review.
- Put bounded encrypted summaries/indexes in separate chunks for overview and search. Search either fetches/decrypts relevant history or builds a local index; never send plaintext search terms to the store.
- Server supports authenticated opaque GET/PUT, immutable objects, quotas, and compare-and-swap on a small manifest head (ETag/If-Match). It knows blob size, access timing and account identity, so metadata confidentiality is incomplete.
- Commit chunks first, then atomically advance manifest head. A failed CAS requires fetching the latest head, merging idempotent operations, rebuilding touched chunks and retrying. No silent last-writer-wins over an entire ledger.
- Assign stable transaction IDs and operation IDs; model tombstones and versioned category/note edits. Money movements need deterministic deduplication and explicit conflict handling. Retain acknowledgments before garbage collection.
- Maintain hash-linked manifest generations and locally remembered sequence numbers for rollback detection. Cross-device freshness and server fork detection require an explicit trust/recovery protocol; authenticated encryption alone cannot detect all replays.
- Rotation wraps/re-encrypts keys and advances key epoch; revocation cannot erase previously downloaded plaintext or keys.

## Delivery steps

1. Replace demo persistence with encrypted transactional local repository and explicit create/unlock/lock flows; verify crash recovery and migrations.
2. Add connector boundary: FinTS/HBCI or regulated banking provider, consent/SCA, rate limits, bank-specific IDs and read-only sync first. Keep payment authorization separate from drafts.
3. Add rules, splits, reconciliation and statement storage, plus fixture-based accounting tests.
4. Prototype encrypted chunk protocol with two clients: offline edits, interrupted upload, CAS conflicts, key rotation, rollback and lost-key recovery.
5. Ship a web client only after threat-model review, CSP/font hardening, key-custody tests and independent cryptography review.
