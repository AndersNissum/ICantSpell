# Story 2.2: Accessibility Permission Request

Status: done

## Story

As a new user,
I want the onboarding flow to explain and request Accessibility permission,
so that I understand why it's needed before I grant it and can do so without leaving the wizard.

## Acceptance Criteria

1. **Given** the onboarding wizard is on the Accessibility permission step, **When** the user reads the explanation, **Then** a clear rationale is shown ("needed to inject dictated text into other apps").

2. **Given** the user clicks the grant button, **When** the app calls `request_accessibility_permission`, **Then** macOS opens System Settings > Privacy & Security > Accessibility automatically.

3. **Given** the user grants Accessibility permission and returns to the app, **When** the wizard polls permission status, **Then** the step is marked complete (status message updated) and the wizard advances to the next step.

4. **Given** the user skips or denies the permission, **When** the wizard checks permission status, **Then** it shows a non-blocking warning that voice mode will not work until permission is granted.

## Tasks / Subtasks

- [x] Task 1: Implement `permissions.rs` — check and request functions (AC: 2, 3)
  - [x] Replace stub comment with real module implementation
  - [x] Add `#[link(name = "ApplicationServices", kind = "framework")]` extern block with `AXIsProcessTrusted()` declaration
  - [x] Implement `pub fn check_accessibility() -> bool` (wraps FFI call, `#[cfg]`-gated, non-macOS returns `false`)
  - [x] Implement `pub fn open_accessibility_settings()` — spawns `open x-apple.systempreferences:...` URL, logs warn on failure
  - [x] Add `#[tauri::command] pub async fn check_accessibility_permission() -> Result<bool, String>`
  - [x] Add `#[tauri::command] pub async fn request_accessibility_permission() -> Result<(), String>`
  - [x] Add `#[cfg(test)]` block with `test_check_accessibility_returns_bool` (no-panic test for FFI call)

- [x] Task 2: Wire permissions module and commands into `lib.rs` (AC: 2, 3)
  - [x] Add `pub mod permissions;` to module declarations (after `pub mod stt;`)
  - [x] Add `.invoke_handler(tauri::generate_handler![permissions::check_accessibility_permission, permissions::request_accessibility_permission,])` to builder chain — place between `.plugin(...)` and `.setup(...)`

- [x] Task 3: Add Accessibility step UI to `onboarding.html` (AC: 1, 4)
  - [x] Add `class="active"` to existing `#step-welcome` div (changes `class="step"` → `class="step active"`)
  - [x] Append `<div class="step" id="step-accessibility">` after `#step-welcome`, containing: heading, two explanation paragraphs, `#accessibility-status` div (empty), `#btn-grant-accessibility` button, `#btn-skip-accessibility` button

- [x] Task 4: Implement step navigation and permission flow in `onboarding.js` (AC: 1–4)
  - [x] Add ES module import: `import { invoke } from "@tauri-apps/api/core";`
  - [x] Implement `showStep(stepId)` — removes `active` from all `.step` elements, adds `active` to target
  - [x] Wire `#btn-get-started` click handler to call `showStep("step-accessibility")` (replaces the stub comment from Story 2.1)
  - [x] Implement `startAccessibilityPolling()` — polls `check_accessibility_permission` every 1s, stops on grant OR after 30s timeout, updates `#accessibility-status`
  - [x] Wire `#btn-grant-accessibility` — `invoke("request_accessibility_permission")`, then `startAccessibilityPolling()`
  - [x] Wire `#btn-skip-accessibility` — stop polling, set warning status, log that mic step navigation is pending Story 2.3

- [x] Task 5: Add step navigation CSS to `styles.css` (AC: 1)
  - [x] Add `#onboarding-app .step { display: none; }` and `#onboarding-app .step.active { display: block; }`
  - [x] Add `#onboarding-app .status-warning` (amber/error text color) and `#onboarding-app .status-info` styles

