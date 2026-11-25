# Vault compatibility fixture

`vault-v1-aes-gcm-0.10.json` is a synthetic GeldApp vault envelope generated
with `aes-gcm 0.10.3` and `argon2 0.6.0`, matching the pre-upgrade vault
implementation. It uses passphrase `legacy aes-gcm fixture passphrase`, a
synthetic empty ledger, and deterministic test-only key, salt, and nonce bytes
so the fixture stays reproducible. These constants are never used by
production code. The fixture is decrypted by the current vault unlock path to
verify on-disk v1 compatibility across the crate upgrade.
