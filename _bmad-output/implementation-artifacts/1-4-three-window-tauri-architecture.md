# Story 1.4: Three-Window Tauri Architecture

Status: done

## Story

As a developer,
I want the three Tauri windows — menu bar popover, correction overlay, and onboarding wizard — pre-created at startup in hidden state,
so that each can be shown instantly without spawn delay when needed.

## Acceptance Criteria

1. **Given** `tauri.conf.json` defines windows labeled `main`, `overlay`, and `onboarding`, **When** the app starts, **Then** all three windows are created but hidden (`visible: false`).

2. **Given** the `overlay` window is configured, **When** it is defined in `tauri.conf.json`, **Then** it has `alwaysOnTop: true`, `decorations: false`, and `transparent: true`.

3. **Given** `index.html`, `overlay.html`, and `onboarding.html` exist in `src/`, **When** the app builds, **Then** each window loads its corresponding HTML file without errors.

4. **Given** `main.js`, `overlay.js`, and `onboarding.js` exist in `src/`, **When** each window loads, **Then** its JS file is included and the browser console shows no errors.

5. **And** `styles.css` in `src/` is linked in all three HTML files.

## Tasks / Subtasks

- [x] Task 1: Add `overlay` and `onboarding` windows to `tauri.conf.json` (AC: 1, 2, 3)
  - [x] Add `overlay` window entry: `label: "overlay"`, `url: "overlay.html"`, `visible: false`, `alwaysOnTop: true`, `decorations: false`, `transparent: true`, `resizable: false`, `width: 320`, `height: 220`
  - [x] Add `onboarding` window entry: `label: "onboarding"`, `url: "onboarding.html"`, `visible: false`, `width: 600`, `height: 500`, `resizable: false`, `center: true`
  - [x] Verify the existing `main` window entry still has `visible: false` and `decorations: false` — do NOT add a `url` field to `main` (it correctly defaults to `index.html`)

- [x] Task 2: Update `capabilities/default.json` to cover all three windows (AC: 1, 4)
  - [x] Add `"overlay"` and `"onboarding"` to the `windows` array alongside `"main"`
  - [x] Retain existing `"core:default"` and `"store:default"` permissions — these apply to all three windows

- [x] Task 3: Fix `overlay.html` — add `styles.css` link and standardise script tag (AC: 3, 4, 5)
  - [x] Add `<link rel="stylesheet" href="styles.css" />` inside `<head>`
  - [x] Change `<script src="overlay.js"></script>` → `<script type="module" src="/overlay.js"></script>` (consistent with `index.html`)
  - [x] Add `<html lang="en">` if not already present

- [x] Task 4: Fix `onboarding.html` — add `styles.css` link and standardise script tag (AC: 3, 4, 5)
  - [x] Add `<link rel="stylesheet" href="styles.css" />` inside `<head>`
  - [x] Change `<script src="onboarding.js"></script>` → `<script type="module" src="/onboarding.js"></script>` (consistent with `index.html`)
  - [x] Add `<html lang="en">` if not already present

- [x] Task 5: Replace JS stub comments with minimal valid content (AC: 4)
  - [x] `main.js`: Replace stub comment with a console.debug line (no errors)
  - [x] `overlay.js`: Replace stub comment with a console.debug line (no errors)
  - [x] `onboarding.js`: Replace stub comment with a console.debug line (no errors)
  - [x] Do NOT add any Tauri API calls yet — those are implemented in later stories

- [x] Task 6: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
  - [x] `cargo test` — 7 tests pass (5 config + 2 stt), zero failures
  - [ ] Manual: `cargo tauri dev` — all three windows created silently on startup, menu bar icon appears, no errors in Tauri or browser console

## Dev Notes

### Critical: Current State of the Codebase

These files already exist — read them before touching anything:

**`src-tauri/tauri.conf.json` — current state (only `main` window defined):**
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "ICantSpell",
  "version": "0.1.0",
  "identifier": "com.icantspell.app",
  "build": { "frontendDist": "../src" },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "label": "main",
        "title": "ICantSpell",
        "width": 300,
        "height": 400,
        "visible": false,
        "decorations": false
      }
    ],
    "security": { "csp": null }
  },
  "bundle": { ... }
}
```

**`src/index.html` — already correct:** has `<link rel="stylesheet" href="styles.css" />` and `<script type="module" src="/main.js" defer></script>`. **Do NOT modify.**

**`src/overlay.html` — needs two changes:** missing `<link rel="stylesheet" ...>` in `<head>`, and uses bare `<script src="overlay.js">` (not module type).

**`src/onboarding.html` — needs two changes:** same issues as overlay.html.

**`src-tauri/capabilities/default.json` — needs one change:** `"windows"` array only has `["main"]`.

**`src-tauri/src/lib.rs` — do NOT modify:** no Rust changes are needed. Tauri creates all windows defined in `tauri.conf.json` automatically at startup with their configured properties.

### Complete `tauri.conf.json` `windows` Array

Replace the existing `"windows"` array with:

```json
"windows": [
  {
    "label": "main",
    "title": "ICantSpell",
    "width": 300,
    "height": 400,
    "visible": false,
    "decorations": false
  },
  {
    "label": "overlay",
    "url": "overlay.html",
    "title": "ICantSpell Overlay",
    "width": 320,
    "height": 220,
    "visible": false,
    "alwaysOnTop": true,
    "decorations": false,
    "transparent": true,
    "resizable": false
  },
  {
    "label": "onboarding",
    "url": "onboarding.html",
    "title": "Welcome to ICantSpell",
    "width": 600,
    "height": 500,
    "visible": false,
    "resizable": false,
    "center": true
  }
]
```

**Critical notes:**
- `main` window has no `url` field — this is intentional; it defaults to `index.html` via `frontendDist: "../src"`. Do NOT add a `url` field to `main`.
- `overlay` window: all three properties `alwaysOnTop`, `decorations: false`, and `transparent` are required by AC 2. A missing `transparent: true` will show a white background on the overlay.
- `onboarding` window: `center: true` positions it in the middle of the screen on first show — good UX default for a wizard.
- Window URLs like `"overlay.html"` are resolved relative to `frontendDist` (`../src`), so `src/overlay.html` is served correctly.

### Complete `capabilities/default.json`

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for all ICantSpell windows",
  "windows": ["main", "overlay", "onboarding"],
  "permissions": [
    "core:default",
    "store:default"
  ]
}
```

All three windows need `core:default` (covers Tauri event system, IPC, window management) and `store:default` (onboarding will read config in Story 2.1). Adding them all here now prevents a missing-capability bug in later stories.

### Fixed `overlay.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>ICantSpell Overlay</title>
  <link rel="stylesheet" href="styles.css" />
</head>
<body>
  <!-- Correction overlay UI — implemented in Story 4.2 -->
  <script type="module" src="/overlay.js"></script>
</body>
</html>
```

### Fixed `onboarding.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Welcome to ICantSpell</title>
  <link rel="stylesheet" href="styles.css" />
</head>
<body>
  <!-- First launch onboarding wizard — implemented in Story 2.1 -->
  <script type="module" src="/onboarding.js"></script>
