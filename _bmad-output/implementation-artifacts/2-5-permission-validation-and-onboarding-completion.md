# Story 2.5: Permission Validation & Onboarding Completion

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a new user,
I want the wizard to validate that required permissions are in place before finishing,
so that the app is confirmed ready to use when onboarding closes.

## Acceptance Criteria

1. **Given** the onboarding wizard reaches its final step (`#step-validation`), **When** it checks Accessibility and Microphone permission status, **Then** it displays a "Ready" state if both are granted, or a warning listing any missing permissions.

2. **Given** both permissions are granted, **When** the user clicks "Finish Setup", **Then** the onboarding window closes, the menu bar popover is ready, and a macOS system notification confirms the app is active.

3. **Given** one or more permissions are missing, **When** the user clicks "Finish Anyway", **Then** the onboarding window closes but voice mode remains disabled until the missing permission is granted (FR24).

4. **And** after onboarding completion, the settings file exists so onboarding does not show on next launch — this is already guaranteed by `config::ensure_defaults()` called during app startup (Story 2.1); no new implementation required for this AC.

## Tasks / Subtasks

- [x] Task 1: Add `PermissionsStatus` struct and `check_all_permissions` command in `permissions.rs` (AC: 1)
  - [x] Add `#[derive(Debug, serde::Serialize)] pub struct PermissionsStatus { pub accessibility: bool, pub microphone: bool }` above the `#[cfg(test)]` block
  - [x] Add `#[tauri::command] pub async fn check_all_permissions() -> Result<PermissionsStatus, String>` calling `check_accessibility()` and `check_microphone()`
  - [x] Add `#[test] fn test_permissions_status_fields()` verifying struct serializes with expected field names

- [x] Task 2: Add `finish_onboarding` command in `lib.rs` and register both new commands (AC: 2, 3)
  - [x] Add `async fn finish_onboarding(app: tauri::AppHandle, all_granted: bool) -> Result<(), String>` before `pub fn run()`: closes the `onboarding` window via `app.get_webview_window("onboarding")`, then if `all_granted` sends a macOS notification via `osascript` using `std::process::Command`
  - [x] Add `permissions::check_all_permissions` and `finish_onboarding` to the existing `invoke_handler` (7 commands total — do NOT create a second handler)

- [x] Task 3: Add `#step-validation` div to `onboarding.html` (AC: 1, 2, 3)
  - [x] Append inside `#onboarding-app`, directly after the closing `</div>` of `#step-hotkey`
  - [x] Include: `<h1>`, `<div id="validation-status">`, `<button id="btn-finish">` (initially hidden), `<button id="btn-finish-anyway">` (initially hidden)

- [x] Task 4: Wire validation step in `onboarding.js` (AC: 1, 2, 3)
  - [x] Replace both `// Story 2.5 will implement: showStep("step-validation");` stubs with `showStep("step-validation");` (two occurrences in hotkey step confirm/skip handlers — verify with grep before editing)
  - [x] Add `showStep` observer: call `loadValidationStep()` when `#step-validation` becomes active — use a `MutationObserver` watching `#step-validation`'s `classList` for `active`
  - [x] Implement `loadValidationStep()`: invokes `check_all_permissions()`, renders permission items in `#validation-status`, shows `#btn-finish` if both granted or `#btn-finish-anyway` if any missing
  - [x] Wire `#btn-finish`: invokes `finish_onboarding({ allGranted: true })`
  - [x] Wire `#btn-finish-anyway`: invokes `finish_onboarding({ allGranted: false })`

- [x] Task 5: Add validation step CSS to `styles.css` (AC: 1)
  - [x] Add `.permission-item` block: `display: flex`, `align-items: center`, `margin: 0.4rem 0`
  - [x] Add `.permission-item .granted` color (green) and `.permission-item .missing` color (amber/red)
  - [x] Add `#btn-finish` and `#btn-finish-anyway` margin rules
  - [x] Insert all new rules BEFORE the `@media (prefers-color-scheme: dark)` block; add dark mode variants for granted/missing colors inside the existing `@media` block

