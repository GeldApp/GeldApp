# Quality gates

The `Check` workflow runs on pushes and pull requests on both macOS and Windows. It installs the locked npm dependencies, builds and tests the frontend, checks Rust formatting, runs Rust tests and compilation, and compiles a debug Tauri application without creating platform bundles. The workflow has read-only repository permissions and does not publish or sign artifacts.

The desktop build depends on each runner's native toolchain and WebView runtime. A green job confirms compilation on that runner; it does not validate Touch ID, Keychain behavior, or a physical Windows installation.

## CSV export safety

CSV fields are quoted with standard double-quote escaping. At serialization time, date, merchant, category, and note fields get an apostrophe prefix when the first non-whitespace/control character is `=`, `+`, `-`, or `@`. Amount remains an unquoted signed decimal number. Tests check the emitted CSV bytes only; they do not establish how a spreadsheet application opens the file.

This prefix is a mitigation for common formula markers in the initial export, not a guarantee that CSV is safe for spreadsheet use or transport of untrusted text. Spreadsheet applications may handle it differently, and [OWASP warns that Excel may remove escaping when a CSV is saved and reopened](https://owasp.org/www-community/attacks/CSV_Injection). We have not tested opening or save/reopen behavior in a spreadsheet application. Consider XLSX export with cells explicitly typed as text for future spreadsheet interchange.

The apostrophe prefix changes formula-like text and is not removed on import. CSV export is for readable interchange, not an exact byte-for-byte backup; use encrypted GeldApp backups for archival and restoration.
