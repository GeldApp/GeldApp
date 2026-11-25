# Release and recovery procedure

The `Release draft` workflow updates a draft on every push to `main`. The Monday 08:00 UTC run and a manual run from `main` use the same exact-main-SHA checks: frontend, macOS and Windows native checks, both CodeQL analyses on that SHA, no open high/critical CodeQL alerts, the active immutable `v*` tag ruleset, and repeated main-head checks.

Before the first publication, a repository administrator must enable GitHub's **immutable releases** setting in the app repository. The protected tag ruleset alone does not lock published assets; immutable releases lock both the tag and attached files after publication.

For scheduled and manual publication, the workflow builds the app on macOS, sets the npm, Tauri, and Cargo package versions to the Conventional Commit version, creates a universal Apple Silicon/Intel DMG with the configured icons, calculates SHA-256, and attaches both assets to the matching draft before creating the protected tag and publishing. The DMG is ad-hoc signed and unnotarized, with no Apple Developer ID signature. The release workflow does not depend on a `release:published` event.

If a build or upload fails before the tag exists, rerun the manual workflow on `main` with the default `resume_draft: false`. If the tag was already created, a plain manual rerun stops with a recovery message. To resume, manually dispatch on `main` with `resume_draft: true`. This path reruns every exact-SHA and security gate, checks the immutable tag still points to the current main SHA, and requires the release to remain a draft targeted at that SHA before replacing draft assets. It cannot resume an older tagged SHA after `main` advances; do not move or delete tags. Wait for the next calculated version instead. A published release is never eligible for asset replacement.

The tap polls public releases hourly. After publication, its updater validates the versioned DMG and matching checksum, then opens a PR with the exact URL and SHA-256. It dispatches the required cask audit on the PR branch and enables squash auto-merge. The cask can take up to an hour plus CI/merge time to appear; it exists only after this PR merges, so no placeholder versions or unchecked hashes are used.

## Current release preparation status — 2026-11-25

The delivery-preparation branch now aligns the app bundle, Objective-C Keychain bridge, and installation instructions on macOS 13.3; Vite targets Safari 16.4 and Chromium 111. The Homebrew cask declares Ventura and explains that its DSL cannot encode the 13.3 minor-version floor. This branch is preparation only: it has not received final review or hosted checks, and this status does not approve or authorize a release.
