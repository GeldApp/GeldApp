# Delivery dependency and platform sync

- Implementation: Luna in isolated worktree; independent adversarial reviewer: gpt-6.1-terra.
- Exact reviewed source: `b2f1f86e2008b3f9a88b8448c7360509d3e1c07e`, integrating previously reviewed delivery `00fbd4fa7c178924fa554f8d257fad10fb1dea3a` and main `152012da18308d2de53cba3eed364df7ecd025a6`.
- Initial verdict: APPROVE source for hosted CI, with no blocking integration findings. Final merge approval and its conditions are recorded below; publication remains a separate primary-agent release decision.
- Reviewed conflict resolution: Vite E2E output stays separate from production while retaining Safari 16.4/Chromium 111 targets; native feature guards, fixture namespace, Keychain exclusions, legacy AES fixture and entropy-failure regression remain intact. All install surfaces state macOS 13.3; Homebrew major-only dependency limitation is explicit.
- Reviewer independently passed frontend build, 14 tests and lint; 15 locked native tests, formatting and strict all-target/all-feature Clippy. Worker also passed locked native build and isolated macOS setup/restart/corruption-recovery E2E suites with synthetic data and Keychain excluded. No real Keychain or physical biometric tests occurred.
- Required hosted checks must pass on the final published head. Windows execution of the new integration and oldest macOS 13.3 runtime remain unverified locally.
- CodeQL alert 4 on the unchanged test-only Argon2 known-answer salt was classified as `used in tests` after explicit user authorization at 2026-11-25 21:19:12 UTC. Only alert 4 was dismissed as authorized; no other alert was dismissed and no security rule, query, path or finding gate changed. No release or tag has been published from this prep.

## Windows startup evidence follow-up

- Reviewed source: `479b8fef23cd92c82ce0c7f5e846609e80ce8d4b`. Independent Terra verdict: APPROVE for hosted CI; Windows startup cause remains undetermined.
- Prior head `51d9f97` passed hosted macOS GUI suites and all other required jobs except Windows GUI startup and the subsequently authorized test-only classification. Windows normal build/tests and isolated app build passed, but the embedded server did not become ready within 15 seconds.
- Fixes confirmed log routing: WebdriverIO uses outputDir, while the previous logDir setting sent captured startup logs outside the artifact path. Info-level logs and compile-gated fixed-string E2E phase markers provide startup evidence without credentials, ledger contents or paths. Production behavior, test-data guards and timeout are unchanged.
- Worker verified log-file creation at the configured artifact path, production/test marker absence/presence respectively, formatting, E2E debug compilation and all three isolated macOS lifecycle suites with captured startup/readiness evidence.
- Reviewer verified compile gating, fixed non-sensitive markers, unchanged production launch behavior and preserved isolation. Fresh hosted Windows results are required before merging; this diagnostic change alone does not establish a startup fix.

## Final merge verdict and hosted validation

- Exact reviewed head: `12cca8ee8cd43edaca9b18ecd6fb0fe02e3619e4`. Independent Terra MERGE VERDICT: APPROVE, with no blocking integration finding remaining, conditional on required checks passing and the authorized alert-4 classification. Both conditions were independently verified by the primary agent.
- All hosted checks passed on that head: frontend; native macOS including GUI suites (6m02s); native Windows including setup/restart/corruption recovery (9m58s), run `45704237297`; both CodeQL analyses and the findings gate; Conventional title/commit policy; Pages validation. PR deployment correctly skipped.
- Windows artifact captured fixed-string E2E phases and backend/driver logs. Initial setup reached the fixture guard and vault session open about 7.03s after process entry. Recovery app spawn to driver readiness was 536ms. The earlier 15-second timeout was not reproduced; its exact cause is unknown. The observability change is not represented as a proven startup fix.
- User-approved classification affects only alert 4 as `used in tests`; no test, production randomness, security rule or required check was weakened. No physical biometric, oldest-macOS or Developer ID/notarization claim is made.
- This final record changes documentation only. The final metadata head must receive Terra verification of its documentation-only diff and pass protected checks before the primary agent squash-merges. No additional source change is authorized by this record. Release/tag publication must separately follow the guarded release workflow and user-requested preview delivery scope.
