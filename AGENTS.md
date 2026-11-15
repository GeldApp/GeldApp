# GeldApp working agreement

## Communication

Keep updates brief, concrete, and complete. Explain limits plainly.

## Ownership and delegation

- The primary agent owns product direction, scope, integration, and release decisions.
- Delegate implementation and coding tasks to `gpt-6-luna` agents in isolated Git worktrees. Use one branch per task, under `.worktrees/`.
- Use a separate `gpt-6.1-terra` agent for adversarial review. The reviewer must not author the implementation it reviews.
- Workers make small, meaningful commits on their task branch. They must not merge to `main`, push, or modify another worker's files.
- Before merging to `main`, require an explicit Terra verdict on the final commit(s), resolution of all blocking findings, and relevant checks. Record reviewed SHA, verdict, fixes, and validation in `docs/reviews/`.
- The primary agent alone merges approved PRs. Keep regular Conventional Commits on task branches; squash-merge approved PRs to `main` using a Conventional Commit PR title. Require passing lint/build/test/CodeQL and commit/title checks where the GitHub plan supports enforcement. Never bypass a failing required check or force-push.
- Every commit to `main` updates or creates a draft release with its version derived from Conventional Commits. Prefer standard/community actions over custom scripts. Publication is a separate action. Record plan/permission limitations honestly; never weaken repository visibility to bypass a plan limit.
- Preserve dirty work and credentials. Never commit local financial data, secrets, build outputs, agent scratch files, or worktrees.

## Documentation and checks

- Keep `TODO.md` current. Distinguish implemented, partial, planned, and unverified behavior. Link MoneyMoney parity claims to official sources.
- Fetch current library/platform documentation through Context7 when implementing library-specific APIs. Resolve the library ID first, then query its docs. For Apple APIs not indexed there, use Apple's primary documentation.
- Run `npm run build` and `npm test` for frontend changes. Run relevant Rust tests, formatting, and compilation for native changes.
- Check actual browser or native behavior when practical. Compilation cannot establish Touch ID or Keychain behavior.
- Native authentication must protect the real encryption key; a cosmetic fingerprint prompt is insufficient. Cancellation, enrollment changes, unavailable hardware, and failed authentication must fail closed while preserving passphrase recovery.
- Do not claim Windows or physical biometric validation without testing it.

## Current product direction

Professional, visually strong banking app with its own identity. React + Tauri; macOS and Windows first. Local-first with encrypted storage. Future web companion: user-controlled keys and a server storing opaque encrypted chunks, with incremental loading. Actual bank payment authorization remains separate from draft preparation.