- [x] Task 6: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
  - [x] `cargo test` — 13 tests pass (12 pre-existing + 1 new `test_permissions_status_fields`)
  - [x] Manual: hotkey confirm/skip → validation step appears → permission items render with correct status → "Finish Setup" / "Finish Anyway" shows based on grant state → finish closes window

## Dev Notes

### No New Crates Required

All implementation uses existing infrastructure:
- `check_accessibility()` and `check_microphone()` already in `permissions.rs` — `check_all_permissions` just calls them
- Window close via `app.get_webview_window("onboarding")?.close()` — Tauri 2 built-in
- macOS notification via `std::process::Command::new("osascript")` — same `std::process::Command` pattern already used in `permissions.rs` for opening System Settings
- No `tauri-plugin-notification`, no new crate, no new `Cargo.toml` entries

### AC 4 Is Already Implemented

`config::ensure_defaults()` is called in `lib.rs::run()` setup closure **before** the first-launch check. It writes `settings.json` to `~/Library/Application Support/icantspell/`. This means the file exists after first run, so the onboarding will not show on subsequent launches. **Nothing to implement for AC 4** — just verify in the story test/manual check that settings.json exists after onboarding finishes.

### Current State of Files Being Modified

**`src-tauri/src/permissions.rs` — current state (after Story 2.3):**
```
// Exports: check_accessibility(), open_accessibility_settings(),
//          check_accessibility_permission (cmd), request_accessibility_permission (cmd),
//          check_microphone(), request_microphone_access(),
//          check_microphone_permission (cmd), request_microphone_permission (cmd)
// Tests: test_check_accessibility_returns_bool, test_check_microphone_returns_bool
```
Add `PermissionsStatus` struct and `check_all_permissions` command AFTER `request_microphone_permission` and BEFORE `#[cfg(test)]`.

**`src-tauri/src/lib.rs` — current `invoke_handler` (after Story 2.4):**
```rust
.invoke_handler(tauri::generate_handler![
    permissions::check_accessibility_permission,
    permissions::request_accessibility_permission,
    permissions::check_microphone_permission,
    permissions::request_microphone_permission,
    config::save_ptt_hotkey,
])
```
`Manager` trait is already imported (`use tauri::Manager`). Add `finish_onboarding` as a bare `async fn` in `lib.rs` (before `pub fn run()`); reference it without a module prefix in the handler.

**`src/onboarding.html` — current body (after Story 2.4):**
```html
<div id="onboarding-app">
  <div class="step active" id="step-welcome"> ... </div>
  <div class="step" id="step-accessibility"> ... </div>
  <div class="step" id="step-microphone"> ... </div>
  <div class="step" id="step-hotkey"> ... </div>
</div>
```
Append `#step-validation` inside `#onboarding-app` after `#step-hotkey`.

**`src/onboarding.js` — current state (after Story 2.4):**
Two stub comments to replace (lines ~308 and ~320):
```js
// Story 2.5 will implement: showStep("step-validation");
```
One inside `btnConfirmHotkey` click handler, one inside `btnSkipHotkey` click handler. Replace BOTH.

**`src/styles.css` — current onboarding rules (after Story 2.4):**
Last rules before `@media (prefers-color-scheme: dark)` are the hotkey field/button rules. Insert validation CSS after them.

---

### Task 1: Exact `permissions.rs` Additions

After `request_microphone_permission` and before `#[cfg(test)]`:

```rust
// ── Combined Permission Check ─────────────────────────────────────────────────

/// Snapshot of both required permission states, returned to the onboarding
/// validation step. Serialized by Tauri to `{ "accessibility": bool, "microphone": bool }`.
#[derive(Debug, serde::Serialize)]
pub struct PermissionsStatus {
    pub accessibility: bool,
    pub microphone: bool,
}

/// Tauri command: check both Accessibility and Microphone permissions in one call.
/// Used by the onboarding validation step to determine final setup state.
#[tauri::command]
pub async fn check_all_permissions() -> Result<PermissionsStatus, String> {
    Ok(PermissionsStatus {
        accessibility: check_accessibility(),
        microphone: check_microphone(),
    })
}
```

