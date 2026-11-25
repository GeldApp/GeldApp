# Release notes

## Install the macOS preview

After a guarded GitHub release is published and its cask update has merged, install the current version:

```sh
brew install --cask geldapp/tap/geldapp
```

Upgrade with `brew upgrade --cask geldapp`; remove the app with `brew uninstall --cask geldapp`. Uninstall preserves the local financial vault. Versioned universal DMGs and SHA-256 checksums are attached to [GitHub releases](https://github.com/GeldApp/GeldApp/releases).

Requires macOS 13.3 or later. The cask declares Homebrew's supported Ventura minimum, which checks the macOS major version; the app bundle declares 13.3 and macOS 13.0 through 13.2 will reject it at launch.

The preview is ad-hoc signed and unnotarized, with no Apple Developer ID signature. macOS Gatekeeper may require a user-approved first launch through Finder's Control-click > Open flow. Homebrew does not remove quarantine. Use sample data only. Keychain access, physical Touch ID prompts, and Windows runtime behavior are unverified.
