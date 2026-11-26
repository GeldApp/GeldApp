# Release and Pages workflow review

Luna implemented the fixes in separate worktrees. Terra independently approved release source `64e1e0cfefea6a79848800522306b3a4aea7d30f` and Pages source `ef73db818bef1e84e764adf0278c701ae148f8fa`. Their combined source is `b2741a7d8a4abd78b072a751b0dd7b0c361432c7`.

Release run 45876343316 failed because the wait action's Ruby lockfile omitted the macOS runner platform. Ubuntu now runs the five existing success-only waits on the exact main commit before the macOS job. Main, tag and CodeQL guards remain in force. Tagged runs may wait before skipping or rejecting publication.

The Pages upload and deployment now agree on an artifact name unique to the run attempt. Attempt 2 of run 45707699964 confirmed the duplicate-name failure. A fresh main dispatch deployed the install instructions successfully.

Validation passed for both files with actionlint and git diff checks. The combined head requires final Terra approval and protected hosted checks before squash merge. The first release build and real cask installation remain to be verified.