Add one test inside the existing `#[cfg(test)]` block:
```rust
    #[test]
    fn test_permissions_status_fields() {
        // Verifies the struct constructs and serializes with the expected JSON field names.
        let status = PermissionsStatus { accessibility: true, microphone: false };
        let json = serde_json::to_value(&status).expect("serialize ok");
        assert_eq!(json["accessibility"], true);
        assert_eq!(json["microphone"], false);
    }
```

---

### Task 2: `lib.rs` Changes

**Add `finish_onboarding` before `pub fn run()`:**
```rust
/// Tauri command: close the onboarding window and optionally notify the user.
///
/// Called by the onboarding validation step's Finish buttons.
/// - `all_granted = true`:  both permissions in place → close window + send macOS notification
/// - `all_granted = false`: permissions missing → close window only (voice mode stays disabled)
///
/// The macOS notification uses `osascript` via `std::process::Command` — the same fire-and-forget
/// pattern used in `permissions.rs` for opening System Settings. No new crate required.
#[tauri::command]
async fn finish_onboarding(app: tauri::AppHandle, all_granted: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("onboarding") {
        win.close().map_err(|e| e.to_string())?;
    }
    if all_granted {
        #[cfg(target_os = "macos")]
        {
            let result = std::process::Command::new("osascript")
                .arg("-e")
                .arg(r#"display notification "Hold your PTT key to start dictating." with title "ICantSpell is ready""#)
                .spawn();
            if let Err(e) = result {
                tracing::warn!("Failed to send completion notification: {}", e);
            }
        }
    }
    tracing::info!(all_granted = all_granted, "Onboarding completed");
    Ok(())
}
```

**Update the `invoke_handler`** (add two commands to the existing list):
```rust
.invoke_handler(tauri::generate_handler![
    permissions::check_accessibility_permission,
    permissions::request_accessibility_permission,
    permissions::check_microphone_permission,
    permissions::request_microphone_permission,
    permissions::check_all_permissions,
    config::save_ptt_hotkey,
    finish_onboarding,
])
```

**Nothing else in `lib.rs` changes.** `Manager` is already imported; `std::process::Command` is in std and needs no import.

---

### Task 3: `onboarding.html` Addition

Append inside `#onboarding-app`, after the closing `</div>` of `#step-hotkey`:
```html
    <div class="step" id="step-validation">
      <h1>Setup Complete</h1>
      <p>Here's your permission summary before you start.</p>
      <div id="validation-status"></div>
      <button id="btn-finish" style="display:none">Finish Setup</button>
      <button id="btn-finish-anyway" style="display:none">Finish Anyway</button>
    </div>
```

---

### Task 4: `onboarding.js` Changes

**Change 1 — Replace both stubs:**

Inside `btnConfirmHotkey` click handler:
```js
// BEFORE:
// Story 2.5 will implement: showStep("step-validation");
// AFTER:
showStep("step-validation");
```

Inside `btnSkipHotkey` click handler:
```js
// BEFORE:
// Story 2.5 will implement: showStep("step-validation");
// AFTER:
showStep("step-validation");
```

**Change 2 — Append validation section** at the end of `onboarding.js`:

```js
// ── Validation Step ───────────────────────────────────────────────────────────

const validationStatusEl = document.getElementById("validation-status");
const btnFinish = document.getElementById("btn-finish");
const btnFinishAnyway = document.getElementById("btn-finish-anyway");

async function loadValidationStep() {
  if (validationStatusEl) validationStatusEl.innerHTML = "";
  if (btnFinish) btnFinish.style.display = "none";
  if (btnFinishAnyway) btnFinishAnyway.style.display = "none";

  let status;
  try {
    status = await invoke("check_all_permissions");
  } catch (e) {
    console.warn("[ICantSpell] onboarding: error checking permissions:", e);
    if (validationStatusEl) {
      validationStatusEl.innerHTML =
        '<p class="status-warning">Could not check permissions. You can still finish setup.</p>';
    }
    if (btnFinishAnyway) btnFinishAnyway.style.display = "";
    return;
  }

  const items = [
    { label: "Accessibility", granted: status.accessibility },
    { label: "Microphone", granted: status.microphone },
  ];

  if (validationStatusEl) {
    validationStatusEl.innerHTML = items
      .map(
        (item) =>
          `<div class="permission-item">
             <span class="${item.granted ? "granted" : "missing"}">
               ${item.granted ? "✓" : "✗"}
             </span>
             <span>${item.label}: ${item.granted ? "Granted" : "Not granted"}</span>
           </div>`
      )
      .join("");
  }

  const allGranted = status.accessibility && status.microphone;
  if (allGranted) {
    if (btnFinish) btnFinish.style.display = "";
  } else {
    if (btnFinishAnyway) btnFinishAnyway.style.display = "";
  }
}

// Watch for the validation step becoming active and load it automatically.
const validationStepEl = document.getElementById("step-validation");
if (validationStepEl) {
  new MutationObserver(() => {
    if (validationStepEl.classList.contains("active")) {
      loadValidationStep();
    }
  }).observe(validationStepEl, { attributes: true, attributeFilter: ["class"] });
}

if (btnFinish) {
  btnFinish.addEventListener("click", async () => {
    try {
      await invoke("finish_onboarding", { allGranted: true });
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error finishing onboarding:", e);
    }
  });
}

if (btnFinishAnyway) {
  btnFinishAnyway.addEventListener("click", async () => {
    try {
      await invoke("finish_onboarding", { allGranted: false });
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error finishing onboarding:", e);
    }
  });
}
```

**Why MutationObserver?** The `showStep()` function uses `classList.add("active")` — this is a DOM mutation that MutationObserver catches reliably. It fires synchronously on the same tick, so `loadValidationStep()` runs immediately after the step becomes visible. This avoids adding special-case logic to `showStep()` itself and mirrors how a real app would observe UI state.

---

### Task 5: `styles.css` Addition

Add after `#onboarding-app #btn-skip-hotkey` rule, before `@media (prefers-color-scheme: dark)`:

```css
#onboarding-app .permission-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0.4rem 0;
  font-size: 0.95em;
}

#onboarding-app .permission-item .granted {
  color: #2a7a2a;
  font-weight: 700;
  width: 1.2em;
  text-align: center;
}

#onboarding-app .permission-item .missing {
  color: #c84b00;
  font-weight: 700;
  width: 1.2em;
  text-align: center;
}

#onboarding-app #btn-finish {
  margin-top: 1rem;
}

#onboarding-app #btn-finish-anyway {
  margin-top: 0.5rem;
  margin-left: 0.5rem;
}
```

Add inside the existing `@media (prefers-color-scheme: dark)` block (after the hotkey dark-mode rules):
```css
  #onboarding-app .permission-item .granted {
    color: #5fba5f;
  }

  #onboarding-app .permission-item .missing {
    color: #ff8c42;
  }
```

---

### Architecture Compliance

- `permissions.rs` remains the ONLY module that calls `AXIsProcessTrusted()` / `AVCaptureDevice` — `check_all_permissions` simply calls the existing helpers.
- `PermissionsStatus` derives `serde::Serialize` — Tauri serializes it to `{ "accessibility": bool, "microphone": bool }` (snake_case fields become camelCase in JS per Tauri serde behavior → `status.accessibility`, `status.microphone`).

  **Wait — Tauri's default serde behavior:** Rust `snake_case` fields serialize to `camelCase` in JS ONLY if the struct uses `#[serde(rename_all = "camelCase")]`. Without it, the JSON keys match the Rust field names exactly: `accessibility` and `microphone`. Both are single words, so there's no case difference. The JS accesses them as `status.accessibility` and `status.microphone`. ✅ No rename needed.

- `finish_onboarding` is `async fn` returning `Result<(), String>` — matches Tauri command pattern.
- `osascript` notification: fire-and-forget via `std::process::Command::spawn()` — same pattern as `open_accessibility_settings()` in `permissions.rs`. Failure logged via `tracing::warn!`, not surfaced as an error (notification is best-effort).
- IPC command names `check_all_permissions` and `finish_onboarding` use `snake_case` — matches naming rules.
- Frontend accesses via `invoke()` from `@tauri-apps/api/core` — correct Tauri v2 pattern.
- `Manager` trait already imported in `lib.rs` — no additional imports needed for `get_webview_window`.

### What NOT to Do

