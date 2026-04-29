# Story 1.1: Tauri Project Scaffold & Menu Bar Shell

Status: done

## Story

As a developer,
I want the Tauri project initialized and running as a macOS menu bar application with no Dock icon,
so that there is a working build baseline for all future feature development.

## Acceptance Criteria

1. **Given** the developer runs `cargo create-tauri-app icantspell --template vanilla`, **When** the app builds and launches, **Then** the menu bar icon appears and no Dock icon is shown.

2. **Given** `tauri.conf.json` is configured and `app.set_activation_policy(ActivationPolicy::Accessory)` is called in the Tauri setup hook, **When** the app launches, **Then** the app has no Dock presence and no visible window on startup.

3. **Given** the app is running, **When** the user clicks the menu bar icon, **Then** a minimal tray menu appears with at least a Quit option.

4. **Given** the developer runs `cargo tauri dev`, **When** they modify a file in `src/`, **Then** the webview hot-reloads without a full Rust rebuild.

5. **And** the project directory structure matches the Architecture spec (`src/`, `src-tauri/src/`, `src-tauri/tauri.conf.json`, etc.) with all required directories created and stub files in place for future stories.

## Tasks / Subtasks

- [x] Task 1: Bootstrap Tauri project scaffold (AC: 1, 4, 5)
  - [x] Install Tauri CLI: `cargo install create-tauri-app --locked`
  - [x] Create project: run `cargo create-tauri-app icantspell --template vanilla` in parent directory (see Dev Notes on scaffold placement)
  - [x] Move/merge generated contents into project root, preserving `_bmad-output/` and `.claude/` siblings
  - [x] Verify `cargo tauri dev` runs without errors and `src/index.html` hot-reloads on save

- [x] Task 2: Restructure frontend to match architecture spec (AC: 4, 5)
  - [x] If `index.html` is at project root after scaffold, move it to `src/index.html`
  - [x] Update `tauri.conf.json` `build.frontendDist` to `"../src"` if not already pointing there
  - [x] Create stub frontend files: `src/overlay.html`, `src/onboarding.html`
  - [x] Create stub JS files: `src/overlay.js`, `src/onboarding.js`
  - [x] Ensure `src/main.js` and `src/styles.css` exist (from scaffold or created fresh)
  - [x] Verify `cargo tauri dev` still starts cleanly after restructure

- [x] Task 3: Configure menu bar — no Dock icon, system tray icon (AC: 1, 2, 3)
  - [x] Add `features = ["tray-icon"]` to the tauri dependency in `src-tauri/Cargo.toml`
  - [x] In `src-tauri/src/lib.rs` `.setup()` hook: call `app.set_activation_policy(ActivationPolicy::Accessory)` behind `#[cfg(target_os = "macos")]`
  - [x] Build tray icon with a `Quit` menu item using `TrayIconBuilder` and `MenuItem::with_id`
  - [x] Wire the `"quit"` event to `app.exit(0)`
  - [x] Set `"visible": false` on the `main` window in `tauri.conf.json` so no window appears at launch
  - [x] Launch and confirm: no Dock icon, menu bar icon visible, Quit item works

- [x] Task 4: Set up Rust module stub structure (AC: 5)
  - [x] Confirm `src-tauri/src/main.rs` delegates to `lib.rs` (Tauri v2 default)
  - [x] Confirm `src-tauri/src/lib.rs` contains the `run()` function with `Builder::default()`
  - [x] Create `src-tauri/src/error.rs` as a stub (empty `pub mod error {}` or minimal `AppError` placeholder — full definition is Story 1.2)
  - [x] Create `src-tauri/src/stt/` directory with empty `mod.rs` stub — full STT trait is Story 1.2
  - [x] Create `src-tauri/src/config.rs` stub — full implementation is Story 1.3
  - [x] Create `src-tauri/src/` stubs for: `audio.rs`, `hotkey.rs`, `injection.rs`, `overlay.rs`, `permissions.rs`, `models.rs` — each a comment-only stub noting which story implements it
  - [x] Create `src-tauri/tests/` directory (empty, holds integration tests added in later stories)
  - [x] Create `src-tauri/models/` directory (placeholder — bundled model added in Story 5.3)
  - [x] Create `src-tauri/capabilities/default.json` if not created by scaffold