- [x] Task 6: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
  - [x] `cargo test` — 10 tests pass (9 pre-existing + 1 new `test_check_accessibility_returns_bool`)
  - [x] Manual: wizard welcome screen visible on first launch → "Get Started" navigates to Accessibility step → "Grant Accessibility" opens System Settings → granting updates status and logs advance → "Skip for Now" shows warning

## Dev Notes

### Current State of Files Being Modified

**`src-tauri/src/permissions.rs` — current contents (stub only, replace entirely):**
```rust
// Permission revocation monitoring — implemented in Story 2.6
// Uses: macOS CoreLocation/Accessibility APIs
// See architecture.md § Permissions Architecture
```
This story replaces the stub with real implementation. Story 2.6 will add revocation monitoring later.

**`src-tauri/src/lib.rs` — relevant current state:**
```rust
pub mod config;
pub mod error;
pub mod stt;
// ... (no `pub mod permissions;` yet — add it in this story)
```
Builder chain currently: `.plugin(...).setup(...).run(...)`. Add `.invoke_handler(...)` between `.plugin(...)` and `.setup(...)`.

**`src/onboarding.html` — current body:**
```html
<div id="onboarding-app">
  <div class="step" id="step-welcome">
    <h1>Welcome to ICantSpell</h1>
    <p>ICantSpell lets you dictate text by voice…</p>
    <p>Everything runs locally on your Mac…</p>
    <button id="btn-get-started">Get Started →</button>
  </div>
</div>
```
`#step-welcome` currently has no `active` class and `.step` has no hide rule in CSS — so it's visible. This story adds CSS step hiding and the `active` class.

**`src/onboarding.js` — current contents:**
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
Replace the entire file — add import, showStep, and all step wiring.

**`src/styles.css` — existing onboarding CSS (do NOT remove):**
```css
#onboarding-app { padding: 3rem 2rem; max-width: 480px; margin: 0 auto; }
#onboarding-app p { margin: 0.75rem 0; }
#onboarding-app #btn-get-started { margin-top: 1.5rem; }
```
Add new rules after these. Do not remove `#onboarding-app` or any existing rules.

---

### Task 1: Exact `permissions.rs` Implementation

Replace the entire stub file with:

```rust
//! macOS permission checking and request flows.
//! This is the ONLY module that directly calls AXIsProcessTrusted().
//! Revocation monitoring is added in Story 2.6.

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    /// Returns true if the current process is trusted for Accessibility access (no side effects).
    fn AXIsProcessTrusted() -> bool;
}

/// Returns true if Accessibility permission has been granted.
/// On non-macOS platforms, always returns false.
pub fn check_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Safety: AXIsProcessTrusted takes no arguments and returns a simple bool.
        // It queries the macOS TCC database without memory allocation or side effects.
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    false
}

/// Opens System Settings > Privacy & Security > Accessibility so the user can grant access.
/// Fire-and-forget: spawns `open` and returns immediately. Logs a warning on failure.
pub fn open_accessibility_settings() {
    #[cfg(target_os = "macos")]
    {
        let result = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
        if let Err(e) = result {
            tracing::warn!("Failed to open Accessibility settings: {}", e);
        }
    }
}

/// Tauri command: check if Accessibility permission is granted.
#[tauri::command]
pub async fn check_accessibility_permission() -> Result<bool, String> {
    Ok(check_accessibility())
}

/// Tauri command: open System Settings so the user can grant Accessibility permission.
#[tauri::command]
pub async fn request_accessibility_permission() -> Result<(), String> {
    open_accessibility_settings();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_accessibility_returns_bool() {
        // Verifies the FFI call (or non-macOS stub) does not panic.
        // Does not assert true/false — depends on system TCC grant state.
        let _ = check_accessibility();
    }
}
```

