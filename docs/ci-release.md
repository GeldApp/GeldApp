# CI and release policy

## Checks

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

GitHub displays required contexts using the check-run job names above (the
workflow name may also appear in the required-check selector). Confirm the
exact context strings after the first successful run before creating a ruleset.

The app uses `@biomejs/biome` with recommended lint rules and React's
recommended domain. `npm run lint` only lints; it does not reformat files.
Native jobs run Clippy with warnings denied. Existing UI accessibility and
style findings remain visible as Biome warnings while the current UI is under
parallel security work; they are not auto-fixed or hidden. This includes
button-type diagnostics whose correct fix depends on whether the control
submits a form.

The PR policy accepts Conventional Commit types `feat`, `fix`, `docs`, `style`,
`refactor`, `perf`, `test`, `build`, `ci`, `chore`, and `revert`. Scope is
optional. Both PR titles and every commit in the PR must be conventional.
Breaking changes use `type(scope)!: ...` and/or a `BREAKING CHANGE:` footer.
The repository is configured for squash merge with the PR title as the squash
commit subject and the PR body as its message. Preserve the footer in the PR
body for release versioning. For example, `feat(storage): add encrypted export`
and `fix(storage)!: rotate the key format` are valid; `add encrypted export`
and `feature: add encrypted export` are rejected. The title job also runs when
the PR title is edited. Both policy checks use `pull_request` with read-only
permissions, including on fork PRs. Commitlint ignores generated Git merge
commits, while checking the commits that contain actual changes.

## CodeQL availability and fork behavior

CodeQL has separate JavaScript/TypeScript and Rust jobs. Rust uses CodeQL's
`none` build mode; the normal native CI jobs still compile and test the app.
GitHub documents both languages as supported by CodeQL.

During initial setup the repository was private, and CodeQL access returned
HTTP 403 because the hosting plan lacked Code Security. The repository is now
public. Both language analyses and the CodeQL findings check passed on CI PR
#5. The workflow uploads SARIF on all supported events; unavailable or failed
analysis must never be treated as a clean scan.

Fork PRs use the unprivileged `pull_request` event and read-only token. GitHub
allows code-scanning SARIF uploads from `pull_request` runs with read-only
tokens. No workflow checks out or executes untrusted code from
`pull_request_target`.

The `Analyze (javascript-typescript)` and `Analyze (rust)` jobs indicate that
the analyses completed. The active ruleset separately requires CodeQL results
with a security-alert threshold of **High or higher** (including Critical),
plus all quality alerts. A workflow job success alone does not mean that no
alerts were found. GitHub
documents this code-scanning threshold rule in [Set code scanning merge
protection](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/manage-your-configuration/set-merge-protection).

## Draft release behavior

Every push to `main` starts the `Release draft` workflow. A serialized
concurrency group allows one release-draft writer at a time; if several pushes
arrive while it runs, GitHub keeps the newest pending run so the draft
converges to the latest `main` commit.

The annotated `v0.1.0` tag on the approved prototype commit is the version
baseline. The workflow fetches tags and uses the standard
[`paulhatch/semantic-version` action](https://github.com/PaulHatch/semantic-version)
to calculate the next version from commits after the latest `v*` tag:

- `feat:` and `feat(scope):` raise the minor version.
- `type(scope)!:` or a `BREAKING CHANGE:` / `BREAKING-CHANGE:` footer raises the
  major version. At `0.x`, a breaking change produces `1.0.0`.
- Other commit types, including `fix`, `docs`, and `chore`, use the action's
  default patch increment. With `bump_each_commit: false`, the action chooses
  one next version from all commits since the latest tag; ordinary commits
  share that implied patch version until a tag is published. A later feature
  or breaking commit can raise the draft's implied version for the whole
  unreleased range. The workflow still updates the draft on every main push.

The workflow passes that version to the standard
[`release-drafter` action](https://github.com/release-drafter/release-drafter),
which creates or updates one GitHub draft release targeted at `main`. Release
drafting does not publish the release or create a Git tag. Publishing remains
a manual release step; its tag becomes the baseline for later version
calculation. Existing prototype commits before `v0.1.0` do not affect the next
version.

The release workflow only runs on pushes to `main`. Branch and draft pull
request commits do not create draft releases.

## Main branch ruleset

`.github/rulesets/main.json` is the REST request body for the active
`main` ruleset. It requires pull requests, squash only, strict passing checks,
Conventional Commit subjects, no force pushes or branch deletion, and CodeQL
results with a High-or-higher security alert threshold. It grants no bypass
actors. Ruleset `27324053` is active and its settings were read back from the
GitHub API. The initial private-repository HTTP 403 restriction was resolved
when the repository became public. Updating the ruleset requires repository
Administration write access.

The required status contexts in the JSON match job names declared by the
workflows and were verified against the live check runs on CI PR #5. The repository's squash-only merge setting is already enabled; the
ruleset repeats that constraint for branch protection.