- [x] Task 5: Add `tracing` logging dependencies (AC: 5)
  - [x] Add `tracing` and `tracing-subscriber` to `src-tauri/Cargo.toml`
  - [x] Initialize `tracing_subscriber` in `lib.rs` `run()` before building the Tauri app
  - [x] Verify `cargo build` compiles cleanly with no warnings

- [x] Task 6: Final validation (AC: 1–5)
  - [x] `cargo tauri dev`: app launches, menu bar icon appears, no Dock icon
  - [x] Click tray icon: Quit menu item appears and exits app cleanly
  - [x] Modify `src/index.html`: webview reloads without full Rust rebuild
  - [x] `cargo clippy --all-targets -- -D warnings`: passes with zero warnings
  - [x] `cargo test`: passes (no tests yet — just confirms test harness works)
  - [x] Verify full directory structure matches architecture spec layout

## Dev Notes

### Critical: Activation Policy API (Architecture Doc vs. Actual Tauri v2 API)

The architecture doc mentions `"activationPolicy": "accessory"` in `tauri.conf.json`, but **this is NOT a tauri.conf.json field in Tauri v2**. The correct approach is to call it from Rust in the `.setup()` hook:

```rust
// src-tauri/src/lib.rs
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            // ... tray setup ...
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Wrap in `#[cfg(target_os = "macos")]` — this API is macOS-only and will fail to compile on Windows/Linux without the guard.

---

### Scaffold Placement Strategy

`cargo create-tauri-app icantspell --template vanilla` creates a **new subdirectory** named `icantspell`. The project root is `/Users/asn-mac/Desktop/ICantSpell`, which already contains `_bmad-output/` and `.claude/` (planning artifacts — do NOT touch these).

**Recommended approach:**
1. Run the create command from `/Users/asn-mac/Desktop/` (parent of ICantSpell): `cd ~/Desktop && cargo create-tauri-app icantspell --template vanilla`
2. Copy all generated contents from `Desktop/icantspell/` into `Desktop/ICantSpell/`, skipping file conflicts (`.gitignore`, `README.md` can be merged manually)
3. Delete the temporary `Desktop/icantspell/` directory

**Alternative (create directly):**
Some versions of `create-tauri-app` support `--directory` or accept `.` as the project name — check the installed version and use if available.

**Result after merge:** `ICantSpell/` should have `src/`, `src-tauri/`, `package.json` at the top level, alongside the existing `_bmad-output/` and `.claude/`.

---

### Vanilla Template Output Structure

The vanilla template generates:
```
icantspell/
├── package.json              ← frontend package management
├── index.html                ← ⚠️ at ROOT, NOT in src/ — must move to src/index.html
├── src/
│   ├── main.js
│   └── styles.css
└── src-tauri/
    ├── Cargo.toml
    ├── Cargo.lock
    ├── build.rs
    ├── tauri.conf.json       ← frontendDist points to ".." (root) by default
    ├── capabilities/
    │   └── default.json
    ├── icons/
    └── src/
        ├── lib.rs            ← app setup code goes here (Tauri v2)
        └── main.rs           ← thin wrapper that calls lib::run()
```

**Action required:** Move `index.html` from project root into `src/`, then update `tauri.conf.json`:
```json
{
  "build": {
    "frontendDist": "../src"
  }
}
```

---

### System Tray Setup (Tauri v2 API)

Required `Cargo.toml` feature flag:
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

Complete `lib.rs` tray setup pattern:
```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_item])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### tauri.conf.json — Required Settings for This Story

Minimum required changes from scaffold defaults:

```json
{
  "productName": "ICantSpell",
  "identifier": "com.icantspell.app",
  "build": {
    "frontendDist": "../src"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "ICantSpell",
        "width": 300,
        "height": 400,
        "visible": false,
        "decorations": false
      }
    ]
  }
}
```

**Do NOT define `overlay` and `onboarding` windows yet** — Story 1.4 owns the three-window architecture. Keep only `main` here.

---

### Stub File Conventions

Rust module stubs should be minimal but valid — they must compile. Use this pattern:
```rust
// src-tauri/src/audio.rs
// Audio capture pipeline — implemented in Story 3.2
// Uses: cpal crate, tokio::sync::mpsc channel
// See architecture.md § Audio Pipeline Architecture
```

Do not add `mod` declarations in `lib.rs` for stubs that don't export anything — this will cause unused warning errors with `clippy -D warnings`. Either omit the mod declaration entirely or add `#[allow(dead_code)]` if Clippy complains on the stub module itself.

