# Local-first now, encrypted sync later

## Current implementation

React owns a versioned ledger using integer euro cents. A small repository adapter selects browser localStorage or Tauri IPC. Rust writes only a fixed app-data path; it does not accept arbitrary paths or expose filesystem/network plugins. Saves are serialized; errors are surfaced. Frontend CSP restricts production scripts and connections. Demo rows and payment drafts never contact banks.

This is a functional product prototype. It is not production banking software: local active storage is plaintext; data volumes are bounded by a single JSON snapshot; no banking transport, secure key custody, automatic rules, split transactions, statements, multi-currency valuation, or cloud service exists yet. An encrypted export is implemented independently of that future sync.

## Proposed vault

Introduce a LedgerRepository interface (query, append, transact, checkpoint) rather than growing the snapshot format. Desktop: SQLite behind Rust, with an audited encryption approach and OS keychain-wrapped random vault key. Browser: encrypted IndexedDB records plus an in-memory unwrapped CryptoKey. Lock must clear decrypted state and keys; idle locking and process lifecycle need platform tests. A passphrase derives a key-encryption key; it wraps a random vault key rather than directly encrypting every record. Version KDF parameters. Plan recovery and rotation before importing bank credentials. Never store a raw vault key in localStorage, logs, URLs, analytics, or the blob server.

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
