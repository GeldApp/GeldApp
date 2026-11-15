# Quality gates

The `Check` workflow runs on pushes and pull requests on both macOS and Windows. It installs the locked npm dependencies, builds and tests the frontend, checks Rust formatting, runs Rust tests and compilation, and compiles a debug Tauri application without creating platform bundles. The workflow has read-only repository permissions and does not publish or sign artifacts.

The desktop build depends on each runner's native toolchain and WebView runtime. A green job confirms compilation on that runner; it does not validate Touch ID, Keychain behavior, or a physical Windows installation.

## CSV export safety

CSV fields are quoted with standard double-quote escaping. Text fields are also prefixed with an apostrophe when the first non-whitespace/control character is `=`, `+`, `-`, or `@`, which prevents common spreadsheet applications from treating the exported cell as a formula. This applies to date, merchant, category, and note. Amount remains an unquoted signed decimal number.

Formula-safe export can change a text value by adding that apostrophe. Importing an exported CSV therefore preserves the spreadsheet-safe text rather than reconstructing the original formula-like string. CSV export is for readable interchange, not an exact byte-for-byte backup; use encrypted GeldApp backups for archival and restoration.