- Do NOT add `tauri-plugin-notification` or any new crate — `osascript` via `std::process::Command` is sufficient and requires no new dependency.
- Do NOT create a second `invoke_handler` in `lib.rs`.
- Do NOT implement permission revocation monitoring — that is Story 2.6.
- Do NOT implement re-granting flow from this step — direct to System Settings only (Story 2.6 handles re-grant detection).
- Do NOT add `showStep` logic inside the existing `showStep()` function — use MutationObserver in the validation section instead.
- Do NOT close the onboarding window from the frontend — always go through the `finish_onboarding` Tauri command.
- Do NOT add any new `#[link]` directives or FFI — this story uses no new macOS APIs.
- Do NOT modify `tauri.conf.json`, `capabilities/default.json`, or `Cargo.toml`.
- Do NOT implement the menu bar popover itself — Story 5.1 handles that. "Menu bar popover is ready" in AC 2 means the menu bar icon is already visible (it appears immediately at startup per Story 1.1).

### Files to Touch

| File | Action | Why |
|------|---------|-----|
| `src-tauri/src/permissions.rs` | MODIFY | Add `PermissionsStatus` struct, `check_all_permissions` command, one new test |
| `src-tauri/src/lib.rs` | MODIFY | Add `finish_onboarding` command, register both new commands in `invoke_handler` |
| `src/onboarding.html` | MODIFY | Append `#step-validation` div |
| `src/onboarding.js` | MODIFY | Replace two Story 2.5 stubs; append validation step section |
| `src/styles.css` | MODIFY | Add `.permission-item`, `#btn-finish`, `#btn-finish-anyway` rules + dark mode variants |

**Files NOT touched:**
- `src-tauri/Cargo.toml` — no new crates
- `src-tauri/tauri.conf.json` — correct from Story 1.4
- `src-tauri/capabilities/default.json` — `core:default` already covers onboarding window
- `src-tauri/src/config.rs` — no changes
- All stub modules (`audio.rs`, `hotkey.rs`, `injection.rs`, `overlay.rs`, `models.rs`)

### Test Count

| Scope | Count |
|-------|-------|
| Pre-existing (config + stt + is_first_launch + accessibility + microphone + ptt_hotkey) | 12 |
| New: `test_permissions_status_fields` in `permissions.rs` | 1 |
| **Total** | **13** |

### Previous Story Intelligence (Story 2.4)

- `onboarding.js` after Story 2.4: two `// Story 2.5 will implement: showStep("step-validation");` stubs at lines ~308 and ~320. One is in `btnConfirmHotkey` click handler (after `save_ptt_hotkey` invoke), one in `btnSkipHotkey` click handler. Verify both locations by grepping before editing.
- `lib.rs` after Story 2.4: `invoke_handler` has 5 commands. This story adds 2 → 7 total.
- `styles.css` after Story 2.4: last rules before `@media` block are the hotkey field/button rules (`#btn-skip-hotkey`). Insert validation CSS after them.
- Test baseline is **12 tests** after Story 2.4. This story adds 1 → **13 total**. All 12 existing tests must pass.
- Story 2.4 `finish_onboarding` placement rationale: `lib.rs` chosen (not `permissions.rs`) because it needs `Manager` (already imported) and is app-level orchestration — same reasoning as `is_first_launch` living in `lib.rs`.

### References

- [Source: epics.md § Story 2.5] — User story, all 4 acceptance criteria
- [Source: architecture.md § API Patterns] — Tauri commands return `Result<T, String>`; async fn; snake_case names
- [Source: architecture.md § System API Boundary] — `permissions.rs` is sole caller of AXIsProcessTrusted / AVCaptureDevice
- [Source: architecture.md § Frontend Architecture] — `invoke()` from `@tauri-apps/api/core`; no direct OS API from JS
- [Source: architecture.md § Logging Patterns] — `tracing::warn!` for recoverable failures; `tracing::info!` for milestones
- [Source: story 2-3 Dev Notes § Task 1] — `std::process::Command` pattern for spawning `open` URL — same pattern used here for `osascript`
- [Source: story 2-4 Completion Notes] — 12 tests baseline; two Story 2.5 stubs in `onboarding.js`; `lib.rs` invoke_handler has 5 commands; `Manager` already imported
- [Source: story 2-1 Dev Notes] — `config::ensure_defaults()` writes `settings.json` at startup; AC 4 is already satisfied

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