**For `error.rs`:** Add at minimum:
```rust
// src-tauri/src/error.rs
// Shared error types — implemented in Story 1.2
// Uses: thiserror crate
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unknown error: {0}")]
    Unknown(String),
}
```
This gives downstream stories a valid import target. Ensure `thiserror` is in `Cargo.toml`.

---

### Privacy Enforcer: NO HTTP Client Crates

Even at scaffold stage, verify `Cargo.toml` contains NONE of: `reqwest`, `hyper`, `ureq`, `surf`, `isahc`. The scaffold should not add these, but double-check. This is enforced by CI in Story 1.5.

[Source: architecture.md § Privacy Boundary]

---

### Logging: tracing from Day One

Initialize `tracing_subscriber` in `lib.rs::run()` before the Tauri builder — all future modules use `tracing::{info, warn, error, debug}` macros. Never use `println!` or `eprintln!` in library code.

```rust
// ✅ Correct
tracing::info!("ICantSpell starting up");

// ❌ Forbidden
println!("ICantSpell starting up");
```

[Source: architecture.md § Logging Patterns]

---

### Tauri v2 main.rs Pattern

In Tauri v2, `main.rs` is a thin desktop wrapper — all app logic lives in `lib.rs`:
```rust
// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    icantspell_lib::run()
}
```

The crate is typically named in `Cargo.toml` as `icantspell-lib` or similar. Check the generated `Cargo.toml` `[lib]` section and use the correct name.

---

### Project Structure Notes

**Target structure after this story (directories and stubs):**
```
ICantSpell/
├── .github/               ← created in Story 1.5
├── src/
│   ├── index.html         ← moved from scaffold root
│   ├── overlay.html       ← stub (minimal HTML, no content)
│   ├── onboarding.html    ← stub (minimal HTML, no content)
│   ├── main.js            ← from scaffold (may be empty or minimal)
│   ├── overlay.js         ← stub (empty)
│   ├── onboarding.js      ← stub (empty)
│   └── styles.css         ← from scaffold (may be empty or minimal)
├── src-tauri/
│   ├── Cargo.toml         ← add tray-icon feature, tracing, thiserror
│   ├── build.rs
│   ├── tauri.conf.json    ← configure main window as hidden, no Dock icon
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   ├── models/            ← empty directory placeholder (Story 5.3 adds model file)
│   └── src/
│       ├── main.rs        ← thin wrapper calling lib::run()
│       ├── lib.rs         ← Tauri Builder, setup hook, tray setup
│       ├── error.rs       ← minimal AppError stub (see above)
│       ├── config.rs      ← comment stub (Story 1.3)
│       ├── audio.rs       ← comment stub (Story 3.2)
│       ├── hotkey.rs      ← comment stub (Story 3.1)
│       ├── injection.rs   ← comment stub (Story 3.4)
│       ├── overlay.rs     ← comment stub (Story 4.1)
│       ├── permissions.rs ← comment stub (Story 2.6)
│       ├── models.rs      ← comment stub (Story 5.3)
│       ├── stt/
│       │   ├── mod.rs     ← comment stub (Story 1.2)
│       │   └── whisper.rs ← comment stub (Story 3.3)
│       └── tests/         ← empty directory (integration tests added in later stories)
├── _bmad-output/          ← DO NOT TOUCH (BMad planning artifacts)
└── .claude/               ← DO NOT TOUCH (BMad skill configuration)
```

**Key violations to avoid:**
- Do NOT add `overlay.html` window to `tauri.conf.json` yet (Story 1.4)
- Do NOT implement any module logic beyond what's listed above (causes scope creep + future merge conflicts)
- Do NOT add `reqwest` or any HTTP client crate (hard privacy constraint)

[Source: architecture.md § Complete Project Directory Structure]

---

### References