</body>
</html>
```

### JS Stub Replacements

Replace each file's single comment line with:

**`main.js`:**
```js
// Menu bar popover logic — IPC listeners and controls added in Story 5.1
console.debug("[ICantSpell] main window loaded");
```

**`overlay.js`:**
```js
// Correction overlay logic — IPC listeners and tap interactions added in Story 4.2
console.debug("[ICantSpell] overlay window loaded");
```

**`onboarding.js`:**
```js
// Onboarding wizard step logic — permission prompts added in Story 2.1
console.debug("[ICantSpell] onboarding window loaded");
```

`console.debug` is intentional — shows in devtools without being a warning/error. Later stories will remove these stubs and add real logic.

### Why No Rust Changes

Tauri v2 creates all windows listed in `tauri.conf.json` automatically during `tauri::Builder::default().run(...)`. The `visible: false` property means they are created in an invisible state — they exist in memory and their webviews are loaded, satisfying NFR3 (< 200ms appearance time for overlay) by eliminating the spawn delay. No `overlay.rs` code is needed for this story; `overlay.rs` will be implemented in Epic 4.

### Transparent Overlay CSS Note

The `transparent: true` Tauri config alone is not enough to make the overlay truly see-through. Later when the overlay UI is built (Story 4.2), `overlay.html`'s body must have `background: transparent` in CSS. For this story the body is empty, so no visible artifact appears — but make a note of this for Story 4.2.

### Tauri v2 Window Property Names

These are the **exact** camelCase names used in `tauri.conf.json` v2 schema:
- `alwaysOnTop` (NOT `always_on_top`)
- `decorations` (boolean)
- `transparent` (boolean)
- `visible` (boolean)
- `resizable` (boolean)
- `center` (boolean — positions window center-screen on first show)

Do NOT use snake_case property names — the Tauri schema validator will reject them.

### Files Touched

| File | Action | Reason |
|------|---------|--------|
| `src-tauri/tauri.conf.json` | MODIFY | Add `overlay` and `onboarding` window entries |
| `src-tauri/capabilities/default.json` | MODIFY | Add `overlay` and `onboarding` to windows array |
| `src/overlay.html` | MODIFY | Add `styles.css` link + fix script tag to `type="module"` |
| `src/onboarding.html` | MODIFY | Add `styles.css` link + fix script tag to `type="module"` |
| `src/main.js` | MODIFY | Replace stub comment with valid minimal content |
| `src/overlay.js` | MODIFY | Replace stub comment with valid minimal content |
| `src/onboarding.js` | MODIFY | Replace stub comment with valid minimal content |

**Files NOT touched:**
- `src/index.html` — already correct, no changes needed
- `src/styles.css` — no changes needed
- `src-tauri/src/lib.rs` — no Rust changes for this story
- `src-tauri/src/main.rs` — no changes
- All other `src-tauri/src/*.rs` stubs — remain as-is

### References

- [Source: epics.md § Story 1.4] — Acceptance criteria and user story
- [Source: architecture.md § Tauri Window Configuration] — Three windows, their properties (alwaysOnTop, decorations, transparent, visible)
- [Source: architecture.md § Frontend Architecture] — "Pre-created hidden WebviewWindow for correction overlay (satisfies NFR3 < 200ms appearance)"
- [Source: architecture.md § Frontend Organization] — `src/` directory layout: index.html, overlay.html, onboarding.html, main.js, overlay.js, onboarding.js, styles.css
- [Source: architecture.md § Project Structure] — Complete directory listing showing `src/` structure
- [Source: story 1-3 Completion Notes] — "capabilities/default.json currently has core:default only; add store:default" (already done; extend windows array here)
- [Source: epics.md § Story 2.1] — "onboarding wizard is implemented in `onboarding.html` / `onboarding.js` using the existing window from Story 1.4"
- [Source: epics.md § Story 5.1] — "popover is implemented in `index.html` / `main.js` using the existing `main` window from Story 1.4"

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

_None — clean implementation, no issues encountered._

### Completion Notes List

- Added `overlay` window to `tauri.conf.json`: `alwaysOnTop: true`, `decorations: false`, `transparent: true`, `visible: false`, `resizable: false`, 320×220, URL `overlay.html`
- Added `onboarding` window to `tauri.conf.json`: `visible: false`, `resizable: false`, `center: true`, 600×500, URL `onboarding.html`
- `main` window left unchanged (no `url` field; defaults to `index.html` via `frontendDist`)
- Updated `capabilities/default.json` `windows` array from `["main"]` to `["main", "overlay", "onboarding"]`
- Fixed `overlay.html`: added `<link rel="stylesheet" href="styles.css" />`, `<meta name="viewport">`, `lang="en"`, changed script to `type="module" src="/overlay.js"`
- Fixed `onboarding.html`: same fixes as overlay.html; updated title to "Welcome to ICantSpell"
- Updated all three JS stub files (`main.js`, `overlay.js`, `onboarding.js`) with valid `console.debug` lines — no Tauri API calls added yet
- `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
- `cargo test` — 7/7 tests pass (5 config + 2 stt), no regressions

### File List

- `src-tauri/tauri.conf.json` — added `overlay` and `onboarding` window entries; enabled `macOSPrivateApi: true` (required for `transparent: true` on overlay window)
- `src-tauri/capabilities/default.json` — extended `windows` array to cover all three windows
- `src/overlay.html` — added `styles.css` link, `viewport` meta, `lang="en"`, standardised script tag to `type="module"`
- `src/onboarding.html` — added `styles.css` link, `viewport` meta, `lang="en"`, standardised script tag to `type="module"`, updated title
- `src/main.js` — replaced stub comment with valid `console.debug` line
- `src/overlay.js` — replaced stub comment with valid `console.debug` line
- `src/onboarding.js` — replaced stub comment with valid `console.debug` line

### Review Findings

- [ ] [Review][Decision] Manual `cargo tauri dev` validation not completed — Task 6 sub-item is unchecked; three hidden windows must be verified at runtime (no console errors, menu bar icon appears)
- [x] [Review][Defer] CSP null disables all Content Security Policy protections [src-tauri/tauri.conf.json:44] — deferred, pre-existing
- [x] [Review][Defer] `withGlobalTauri: true` exposes full Tauri API to all windows [src-tauri/tauri.conf.json:10] — deferred, pre-existing
- [x] [Review][Defer] Redundant `defer` attribute on `type="module"` script in index.html [src/index.html:8] — deferred, pre-existing (file not modified in this story)
- [x] [Review][Defer] Overlay transparency defeated by shared styles.css opaque background [src/styles.css:12] — deferred, explicitly deferred to Story 4.2 per spec
- [x] [Review][Defer] No meta CSP fallback in any HTML file — deferred, pre-existing (related to null CSP config)
- [x] [Review][Defer] `macOSPrivateApi: true` present in tauri.conf.json but absent from spec snippet — deferred, documentation discrepancy from prior story

## Change Log

- 2026-04-28: Story 1.4 created — Three-Window Tauri Architecture ready for dev.
- 2026-04-29: Implemented Story 1.4 — Three-Window Tauri Architecture. Added overlay and onboarding windows to tauri.conf.json, extended capabilities to all three windows, fixed overlay.html and onboarding.html (styles.css link + module script tags), replaced JS stubs with valid content. All 7 tests pass.
- 2026-04-29: Code review completed — 0 patches, 1 decision-needed (manual test), 6 deferred, 8 dismissed.
