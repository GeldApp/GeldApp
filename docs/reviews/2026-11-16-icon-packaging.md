# Desktop icon packaging review

- Source reviewed: `740480170e20eba818c407069a789f992f99dd3a`.
- Independent Terra verdict: approved for draft PR and live checks; no source blocker.
- Fix: explicitly configure existing PNG, ICNS and ICO assets in Tauri's bundle icon list. Before the fix, the built macOS app omitted its icon resource.
- Worker validation: macOS debug app bundle rebuilt successfully; `CFBundleIconFile` names `icon.icns`, and the bundled ICNS SHA-256 matches its source asset. The configured ICO exists.
- Finder/Dock rendering and the Windows installer icon have not been directly inspected. Final-head CI and unchanged-base review remain merge conditions.
