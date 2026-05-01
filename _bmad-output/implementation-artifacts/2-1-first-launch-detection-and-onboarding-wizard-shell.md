# Story 2.1: First-Launch Detection & Onboarding Wizard Shell

Status: review

## Story

As a new user,
I want the app to detect my first launch and open an onboarding wizard automatically,
so that I know what to do next without having to find any documentation.

## Acceptance Criteria

1. **Given** no settings file exists in `~/Library/Application Support/com.icantspell.app/`, **When** the app launches, **Then** the onboarding window is shown automatically.

2. **Given** the onboarding wizard opens, **When** it renders, **Then** it presents a welcome screen explaining what ICantSpell does in plain language (voice dictation, local-only, no data leaves device).

3. **Given** a settings file already exists from a previous launch, **When** the app launches, **Then** the onboarding window is NOT shown.

4. **And** the onboarding wizard is implemented in `onboarding.html` / `onboarding.js` using the existing window from Story 1.4.

## Tasks / Subtasks

- [x] Task 1: Add first-launch detection and onboarding window show to `lib.rs` (AC: 1, 3)
  - [x] Import `tauri::Manager` trait in `lib.rs` (needed for `get_webview_window()` and `path()`)
  - [x] Before calling `config::ensure_defaults`, check if `settings.json` exists in `app.path().app_data_dir()`; store result as `is_first_launch: bool`
  - [x] After `config::ensure_defaults` call, if `is_first_launch` is true: call `app.get_webview_window("onboarding")` and `win.show()` on the result
  - [x] Log `tracing::info!("First launch detected — showing onboarding wizard")` when showing onboarding
  - [x] Handle the `Option` from `get_webview_window` with `if let Some(win)` + `tracing::warn!` on the None branch

- [x] Task 2: Implement welcome screen in `onboarding.html` and `onboarding.js` (AC: 2, 4)
  - [x] Replace body stub comment in `onboarding.html` with a `<div id="onboarding-app">` containing a `<div class="step" id="step-welcome">` with heading, two paragraph explanations, and a "Get Started →" button
  - [x] Add minimal inline or `styles.css`-based styles if needed for readability (do NOT modify `styles.css` if it would affect other windows — scope styles to `#onboarding-app`)
  - [x] In `onboarding.js`, replace the `console.debug` stub with: select the `#btn-get-started` button and add a click listener that logs a debug message noting step navigation is pending (Stories 2.2–2.4)

- [x] Task 3: Write unit test for first-launch detection logic (AC: 1, 3)
  - [x] Extract a standalone `fn is_first_launch(data_dir: &std::path::Path) -> bool` in `lib.rs` (or a new `onboarding_support.rs` — see Dev Notes)
  - [x] Add `#[cfg(test)]` block with two tests: one for a path where settings.json does NOT exist (returns `true`), one for a path where it DOES exist (returns `false`)

- [x] Task 4: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
  - [x] `cargo test` — all 7 existing tests pass plus 2 new tests for first-launch detection (9 total)
  - [ ] Manual: `cargo tauri dev` — on first run (delete `~/Library/Application Support/com.icantspell.app/settings.json` if it exists) the onboarding window appears automatically; on subsequent runs it does NOT appear

## Dev Notes

### Current State of Files Being Modified

