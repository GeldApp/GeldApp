# CI and release policy

## Checks and merge policy

`Checks` runs on pull requests targeting `main` and on pushes to `main`:

| Required check name | What it runs |
| --- | --- |
| `Frontend` | Biome lint, TypeScript/Vite build, Vitest |
| `Native (macOS)` | Frontend build, Rust formatting, Clippy, Rust tests/check, Tauri debug compile |
| `Native (Windows)` | Same native checks on Windows |
| `PR title` | Conventional Commit title validation |
| `Commit messages` | Commitlint on every commit in the pull request |
| `Analyze (javascript-typescript)` | CodeQL JavaScript and TypeScript queries |
| `Analyze (rust)` | CodeQL Rust queries |

The active `main` ruleset requires pull requests, strict passing checks,
Conventional Commit subjects, no force pushes or branch deletion, and CodeQL
results with a High-or-higher security alert threshold. It has no bypass actors.
Repository settings allow squash merges only, using the PR title as the squash
commit subject and the PR body as its message.

The app uses `@biomejs/biome` with recommended lint rules and React's
recommended domain. `npm run lint` only lints; it does not reformat files.
Native jobs run Clippy with warnings denied. The PR policy accepts Conventional
Commit types `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`,
`ci`, `chore`, and `revert`. Scope is optional. Breaking changes use
`type(scope)!: ...` and/or a `BREAKING CHANGE:` footer. Both PR titles and every
commit in the PR are checked. The title job also runs when a title is edited.
Both policy checks use `pull_request` with read-only permissions, including on
fork PRs. Commitlint ignores generated Git merge commits while checking commits
that contain actual changes.

## CodeQL and fork behavior

CodeQL has separate JavaScript/TypeScript and Rust jobs. Rust uses CodeQL's
`none` build mode; the normal native CI jobs still compile and test the app.
The repository is public, code scanning is available, and the active ruleset
requires CodeQL results and blocks High-or-higher security findings. Earlier
403 responses while the repository was private are historical.

Fork PRs use the unprivileged `pull_request` event and read-only token. GitHub
allows SARIF uploads from `pull_request` runs with read-only tokens. No workflow
checks out or executes untrusted code from `pull_request_target`.

The PR-only `Code scanning results / CodeQL` gate is not a check run on main
pushes. Weekly publication instead requires the two successful CodeQL analysis
check runs on the exact main SHA, CodeQL analysis records for that SHA and both
language categories with empty analysis errors, and no open High or Critical
CodeQL alerts for `main`. Missing records, API errors, stale SHAs, or any
matching alert fail the scheduled run closed.

## Draft and weekly release behavior

Every push to `main` updates the GitHub draft release with `publish: false`.
The annotated `v0.1.0` tag on the approved prototype commit is the initial
version baseline. The published `v0.1.1` release is the current version baseline.
The workflow fetches all tags and uses the pinned
[`paulhatch/semantic-version` action](https://github.com/PaulHatch/semantic-version)
to calculate the next version from commits after the latest `v*` tag:

- `feat:` and `feat(scope):` raise the minor version.
- `type(scope)!:` or a `BREAKING CHANGE:` / `BREAKING-CHANGE:` footer raises the
  major version. At `0.x`, a breaking change produces `1.0.0`.
- Other commit types, including `fix`, `docs`, and `chore`, use the default
  patch increment. With `bump_each_commit: false`, ordinary commits share one
  implied next patch version until a release is tagged. A later feature or
  breaking commit can raise the draft's implied version for the whole
  unreleased range. Every main push still updates the draft.

The pinned `release-drafter` action creates or updates the draft targeted at the
exact main commit. Push-triggered draft updates do not create tags.

Each Monday at 08:00 UTC, the same serialized workflow checks the latest main
SHA before and during the release operation. The scheduled run waits for
successful `Frontend`, `Native (macOS)`, `Native (Windows)`,
`Analyze (javascript-typescript)`, and `Analyze (rust)` checks on that exact SHA.
It then validates CodeQL analysis records and checks the alert API for open
Critical and High alerts. If any check or security result is missing, pending,
failed, stale, malformed, or unavailable, publication stops. After checking
that the scheduled SHA is still main, the run refreshes the draft, creates the
calculated version tag at that exact SHA, verifies the tag target, and publishes
the release. A conflicting/preexisting version tag makes tag creation fail and
blocks publication. If the scheduled SHA already has its calculated version
tag, the run skips publication.

Draft updates and scheduled publication share one concurrency group. GitHub
keeps the newest pending run when several main pushes queue, so the draft
converges to latest main. Each schedule run publishes its frozen, validated
`github.sha` snapshot. A main-head recheck before tag creation aborts if main
advanced during the checks; a later scheduled run can publish the new head
after its checks pass. Feature branches and pull requests do not create drafts
or publish releases.

The active `Protect release tags` ruleset in
`.github/rulesets/release-tags.json` matches `refs/tags/v*`, permits creation,
and blocks updates and deletions with no bypass actors. Its live id is
`27325536`. Before making a tag, the workflow reads that exact public ruleset
and checks its active tag target, `refs/tags/v*` condition, and update/deletion
rules. The administrator readback verifies the empty bypass list; GitHub omits
that field from unauthenticated public API responses. If tag creation succeeds
but release publication fails, the version tag remains protected and the draft
remains unpublished. An operator must publish the existing draft against that
tag after checking the validated commit. Do not delete or retarget the tag as
recovery.

## Ruleset source

`.github/rulesets/main.json` records the active main ruleset request body for
review. The live ruleset is active and verified; its checks include the PR
title, commit message, frontend, macOS, Windows, and both CodeQL language jobs.
The live release-tag ruleset is active and read back as id `27325536`.