**Why `open` URL instead of `AXIsProcessTrustedWithOptions`:**
On macOS 13+ (Ventura/Sonoma), `AXIsProcessTrustedWithOptions({kAXTrustedCheckOptionPrompt: true})` shows a modal alert dialog only (does NOT open System Settings directly). Opening the system preferences URL is the correct and reliable approach for modern macOS. The AC intent — "macOS opens System Settings > Privacy & Security > Accessibility" — is satisfied by this URL.

**Why `open_accessibility_settings` is `pub` (not just used by commands):**
Enables future integration testing and simplifies Story 2.6 when revocation monitoring may need to surface the same Settings link.

---

### Task 2: `lib.rs` Changes

**Add module declaration** (after `pub mod stt;`):
```rust
pub mod permissions;
```

**Add invoke_handler** to builder chain — between `.plugin(...)` and `.setup(...)`:
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_store::Builder::default().build())
    .invoke_handler(tauri::generate_handler![
        permissions::check_accessibility_permission,
        permissions::request_accessibility_permission,
    ])
    .setup(|app| {
        // ... existing setup code unchanged ...
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

**Everything else in `lib.rs` is unchanged:** `is_first_launch`, `Manager` import, tray setup, first-launch detection, existing tests — do NOT touch.

---

### Task 3: `onboarding.html` Changes

**Change 1:** Add `active` class to `#step-welcome`:
```html
<!-- BEFORE -->
<div class="step" id="step-welcome">
<!-- AFTER -->
<div class="step active" id="step-welcome">
```

**Change 2:** Append accessibility step after `#step-welcome`, inside `#onboarding-app`:
```html
<div class="step" id="step-accessibility">
  <h1>Accessibility Permission</h1>
  <p>ICantSpell needs Accessibility permission to inject your dictated text into any app — word processors, browsers, text editors, and more. Without it, voice typing will not work.</p>
  <p>Click "Grant Accessibility" to open System Settings. Add ICantSpell to the Accessibility list, then return to this window.</p>
  <div id="accessibility-status"></div>
  <button id="btn-grant-accessibility">Grant Accessibility</button>
  <button id="btn-skip-accessibility">Skip for Now</button>
</div>
```

---

### Task 4: Complete `onboarding.js` Replacement

```js
// Onboarding wizard — step logic and IPC commands added incrementally in Stories 2.2–2.5
import { invoke } from "@tauri-apps/api/core";

console.debug("[ICantSpell] onboarding window loaded");

// ── Step Navigation ──────────────────────────────────────────────────────────

function showStep(stepId) {
  document.querySelectorAll(".step").forEach((el) => el.classList.remove("active"));
  const target = document.getElementById(stepId);
  if (target) {
    target.classList.add("active");
  } else {
    console.warn(`[ICantSpell] onboarding: step '${stepId}' not found`);
  }
}

// ── Welcome Step ─────────────────────────────────────────────────────────────

const btnGetStarted = document.getElementById("btn-get-started");
if (btnGetStarted) {
  btnGetStarted.addEventListener("click", () => {
    showStep("step-accessibility");
  });
}

// ── Accessibility Step ────────────────────────────────────────────────────────

let accessibilityPollInterval = null;

function stopAccessibilityPolling() {
  if (accessibilityPollInterval !== null) {
    clearInterval(accessibilityPollInterval);
    accessibilityPollInterval = null;
  }
}

function setAccessibilityStatus(message, isWarning = false) {
  const statusEl = document.getElementById("accessibility-status");
  if (statusEl) {
    statusEl.textContent = message;
    statusEl.className = isWarning ? "status-warning" : "status-info";
  }
}

async function startAccessibilityPolling() {
  const btn = document.getElementById("btn-grant-accessibility");
  if (btn) btn.disabled = true;
  setAccessibilityStatus("Waiting for permission…");

  let elapsed = 0;
  const POLL_INTERVAL_MS = 1000;
  const TIMEOUT_MS = 30000;

  accessibilityPollInterval = setInterval(async () => {
    elapsed += POLL_INTERVAL_MS;
    try {
      const granted = await invoke("check_accessibility_permission");
      if (granted) {
        stopAccessibilityPolling();
        if (btn) btn.disabled = false;
        setAccessibilityStatus("Accessibility permission granted ✓");
        console.debug("[ICantSpell] onboarding: accessibility granted — mic step pending Story 2.3");
        // Story 2.3 will implement: showStep("step-microphone");
      } else if (elapsed >= TIMEOUT_MS) {
        stopAccessibilityPolling();
        if (btn) btn.disabled = false;
        setAccessibilityStatus(
          "Permission not yet granted. Voice typing will not work until you grant it in System Settings.",
          true
        );
      }
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error checking accessibility permission:", e);
    }
  }, POLL_INTERVAL_MS);
}

const btnGrantAccessibility = document.getElementById("btn-grant-accessibility");
if (btnGrantAccessibility) {
  btnGrantAccessibility.addEventListener("click", async () => {
    try {
      await invoke("request_accessibility_permission");
      await startAccessibilityPolling();
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error requesting accessibility permission:", e);
    }
  });
}

const btnSkipAccessibility = document.getElementById("btn-skip-accessibility");
if (btnSkipAccessibility) {
  btnSkipAccessibility.addEventListener("click", () => {
    stopAccessibilityPolling();
    setAccessibilityStatus(
      "Skipped. Voice typing will not work until Accessibility permission is granted in System Settings.",
      true
    );
    console.debug("[ICantSpell] onboarding: accessibility skipped — mic step pending Story 2.3");
    // Story 2.3 will implement: showStep("step-microphone");
  });
}
```

**Why replace the whole file:** The import must be at the top of the module. Restructuring around a new `showStep` function is cleaner than patching the old stub.

---

### Task 5: `styles.css` Additions

Append after the existing `#onboarding-app #btn-get-started` rule, before the `@media` block:

```css
#onboarding-app .step {
  display: none;
}

#onboarding-app .step.active {
  display: block;
}

#onboarding-app .status-warning {
  color: #c84b00;
  margin: 0.75rem 0;
  font-size: 0.9em;
}

#onboarding-app .status-info {
  color: #555;
  margin: 0.75rem 0;
  font-size: 0.9em;
}

#onboarding-app #btn-skip-accessibility {
  margin-top: 0.5rem;
  margin-left: 0.5rem;
}
```

**Dark mode note:** `#c84b00` for warning text is readable on both light (`#f6f6f6`) and dark (`#2f2f2f`) backgrounds. No separate dark-mode override needed.

---

### Architecture Compliance

- `permissions.rs` is the **only** module calling `AXIsProcessTrusted()` — architecture rule enforced.
- All Tauri commands are `async fn` returning `Result<T, String>` — matches architecture pattern.
- `std::process::Command` used for `open` call — no HTTP client crates added.
- IPC command names `check_accessibility_permission` and `request_accessibility_permission` use `snake_case` — matches naming rules.
- `tracing::warn!` used for errors — no `println!` or `unwrap()` outside tests.
- Frontend uses `invoke()` from `@tauri-apps/api/core` — correct Tauri v2 import (NOT from `@tauri-apps/api/tauri`).
- The `capabilities/default.json` already covers the onboarding window with `core:default` — no changes needed for `invoke()` access.

### What NOT to Do

- Do NOT implement microphone permission request — that is Story 2.3.
- Do NOT implement hotkey capture — that is Story 2.4.
- Do NOT add `showStep("step-microphone")` call — `#step-microphone` does not exist yet; stub the navigation with a comment.
- Do NOT add `pub mod audio;`, `pub mod hotkey;`, `pub mod injection;`, `pub mod overlay;` — those stubs are not wired until their respective stories.
- Do NOT add a "Finish" or "Done" button — that is Story 2.5.
- Do NOT add revocation monitoring to `permissions.rs` — that is Story 2.6.
- Do NOT use `AXIsProcessTrustedWithOptions` with CFDictionary — requires `core-foundation` crate and is incorrect on macOS 13+; the URL approach satisfies the spec intent.
- Do NOT add `core-foundation` or `accessibility` crates to `Cargo.toml` — not needed.
- Do NOT modify `tauri.conf.json` — windows are already configured from Story 1.4.
- Do NOT modify `capabilities/default.json` — `core:default` already allows invoke for all windows.

### Files to Touch

| File | Action | Why |
|------|---------|-----|
| `src-tauri/src/permissions.rs` | REPLACE | Implement accessibility check/request from stub |
| `src-tauri/src/lib.rs` | MODIFY | Add `pub mod permissions;` + `invoke_handler` with 2 commands |
| `src/onboarding.html` | MODIFY | Add `active` to step-welcome; add step-accessibility div |
| `src/onboarding.js` | REPLACE | Add import, showStep, step wiring, polling logic |
| `src/styles.css` | MODIFY | Add step show/hide rules and status text styles |

**Files NOT touched:**
- `src-tauri/Cargo.toml` — no new crates needed
- `src-tauri/tauri.conf.json` — already correct from Story 1.4
- `src-tauri/capabilities/default.json` — already covers onboarding window with core:default
- `src-tauri/src/config.rs`, `error.rs`, `stt/` — no changes
- All other stub modules (`audio.rs`, `hotkey.rs`, `injection.rs`, `overlay.rs`, `models.rs`)

### Test Count

| Scope | Count |
|-------|-------|
| Pre-existing (config + stt + is_first_launch) | 9 |
| New: `test_check_accessibility_returns_bool` in `permissions.rs` | 1 |
| **Total** | **10** |

The new test runs on all platforms (macOS returns actual TCC result, non-macOS returns `false`). It verifies the FFI binding does not panic, not a specific permission state.

### Previous Story Intelligence (Story 2.1)

- `lib.rs` module order: `config`, `error`, `stt` — add `permissions` as 4th.
- Story 2.1 deliberately left permissions.rs untouched. This story activates it.
- `#step-welcome` in `onboarding.html` currently has `class="step"` only — add `active` (do NOT remove `step`).
- `styles.css` existing onboarding rules are scoped to `#onboarding-app` — follow same pattern.
- The `is_first_launch` function and test block are at the bottom of `lib.rs` — do NOT disturb.
- All 9 existing tests pass with zero clippy warnings. `cargo clippy --all-targets -- -D warnings` must remain green after this story.

### References

- [Source: epics.md § Story 2.2] — User story, all 4 acceptance criteria
- [Source: architecture.md § Structure Patterns] — `permissions.rs` is the sole AXIsProcessTrusted() caller
- [Source: architecture.md § Format Patterns] — `async fn`, `Result<T, String>` at command boundary
- [Source: architecture.md § Naming Patterns] — `snake_case` IPC command names
- [Source: architecture.md § Logging Patterns] — `tracing::warn!` for recoverable issues
- [Source: architecture.md § Frontend Architecture] — `invoke()` from `@tauri-apps/api/core`; no direct OS API calls from JS
- [Source: architecture.md § Enforcement Guidelines] — no `unwrap()` outside tests, no `println!`
- [Source: story 2-1 Completion Notes] — 9 tests baseline; `styles.css` scoped to `#onboarding-app`; permissions.rs deliberately left as stub
- [Source: story 2-1 Dev Notes § What NOT to Do] — "Do NOT add `pub mod permissions;` to `lib.rs` — permissions.rs will be wired up in Story 2.2/2.6"
- [Source: story 1-4 Dev Notes] — tauri.conf.json window labels; onboarding window is label "onboarding"

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

_None — clean implementation, no issues encountered._

### Completion Notes List

- Replaced `permissions.rs` stub with full implementation: `AXIsProcessTrusted()` FFI via `#[link(name = "ApplicationServices", kind = "framework")]` extern block, `#[cfg]`-gated for macOS, returns `false` on non-macOS for CI compatibility
- `open_accessibility_settings()` uses `std::process::Command::new("open")` with the system preferences URL — correct approach for macOS 13+ (Ventura/Sonoma) where `AXIsProcessTrustedWithOptions` no longer opens Settings directly
- Two Tauri commands: `check_accessibility_permission() -> Result<bool, String>` and `request_accessibility_permission() -> Result<(), String>` — both `async fn` per architecture rule
- `pub mod permissions;` added to `lib.rs` module declarations; `.invoke_handler(tauri::generate_handler![...])` added between `.plugin(...)` and `.setup(...)` — first commands registered in the app
- `onboarding.html`: added `active` class to `#step-welcome`; appended `#step-accessibility` div with heading, two explanation paragraphs, `#accessibility-status` div, Grant and Skip buttons
- `onboarding.js`: full replacement — added `@tauri-apps/api/core` ES module import, `showStep()` step navigation function, wired Get Started → accessibility step, implemented 1s polling loop with 30s timeout, skip flow with non-blocking warning; next-step navigation stubbed with comments pending Story 2.3
- `styles.css`: added `#onboarding-app .step { display: none }` / `.step.active { display: block }` step show/hide rules; `status-warning` (amber) and `status-info` text styles; skip button margin rule
- 10/10 tests pass (9 pre-existing + 1 new `test_check_accessibility_returns_bool`); `cargo clippy --all-targets -- -D warnings` — zero warnings

### File List

- `src-tauri/src/permissions.rs` — replaced stub with full implementation: FFI extern block, check_accessibility, open_accessibility_settings, two Tauri commands, unit test
- `src-tauri/src/lib.rs` — added `pub mod permissions;` and `.invoke_handler(...)` with two permission commands
- `src/onboarding.html` — added `active` class to `#step-welcome`; appended `#step-accessibility` step div
- `src/onboarding.js` — full replacement: ES import, showStep(), step wiring, accessibility polling logic
- `src/styles.css` — added step show/hide rules and status text styles

### Review Findings

- [x] [Review][Patch] Multiple Grant clicks can stack orphaned polling intervals [src/onboarding.js:~73-80] — fixed: added `stopAccessibilityPolling()` at top of `startAccessibilityPolling()`
- [x] [Review][Patch] Polling timeout never evaluated on IPC exception path [src/onboarding.js:~55-70] — fixed: moved timeout check after try/catch so it runs regardless of IPC success/failure
- [x] [Review][Patch] Dark mode missing overrides for `.status-warning` and `.status-info` [src/styles.css] — fixed: added `#ff8c42` warning and `#aaa` info color overrides in dark mode media query
- [x] [Review][Defer] FFI `bool` vs `u8` for `AXIsProcessTrusted` return type [src-tauri/src/permissions.rs:8] — deferred, spec-prescribed declaration; Apple guarantees 0/1 return
- [x] [Review][Defer] `is_first_launch` and onboarding show logic is Story 2.1 scope [src-tauri/src/lib.rs] — deferred, pre-existing (Story 2.1)
- [x] [Review][Defer] Pre-existing `.expect()` in `run()` — technically unwrap outside tests [src-tauri/src/lib.rs] — deferred, pre-existing
- [x] [Review][Defer] `is_first_launch` checks `settings.json` path vs tauri-plugin-store actual on-disk path [src-tauri/src/lib.rs] — deferred, Story 2.1 scope

## Change Log

- 2026-04-29: Story 2.2 created — Accessibility Permission Request ready for dev.
- 2026-04-29: Implemented Story 2.2 — permissions.rs FFI implementation (AXIsProcessTrusted, open URL to System Settings), lib.rs wired with pub mod and invoke_handler, onboarding multi-step navigation with CSS show/hide, accessibility permission flow with 1s polling and 30s timeout. 10/10 tests pass, zero clippy warnings.
