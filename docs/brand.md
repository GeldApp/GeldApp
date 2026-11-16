# GeldApp visual identity

GeldApp uses a blue-black-and-gold identity over a clear ivory interface. The original site mark is shared across the app, browser favicon, wordmark, and Tauri source icon so the product reads as one brand from landing page to desktop.

## Palette

| Role | Color | Use |
| --- | --- | --- |
| Navy | `#123B73` | Brand mark, primary actions, balance card, selected navigation |
| Deep navy | `#0C2B54` | Toasts and deep surfaces |
| Warm gold | `#EFC76F` | Brand G, selected state details, current chart accents |
| Ink | `#172536` | Main text and wordmark |
| Muted | `#5D6876` | Supporting text |
| Ivory gray | `#F5F6F7` | Main canvas and light surfaces |
| Pale blue | `#E6ECF4` | Navigation rail and secondary surfaces |

Gold stays on dark or tinted surfaces, or appears as a compact accent. Analytic series use blue and gold; transaction totals retain readable green for incoming money and red for outflows.

## Assets

- `public/geldapp.svg` and `public/geldapp-mark.svg` share the editable site mark for browser tabs and the compact app rail.
- `public/geldapp-wordmark.svg` is the editable lockup.
- `src-tauri/icons/app-icon.svg` is the editable desktop icon source. `npm run tauri -- icon src-tauri/icons/app-icon.svg` regenerates the standard Tauri PNG, ICNS, ICO, Android, and iOS assets.
- `brand-sheet.svg` presents the mark and palette.

Bundled variable Manrope headings and DM Sans interface text keep the typography consistent with the site. The SVG path remains editable and scales cleanly at small sizes.

## Interface treatment

The ivory content surface keeps transaction rows and settings readable. Navy anchors the rail and primary balance, while gold marks selected navigation and the identity. Fine blue-gray dividers and all-caps labels keep dense financial information organized. Positive and negative amounts remain distinct through green and red text.

Desktop and narrow browser previews were inspected at 1280×720 and 390px. The browser preview uses demo data and local storage; it cannot establish native vault behavior, Keychain behavior, or Windows appearance.
