# Release ruleset API review

Luna implemented source `c77ced63aee704f4e737f078641eec01e0d79e28` in an isolated worktree. Terra independently approved it against main `e666345b6975b00c0b47ac8335cd0bb7bc50e06e` and passed actionlint and git diff checks.

Run 45882467931 passed exact-commit CI and hosted DMG verification. The downloaded draft asset matched its checksum. Publication stopped before tag creation when the anonymous ruleset request returned HTTP 403. The fix uses the existing workflow token for the same GET and checks the same ruleset predicates. GitHub documents metadata read for this endpoint. The authenticated local lookup passed, and the hosted job must verify its own token and remaining publication steps.

The final documentation head requires Terra approval and protected hosted checks before squash merge.