- [Source: architecture.md § Starter Template Evaluation] — scaffold command and template selection rationale
- [Source: architecture.md § Implementation Patterns & Consistency Rules] — naming, logging, error, test placement rules
- [Source: architecture.md § Project Structure & Boundaries] — complete directory structure
- [Source: architecture.md § Tauri Window Configuration] — window labels and startup visibility
- [Source: architecture.md § Privacy Boundary] — no HTTP crates constraint
- [Source: epics.md § Story 1.1] — acceptance criteria and user story statement
- [Source: epics.md § Epic 1: App Shell & Foundation] — epic-level objectives and FRs covered (FR17)

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- Fixed `capabilities/default.json`: removed `opener:default` permission (referenced `tauri-plugin-opener` which was not included in this story's dependencies)
- Removed `Manager` import from `lib.rs` — unused, caused `-D warnings` clippy failure
- Scaffold was created in `/tmp/icantspell-scaffold/` (macOS case-insensitive FS prevented creating in Desktop parent)
- `--manager cargo` template places `index.html` already in `src/` (no move needed)
- Tray icon required explicit `.icon()` call on `TrayIconBuilder` — used `app.default_window_icon().unwrap().clone()`; `Image::from_path` does not exist in Tauri v2 API

### Completion Notes List

- Scaffolded Tauri v2 vanilla project (cargo template) and merged into project root alongside `_bmad-output/` and `.claude/`
- Configured `tauri.conf.json`: productName=ICantSpell, identifier=com.icantspell.app, main window hidden (visible:false, decorations:false)
- Implemented tray icon with Quit menu item via `TrayIconBuilder`; macOS activation policy set to `Accessory` (no Dock icon) in `.setup()` hook
- Added `tracing`/`tracing-subscriber`/`thiserror` to Cargo.toml; `tracing_subscriber::fmt::init()` called in `run()` before Tauri builder
- Created all module stubs: `error.rs` (minimal `AppError`), `config.rs`, `audio.rs`, `hotkey.rs`, `injection.rs`, `overlay.rs`, `permissions.rs`, `models.rs`, `stt/mod.rs`, `stt/whisper.rs`
- Created frontend stubs: `src/overlay.html`, `src/onboarding.html`, `src/overlay.js`, `src/onboarding.js`
- Created empty `src-tauri/tests/` and `src-tauri/models/` directories
- `cargo clippy --all-targets -- -D warnings`: zero errors/warnings
- `cargo test`: 0 tests, harness functional

### File List

- `src-tauri/Cargo.toml` (modified — renamed package, added tray-icon feature, tracing, thiserror, removed tauri-plugin-opener)
- `src-tauri/Cargo.lock` (generated)
- `src-tauri/tauri.conf.json` (modified — productName, identifier, hidden main window)
- `src-tauri/capabilities/default.json` (modified — removed opener:default)
- `src-tauri/build.rs` (from scaffold)
- `src-tauri/src/lib.rs` (modified — tray setup, activation policy, tracing init)
- `src-tauri/src/main.rs` (modified — updated crate name to icantspell_lib)
- `src-tauri/src/error.rs` (new — minimal AppError stub)
- `src-tauri/src/config.rs` (new — comment stub)
- `src-tauri/src/audio.rs` (new — comment stub)
- `src-tauri/src/hotkey.rs` (new — comment stub)
- `src-tauri/src/injection.rs` (new — comment stub)
- `src-tauri/src/overlay.rs` (new — comment stub)
- `src-tauri/src/permissions.rs` (new — comment stub)
- `src-tauri/src/models.rs` (new — comment stub)
- `src-tauri/src/stt/mod.rs` (new — comment stub)
- `src-tauri/src/stt/whisper.rs` (new — comment stub)
- `src-tauri/tests/` (new — empty directory)
- `src-tauri/models/` (new — empty directory)
- `src/index.html` (from scaffold)
- `src/main.js` (from scaffold)
- `src/styles.css` (from scaffold)
- `src/overlay.html` (new — stub)
- `src/onboarding.html` (new — stub)
- `src/overlay.js` (new — stub)
- `src/onboarding.js` (new — stub)
- `.gitignore` (from scaffold)
- `README.md` (from scaffold)

## Change Log

- 2026-04-28: Story implemented — Tauri v2 project scaffolded, menu bar tray with Quit item configured, no Dock icon (ActivationPolicy::Accessory), all module stubs created, tracing initialized, cargo clippy -D warnings and cargo test pass (claude-sonnet-4-6)
- 2026-04-28: Post-review cleanup — stripped main.js and index.html of scaffold boilerplate; both reduced to stubs (claude-sonnet-4-6)
