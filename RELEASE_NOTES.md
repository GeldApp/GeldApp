# Release notes

## Install the macOS preview

After a guarded GitHub release is published and its cask update has merged, install the current version:

```sh
brew install --cask geldapp/tap/geldapp
```

Upgrade with `brew upgrade --cask geldapp`; remove the app with `brew uninstall --cask geldapp`. Uninstall preserves the local financial vault. Versioned universal DMGs and SHA-256 checksums are attached to [GitHub releases](https://github.com/GeldApp/GeldApp/releases).

Requires macOS 13.3 or later. The cask declares Homebrew's supported Ventura minimum, which checks the macOS major version; the app bundle declares 13.3 and macOS 13.0 through 13.2 will reject it at launch.

The preview is ad-hoc signed and unnotarized, with no Apple Developer ID signature. Homebrew does not remove quarantine. If Gatekeeper blocks it, first verify the download is from the linked GitHub release and matches its published SHA-256 checksum. If you choose to proceed, try opening the app, then go to System Settings > Privacy & Security > Open Anyway and review Apple's confirmation prompt. Follow [Apple's current instructions](https://support.apple.com/en-us/102445); do not change system-wide security settings. Use sample data only. Keychain access, physical Touch ID prompts, and Windows runtime behavior are unverified.