**`src-tauri/src/lib.rs` — current contents (read this first, do NOT guess):**
```rust
pub mod config;
pub mod error;
pub mod stt;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

pub fn run() {
    tracing_subscriber::fmt::init();
    tracing::info!("ICantSpell starting up");

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_item])?;
            let icon = app.default_window_icon().unwrap().clone();
            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            config::ensure_defaults(app.handle())?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**`src/onboarding.html` — current contents:**
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

**`src/onboarding.js` — current contents:**
```js
// Onboarding wizard step logic — permission prompts added in Story 2.1
console.debug("[ICantSpell] onboarding window loaded");
```

**`src-tauri/src/permissions.rs` — stub only, do NOT modify for this story:**
```rust
// Permission revocation monitoring — implemented in Story 2.6
// Uses: macOS CoreLocation/Accessibility APIs
// See architecture.md § Permissions Architecture
```

### Task 1: Exact `lib.rs` Changes

**What changes and what does NOT change:**

Add `tauri::Manager` to the import (needed for `get_webview_window()` and `app.path()`). Add first-launch detection in the setup hook **before** `config::ensure_defaults`. Show the onboarding window after `ensure_defaults` if first launch. Everything else in `lib.rs` is untouched.

**Updated import:**
```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
```

**Updated setup hook body (only the end of `.setup()` changes — tray setup is unchanged):**
```rust
.setup(|app| {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_item])?;
    let icon = app.default_window_icon().unwrap().clone();
    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if event.id().as_ref() == "quit" {
                app.exit(0);
            }
        })
        .build(app)?;

    // First-launch detection: check BEFORE ensure_defaults writes settings.json
    let data_dir = app.path().app_data_dir()?;
    let first_launch = is_first_launch(&data_dir);

    config::ensure_defaults(app.handle())?;

    if first_launch {
        tracing::info!("First launch detected — showing onboarding wizard");
        if let Some(win) = app.get_webview_window("onboarding") {
            win.show()?;
        } else {
            tracing::warn!("Onboarding window not found in Tauri config");
        }
    }

    Ok(())
})
```

**The helper function (add below `pub fn run()` in `lib.rs`):**
```rust
/// Returns true if this is the first launch (settings file has never been written).
/// Extracted for testability — callers should use this rather than inlining the path check.
fn is_first_launch(data_dir: &std::path::Path) -> bool {
    !data_dir.join("settings.json").exists()
}
```

**Why check BEFORE `ensure_defaults`:**
`config::ensure_defaults()` calls `config::save()` which creates `settings.json` on disk. If we check after, the file always exists and first-launch is never detected. The detection MUST happen before the first write.

**Why `if let Some(win)` rather than `unwrap()`:**
The window is defined in `tauri.conf.json` and will always exist at runtime, but `get_webview_window` returns `Option`. Using `if let` with a warn log on None is defensive and satisfies the "no unwrap outside tests" architecture rule.

**Tauri v2 `app.path()` note:**
In Tauri v2, `app.path()` returns a `PathResolver`. `app_data_dir()` returns `Result<PathBuf, tauri::Error>`, so `?` propagates through the setup hook's error type correctly. The resolved path is `~/Library/Application Support/com.icantspell.app/` (using the bundle identifier, NOT the product name — confirmed in Story 1.3).

### Task 2: Onboarding Welcome Screen

**Updated `onboarding.html` body section:**
```html
<body>
  <div id="onboarding-app">
    <div class="step" id="step-welcome">
      <h1>Welcome to ICantSpell</h1>
      <p>ICantSpell lets you dictate text by voice — hold a key, speak, and your words appear wherever you're typing, in any app.</p>
      <p>Everything runs locally on your Mac. No internet connection. No cloud service. Your voice never leaves your device.</p>
      <button id="btn-get-started">Get Started →</button>
    </div>
  </div>
  <script type="module" src="/onboarding.js"></script>
</body>
```

**Updated `onboarding.js`:**
```js
// Onboarding wizard — step logic and IPC commands added incrementally in Stories 2.2–2.5
console.debug("[ICantSpell] onboarding window loaded");

const btnGetStarted = document.getElementById("btn-get-started");
if (btnGetStarted) {
  btnGetStarted.addEventListener("click", () => {
    // Step navigation to permissions will be implemented in Story 2.2
    console.debug("[ICantSpell] onboarding: Get Started clicked — step navigation pending");
  });
}
```

**CSS — what's already in `styles.css` (do NOT re-add):**
- `:root` — font-family, font-size 16px, line-height 24px, color `#0f0f0f`, bg `#f6f6f6`, dark mode via `prefers-color-scheme`
- `h1` — `text-align: center`
- `button` — `border-radius: 8px`, `padding: 0.6em 1.2em`, `box-shadow`, hover/active states — already styled
- `.container` — `padding-top: 10vh`, flex column, centered — available for reuse