No issues — all tasks implemented cleanly without debug iterations.

### Completion Notes List

- Added `PermissionsStatus { accessibility: bool, microphone: bool }` struct with `serde::Serialize` to `permissions.rs`; added `check_all_permissions` Tauri command that calls the existing `check_accessibility()` and `check_microphone()` helpers.
- Added `finish_onboarding(app, all_granted)` Tauri command to `lib.rs` (before `run()`): closes `onboarding` window via `app.get_webview_window("onboarding")?.close()`; if `all_granted`, spawns `osascript` for macOS system notification via `std::process::Command` (same fire-and-forget pattern as `open_accessibility_settings()`). Logs completion at `info` level.
- Registered `permissions::check_all_permissions` and `finish_onboarding` in `invoke_handler` — 7 commands total.
- Appended `#step-validation` to `onboarding.html`: heading, permission summary div, hidden `#btn-finish` and `#btn-finish-anyway` buttons.
- Replaced both `showStep("step-validation")` stubs in `onboarding.js` (hotkey confirm and skip handlers).
- Appended validation section to `onboarding.js`: `loadValidationStep()` invokes `check_all_permissions`, renders `.permission-item` divs with granted/missing spans, shows `#btn-finish` (all granted) or `#btn-finish-anyway` (any missing); MutationObserver on `#step-validation` triggers `loadValidationStep()` when the step becomes active; error path shows fallback warning and `#btn-finish-anyway`.
- Added `.permission-item`, `.granted`, `.missing`, `#btn-finish`, `#btn-finish-anyway` CSS rules; dark mode variants for granted/missing colors.
- 13/13 tests pass; `cargo clippy --all-targets -- -D warnings` — zero warnings.

### File List

- `src-tauri/src/permissions.rs` — added `PermissionsStatus` struct, `check_all_permissions` command, `test_permissions_status_fields` test
- `src-tauri/src/lib.rs` — added `finish_onboarding` command, registered `permissions::check_all_permissions` and `finish_onboarding` in `invoke_handler`
- `src/onboarding.html` — appended `#step-validation` step div
- `src/onboarding.js` — replaced two Story 2.5 stubs; appended validation step section with MutationObserver, `loadValidationStep`, finish button handlers
- `src/styles.css` — added `.permission-item`, `.granted`, `.missing`, `#btn-finish`, `#btn-finish-anyway` rules; dark mode variants

### Review Findings

- [x] [Review][Patch] Double-click on finish buttons sends duplicate IPC calls — disable button on first click to prevent duplicate `finish_onboarding` invocations and duplicate macOS notifications [src/onboarding.js: btnFinish/btnFinishAnyway click handlers]
- [x] [Review][Defer] Permission monitor thread (`start_permission_monitor`) has no shutdown/cancellation mechanism — thread runs forever with no `JoinHandle` stored [src-tauri/src/permissions.rs] — deferred, Story 2.6 scope
- [x] [Review][Defer] `PermissionChangedPayload` and `test_permission_changed_payload_fields` are Story 2.6 scope — present in diff but not part of Story 2.5 spec [src-tauri/src/permissions.rs] — deferred, Story 2.6 scope
- [x] [Review][Defer] Event names `permission_revoked`/`permission_restored` use snake_case — verify JS listener convention matches when Story 2.6 frontend is implemented [src-tauri/src/permissions.rs] — deferred, Story 2.6 scope

## Change Log

- 2026-04-30: Story 2.5 created — Permission Validation & Onboarding Completion ready for dev.
- 2026-04-30: Code review complete — 1 patch applied (double-click guard on finish buttons), 3 deferred (Story 2.6 scope), 12 dismissed.
- 2026-04-30: Implemented Story 2.5 — `PermissionsStatus` + `check_all_permissions` in `permissions.rs`; `finish_onboarding` in `lib.rs` (closes window + osascript notification when all_granted); `#step-validation` onboarding step with MutationObserver-triggered permission summary and conditional Finish/Finish Anyway buttons. 13/13 tests pass, zero clippy warnings.
