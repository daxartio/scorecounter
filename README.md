# Score Counter

Single-page PWA score tracker with full-height rows and offline persistence.

## Features
- Tap ± for ±1, hold ~0.5s for ±5 (mobile-friendly; zoom and selection disabled on controls).
- Add rows from the faint bottom cell; each row gets its own color, name, and score.
- Edit/delete via row actions; settings view with quick usage tips.
- State is saved in `localStorage` (`scorecounter:v1`, schema versioned).
- Install-ready PWA: scoped manifest, app icons (incl. iOS touch icon), theme color, and a service worker cache.

## Run (dev)
```bash
dx serve --platform web
```
The app is single-page; Settings is toggled via the top bar button.

## Build (web/PWA)
```bash
dx build --platform web --release
```
Output lives under `target/dx/scorecounter/release/web/public/` with `manifest.webmanifest`, icons, and `service-worker.js`.

### Add to home screen
- The manifest (`assets/manifest.webmanifest`) is scoped to `/scorecounter/` and aligns with `Dioxus.toml`'s `base_path`. Adjust both together if you deploy under a different path.
- `sw-register.js` derives the service worker path/scope from the manifest link, so the SW continues to work when the base path changes.
- iOS gets `apple-touch-icon.png` plus `apple-mobile-web-app-*` meta tags; Android/desktop use the web manifest. Use the browser's “Add to Home Screen” / “Install app” action to pin it.

## Files to know
- `src/app.rs` — UI/state, dialogs, settings view, long-press logic, add-row UX.
- `src/state.rs` — store, schema/version, serialization tests.
- `assets/main.css` — layout/typography, row heights, tactile controls.
- `assets/manifest.webmanifest`, `assets/service-worker.js`, `assets/sw-register.js`, `assets/interaction.js` — PWA glue and gesture tweaks.

## Tests
```bash
cargo test
```