**CSS scoping recommendation:**
The welcome screen will look acceptable with existing styles alone (centered h1, styled button). To add page padding and center the content vertically, add `#onboarding-app` to `styles.css`:
```css
#onboarding-app {
  padding: 3rem 2rem;
  max-width: 480px;
  margin: 0 auto;
}
#onboarding-app p {
  margin: 0.75rem 0;
}
#onboarding-app #btn-get-started {
  margin-top: 1.5rem;
}
```
These selectors are new and do NOT conflict with any existing selectors in `styles.css`.

### Task 3: Unit Tests for `is_first_launch`

Add to `lib.rs` at the bottom (inside `#[cfg(test)]`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_first_launch_when_settings_absent() {
        let dir = std::env::temp_dir().join(format!("icantspell_test_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()));
        fs::create_dir_all(&dir).unwrap();
        assert!(is_first_launch(&dir), "should be first launch when settings.json absent");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_is_not_first_launch_when_settings_present() {
        let dir = std::env::temp_dir().join(format!("icantspell_test_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("settings.json"), b"{}").unwrap();
        assert!(!is_first_launch(&dir), "should NOT be first launch when settings.json exists");
        fs::remove_dir_all(&dir).unwrap();
    }
}
```

**Test naming note:** If `lib.rs` already has a `mod tests` block from a prior story addition, merge the new tests into it rather than creating a duplicate `mod tests` block.

### Clippy Guardrails

- `Manager` import: used by `app.get_webview_window()` and `app.path()` — not unused.
- `is_first_launch()` function: called in `run()` and in `#[cfg(test)]` — not unused.
- `tracing::info!` and `tracing::warn!` calls: consistent with established logging pattern.
- `fn is_first_launch` is not `pub` — it's an internal helper. Clippy will not warn on dead code since it's called within the same file.
- Do NOT use `println!` or `unwrap()` outside of test blocks.

### Architecture Compliance

- `onboarding.html` / `onboarding.js` use the existing window defined in `tauri.conf.json` (label `"onboarding"`) — no new window is created (satisfies architecture constraint "pre-created hidden window").
- The window is shown via `win.show()` (Tauri WebviewWindow API) — never spawned on demand.
- No Tauri commands are added in this story. The `show()` is triggered from the Rust setup hook, not from JS.
- No `permissions.rs` changes — that module is untouched until Story 2.2.
- No new Rust dependencies required.
- No HTTP calls introduced.
- IPC event names (none in this story) would follow `snake_case` — no violations.

### What NOT to Do

- Do NOT add `pub mod permissions;` to `lib.rs` — permissions.rs will be wired up in Story 2.2/2.6.
- Do NOT implement permission request buttons in the onboarding JS — that's Stories 2.2 and 2.3.
- Do NOT implement hotkey capture UI — that's Story 2.4.
- Do NOT add a "Finish" or "Skip" button that saves onboarding state — that's Story 2.5.
- Do NOT add a `close_onboarding` Tauri command — not needed until Story 2.5.
- Do NOT put step navigation skeleton beyond the "Get Started" button click handler.
- Do NOT add a `has_completed_onboarding` field to `Settings` — not in spec; use file-existence check per AC wording.

### Files to Touch

| File | Action | Why |
|------|---------|-----|
| `src-tauri/src/lib.rs` | MODIFY | Add `Manager` import, `is_first_launch` fn, first-launch detection + window show in setup, unit tests |
| `src/onboarding.html` | MODIFY | Replace body stub comment with welcome screen HTML |
| `src/onboarding.js` | MODIFY | Replace stub with button listener |
| `src/styles.css` | MAYBE MODIFY | Only if onboarding elements need layout/padding not covered by existing styles |

**Files NOT touched:**
- `src-tauri/src/permissions.rs` — stub, leave untouched
- `src-tauri/src/config.rs` — no changes
- `src-tauri/src/error.rs` — no new error variants needed
- `src-tauri/tauri.conf.json` — already correct from Story 1.4; onboarding window already defined
- `src-tauri/capabilities/default.json` — already covers all three windows from Story 1.4
- `src/index.html`, `src/overlay.html` — no changes

### References

- [Source: epics.md § Story 2.1] — Acceptance criteria, user story, "And the onboarding wizard is implemented in onboarding.html / onboarding.js using the existing window from Story 1.4"
- [Source: architecture.md § Frontend Architecture] — "Onboarding wizard: shown as primary window on first launch only"
- [Source: architecture.md § Data Architecture] — "Storage: JSON file in ~/Library/Application Support/icantspell/" (actual path uses bundle identifier `com.icantspell.app`)
- [Source: architecture.md § Implementation Patterns] — Manager trait, no unwrap(), tracing logging
- [Source: architecture.md § Rust Module Organization] — permissions.rs for permission checking; lib.rs for app startup orchestration
- [Source: story 1-3 Completion Notes] — "actual path is ~/Library/Application Support/com.icantspell.app/settings.json" (bundle identifier, not product name)
- [Source: story 1-4 Dev Notes] — onboarding window config: label "onboarding", url "onboarding.html", visible: false, resizable: false, center: true
- [Source: story 1-4 Completion Notes] — "7/7 tests pass (5 config + 2 stt), no regressions" — these 7 tests must still pass

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

_None — clean implementation, no issues encountered._

### Completion Notes List

- Added `tauri::Manager` to imports in `lib.rs` (required for `app.get_webview_window()` and `app.path()`)
- Extracted `fn is_first_launch(data_dir: &std::path::Path) -> bool` helper — checks for absence of `settings.json` in the app data dir; testable without an app handle
- First-launch detection placed in setup hook BEFORE `config::ensure_defaults` so the file-existence check happens before `ensure_defaults` writes `settings.json`
- After `ensure_defaults`, shows the pre-created `onboarding` window (from Story 1.4) via `win.show()` when first launch detected; uses `if let Some(win)` to handle `Option` without unwrap
- Replaced `onboarding.html` body stub with full welcome screen: `#onboarding-app` wrapper, `#step-welcome` div, `<h1>`, two `<p>` tags, and `#btn-get-started` button
- Updated `onboarding.js` with `#btn-get-started` click listener that logs a debug message (step navigation pending for Stories 2.2–2.4)
- Added `#onboarding-app`, `#onboarding-app p`, and `#onboarding-app #btn-get-started` CSS rules to `styles.css` — scoped selectors, no conflict with existing windows
- 9/9 tests pass (7 pre-existing + 2 new `is_first_launch` tests); `cargo clippy --all-targets -- -D warnings` — zero warnings

### File List

- `src-tauri/src/lib.rs` — added `Manager` import, `is_first_launch` helper fn, first-launch detection + onboarding window show in setup hook, `#[cfg(test)]` block with 2 new tests
- `src/onboarding.html` — replaced body stub comment with welcome screen HTML (`#onboarding-app`, `#step-welcome`, heading, paragraphs, button)
- `src/onboarding.js` — replaced stub with `#btn-get-started` click listener
- `src/styles.css` — added `#onboarding-app` layout rules (padding, max-width, margin) scoped to onboarding window

## Change Log

- 2026-04-29: Story 2.1 created — First-Launch Detection & Onboarding Wizard Shell ready for dev.
- 2026-04-29: Implemented Story 2.1 — First-launch detection in lib.rs (is_first_launch helper, Manager import, setup hook detection + window show), welcome screen in onboarding.html/onboarding.js, scoped CSS in styles.css. 9/9 tests pass, zero clippy warnings.
