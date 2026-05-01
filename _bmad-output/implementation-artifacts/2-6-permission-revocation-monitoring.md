# Story 2.6: Permission Revocation Monitoring

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user who has completed onboarding,
I want the app to notify me if I revoke a required permission after initial setup,
so that I understand why voice mode stopped working without the app crashing or freezing.

## Acceptance Criteria

1. **Given** the app is running with all permissions granted, **When** the user revokes Accessibility or Microphone permission in System Settings, **Then** `permissions.rs` detects the revocation within ≤5 seconds (polling interval).

2. **Given** a permission is revoked, **When** detection occurs, **Then** the app emits a `permission_revoked` IPC event with the permission name (`"Accessibility"` or `"Microphone"`).

3. **Given** the `permission_revoked` event is received by the frontend, **When** it renders, **Then** a non-intrusive indicator appears in the menu bar popover (`index.html`) explaining which permission was lost.

4. **Given** Accessibility permission is revoked, **When** the user attempts PTT dictation, **Then** voice mode does not crash — it surfaces a graceful error. (Voice mode is implemented in Epic 3; this AC is forward-looking. This story satisfies it by ensuring the monitoring + notification infrastructure is in place so Epic 3 can check `check_accessibility()` before activating.)

5. **And** re-granting the permission in System Settings and returning to the app clears the warning without requiring a restart (implemented via a `permission_restored` backend event → frontend hides the warning).

## Tasks / Subtasks

- [x] Task 1: Add `PermissionChangedPayload` and `start_permission_monitor` to `permissions.rs` (AC: 1, 2)
  - [x] Add `#[derive(Debug, Clone, serde::Serialize)] pub struct PermissionChangedPayload { pub permission_name: String }` BEFORE the existing `PermissionsStatus` struct (both are payload structs; group them)
  - [x] Add `pub fn start_permission_monitor(app: tauri::AppHandle)` AFTER `check_all_permissions` and BEFORE `#[cfg(test)]`; the function calls `std::thread::spawn` with a loop that: (a) sleeps 5 s via `std::time::Duration::from_secs(5)`, (b) calls `check_accessibility()` and `check_microphone()`, (c) compares to `last_accessibility` / `last_microphone`, (d) on granted→revoked emits `"permission_revoked"` event, (e) on revoked→granted emits `"permission_restored"` event; use `use tauri::Emitter;` inside the spawned closure where `app.emit()` is called
  - [x] Add test `test_permission_changed_payload_fields` inside the existing `#[cfg(test)]` block: construct `PermissionChangedPayload { permission_name: "Accessibility".to_string() }`, serialize via `serde_json::to_value`, assert `json["permission_name"] == "Accessibility"`

- [x] Task 2: Call `start_permission_monitor` in `lib.rs` setup (AC: 1, 2)
  - [x] In the `setup` closure after `config::ensure_defaults(app.handle())?;`, add `permissions::start_permission_monitor(app.handle().clone());`
  - [x] No new `use` imports needed — `permissions::` is already accessible via `pub mod permissions;`

- [x] Task 3: Add permission warning div to `index.html` (AC: 3, 5)
  - [x] Replace the `<!-- Main window UI — implemented in Story 1.4 -->` comment with a real `<div id="permission-warning" style="display:none"></div>` inside `<body>` (Story 5.1 will build the full popover; this story only adds the warning div)

- [x] Task 4: Wire event listeners in `main.js` (AC: 3, 5)
  - [x] Add `import { listen } from "@tauri-apps/api/event";` at the top
  - [x] Declare `const revokedPermissions = new Set();` to track which permissions are currently revoked
  - [x] Implement `updatePermissionWarning()`: reads `revokedPermissions`, builds a warning message listing revoked items, sets `#permission-warning` `textContent` and toggles `display` (visible if set is non-empty, hidden otherwise)
  - [x] Call `listen("permission_revoked", (event) => { revokedPermissions.add(event.payload.permissionName); updatePermissionWarning(); })` (note: Tauri serializes `snake_case` Rust fields to `camelCase` in JS — `permission_name` → `permissionName`)
  - [x] Call `listen("permission_restored", (event) => { revokedPermissions.delete(event.payload.permissionName); updatePermissionWarning(); })`

- [x] Task 5: Add `#permission-warning` CSS to `styles.css` (AC: 3)
  - [x] Add `#permission-warning` block: `padding: 0.75rem 1rem`, `background: #fff3cd`, `border: 1px solid #ffc107`, `border-radius: 6px`, `color: #664d03`, `font-size: 0.875em`, `margin: 0.5rem`; BEFORE the `@media (prefers-color-scheme: dark)` block
  - [x] Add dark-mode variant inside existing `@media` block: `#permission-warning { background: #3d2e00; border-color: #ffc107; color: #ffd966; }`

- [x] Task 6: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
  - [x] `cargo test` — 14 tests pass (13 pre-existing + 1 new `test_permission_changed_payload_fields`)
  - [x] Verify `start_permission_monitor` compiles without `tokio` in `Cargo.toml` (uses `std::thread::spawn` + `std::time::Duration`, no external async dependency)

## Dev Notes

### No New Crates Required

All implementation uses existing infrastructure:
- `std::thread::spawn` + `std::thread::sleep` — standard library, no new dependency
- `tauri::Emitter` trait — already in the `tauri` crate dependency
- No `tokio` feature needed; `std::thread::spawn` is the correct choice here because this is a long-running background loop, not a short async task

### Architecture: IPC Events `permission_revoked` and `permission_restored`

The architecture spec lists `BE → FE | permission_revoked | { permission_name }` as a planned IPC event (architecture.md § API Patterns). `permission_restored` is additive (not in the original table) but follows the same pattern and is required to satisfy AC 5 (clear warning on re-grant). Both use `snake_case` event names as required.

**Tauri camelCase deserialization:** Rust struct field `permission_name: String` serializes to JSON key `"permission_name"`. Tauri's default serde behavior for events does NOT rename fields — they stay `snake_case` in the JSON. However, Tauri's JS IPC layer converts them to camelCase on the JS side. So in `main.js` the field is `event.payload.permissionName` (camelCase). Verify this at test time if the name doesn't match.

**Actually — double-check this:** Tauri commands use a deserializer that renames to camelCase for command args, but for `app.emit()` payloads the JSON is passed as-is from `serde_json::to_value`. The field name in JS will be `"permission_name"` (snake_case), NOT `"permissionName"`. Use `event.payload.permission_name` in JS.

To eliminate ambiguity and be safe: add `#[serde(rename_all = "camelCase")]` to `PermissionChangedPayload` so the JSON field is `permissionName` and the JS access is `event.payload.permissionName`. This is consistent with the architecture note that "Tauri automatically serializes Rust snake_case struct fields to camelCase JSON for the frontend."

### Current State of Files Being Modified

**`src-tauri/src/permissions.rs` — current state (after Story 2.5, 13 tests):**
```
// Exports: check_accessibility(), open_accessibility_settings(),
//          check_accessibility_permission (cmd), request_accessibility_permission (cmd),
//          check_microphone(), request_microphone_access(),
//          check_microphone_permission (cmd), request_microphone_permission (cmd),
//          PermissionsStatus struct, check_all_permissions (cmd)
// Tests: test_check_accessibility_returns_bool, test_check_microphone_returns_bool,
//        test_permissions_status_fields
// Total: 3 tests
```
Add `PermissionChangedPayload` struct BEFORE `PermissionsStatus` (group related payload types), and `start_permission_monitor` function AFTER `check_all_permissions` and BEFORE `#[cfg(test)]`.

**`src-tauri/src/lib.rs` — current setup closure (after Story 2.5):**
```rust
config::ensure_defaults(app.handle())?;

if first_launch { ... show onboarding ... }

Ok(())
```
Insert `permissions::start_permission_monitor(app.handle().clone());` immediately after `config::ensure_defaults(app.handle())?;` and before the `if first_launch` block. This ensures monitoring starts on every launch, not just first launch.

**`src/index.html` — current body (after Story 1.4):**
```html
<body>
  <!-- Main window UI — implemented in Story 1.4 -->
</body>
```
Replace the comment with `<div id="permission-warning" style="display:none"></div>`. Keep the `<script type="module" src="/main.js" defer></script>` in the `<head>` (it's already there from Story 1.4).

**`src/main.js` — current state (after Story 1.4):**
```js
// Menu bar popover logic — IPC listeners and controls added in Story 5.1
console.debug("[ICantSpell] main window loaded");
```
Append the event listener code after the existing console.debug line. The comment on line 1 can stay — it describes Story 5.1 work that's still upcoming.

**`src/styles.css` — current onboarding rules (after Story 2.5):**
Last rules before `@media (prefers-color-scheme: dark)` are:
```css
#onboarding-app #btn-finish-anyway { margin-top: 0.5rem; margin-left: 0.5rem; }
```
Insert `#permission-warning` CSS after `#btn-finish-anyway` rule, before `@media`.

---

### Task 1: Exact `permissions.rs` Additions

Add BEFORE the existing `PermissionsStatus` struct (after the `request_microphone_permission` command):

```rust
// ── Permission Change Monitoring ──────────────────────────────────────────────

/// Payload for `permission_revoked` and `permission_restored` IPC events.
/// `permission_name` is either `"Accessibility"` or `"Microphone"`.
/// The `camelCase` rename ensures JS receives `permissionName` (camelCase).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionChangedPayload {
    pub permission_name: String,
}

/// Starts a background thread that polls Accessibility and Microphone permission
/// status every 5 seconds. On state transitions (granted→revoked or revoked→granted),
/// emits `permission_revoked` or `permission_restored` Tauri events to all windows.
///
/// Uses `std::thread::spawn` rather than `tokio::spawn` to avoid requiring
/// explicit `tokio` features — this is a long-running background loop, not
/// a short async task.
pub fn start_permission_monitor(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        use tauri::Emitter;

        let mut last_accessibility = check_accessibility();
        let mut last_microphone = check_microphone();

        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));

            let accessibility = check_accessibility();
            let microphone = check_microphone();

            if last_accessibility && !accessibility {
                let _ = app.emit(
                    "permission_revoked",
                    PermissionChangedPayload {
                        permission_name: "Accessibility".to_string(),
                    },
                );
                tracing::warn!("Accessibility permission revoked — notified frontend");
            } else if !last_accessibility && accessibility {
                let _ = app.emit(
                    "permission_restored",
                    PermissionChangedPayload {
                        permission_name: "Accessibility".to_string(),
                    },
                );
                tracing::info!("Accessibility permission restored — notified frontend");
            }

            if last_microphone && !microphone {
                let _ = app.emit(
                    "permission_revoked",
                    PermissionChangedPayload {
                        permission_name: "Microphone".to_string(),
                    },
                );
                tracing::warn!("Microphone permission revoked — notified frontend");
            } else if !last_microphone && microphone {
                let _ = app.emit(
                    "permission_restored",
                    PermissionChangedPayload {
                        permission_name: "Microphone".to_string(),
                    },
                );
                tracing::info!("Microphone permission restored — notified frontend");
            }

            last_accessibility = accessibility;
            last_microphone = microphone;
        }
    });
}
```

Add inside the existing `#[cfg(test)]` block:
```rust
    #[test]
    fn test_permission_changed_payload_fields() {
        // Verifies the struct serializes with camelCase field name ("permissionName").
        let payload = PermissionChangedPayload {
            permission_name: "Accessibility".to_string(),
        };
        let json = serde_json::to_value(&payload).expect("serialize ok");
        assert_eq!(json["permissionName"], "Accessibility");
    }
```

---

### Task 2: `lib.rs` Change

In the `setup` closure, add one line after `config::ensure_defaults(app.handle())?;`:

```rust
config::ensure_defaults(app.handle())?;
permissions::start_permission_monitor(app.handle().clone());  // ← add this line
```

No other changes to `lib.rs`.

---

### Task 3: `index.html` Change

Replace:
```html
  <body>
    <!-- Main window UI — implemented in Story 1.4 -->
  </body>
```
With:
```html
  <body>
    <div id="permission-warning" style="display:none"></div>
  </body>
```

---

### Task 4: `main.js` Changes

Replace the entire file content with:
```js
// Menu bar popover logic — IPC listeners and controls added in Story 5.1
import { listen } from "@tauri-apps/api/event";

console.debug("[ICantSpell] main window loaded");

// ── Permission Revocation Monitoring ─────────────────────────────────────────

const revokedPermissions = new Set();

function updatePermissionWarning() {
  const el = document.getElementById("permission-warning");
  if (!el) return;
  if (revokedPermissions.size === 0) {
    el.style.display = "none";
    el.textContent = "";
    return;
  }
  const names = [...revokedPermissions].join(" and ");
  el.textContent = `⚠ ${names} permission revoked. Re-grant in System Settings to restore voice mode.`;
  el.style.display = "";
}

listen("permission_revoked", (event) => {
  revokedPermissions.add(event.payload.permissionName);
  updatePermissionWarning();
}).catch((e) => console.warn("[ICantSpell] Failed to listen for permission_revoked:", e));

listen("permission_restored", (event) => {
  revokedPermissions.delete(event.payload.permissionName);
  updatePermissionWarning();
}).catch((e) => console.warn("[ICantSpell] Failed to listen for permission_restored:", e));
```

---

### Task 5: `styles.css` Addition

Add after `#onboarding-app #btn-finish-anyway` rule, before `@media (prefers-color-scheme: dark)`:

```css
#permission-warning {
  padding: 0.75rem 1rem;
  background: #fff3cd;
  border: 1px solid #ffc107;
  border-radius: 6px;
  color: #664d03;
  font-size: 0.875em;
  margin: 0.5rem;
}
```

Add inside the existing `@media (prefers-color-scheme: dark)` block (after the `.missing` dark rule):
```css
  #permission-warning {
    background: #3d2e00;
    border-color: #ffc107;
    color: #ffd966;
  }
```

---

### Architecture Compliance

- `permissions.rs` remains the ONLY module that queries system permission APIs — `start_permission_monitor` calls the existing `check_accessibility()` and `check_microphone()` helpers.
- IPC event names `permission_revoked` and `permission_restored` use `snake_case` — matches architecture naming rule.
- `PermissionChangedPayload` uses `#[serde(rename_all = "camelCase")]` so the JS receives `permissionName` — consistent with architecture note about Tauri's camelCase serialization.
- `std::thread::spawn` for the background loop — no new crate needed, no `tokio` features required beyond what Tauri already provides.
- `tracing::warn!` and `tracing::info!` for state changes — follows logging patterns.
- `tauri::Emitter` trait brought into scope inside the thread closure where `app.emit()` is called.

### What NOT to Do

- Do NOT add `tokio` to `Cargo.toml` — `std::thread::spawn` + `std::thread::sleep` is sufficient for a polling loop.
- Do NOT implement PTT dictation or voice mode here — that is Epic 3. AC 4 is satisfied by the monitoring infrastructure being in place; Epic 3 stories will check `check_accessibility()` before activating.
- Do NOT open System Settings from the monitoring loop — the loop only detects and emits; the frontend decides how to respond.
- Do NOT modify `tauri.conf.json`, `capabilities/default.json`, or `Cargo.toml`.
- Do NOT add new Tauri commands for the monitoring — it's started via `start_permission_monitor()` in setup, not via a frontend invoke call.
- Do NOT make `start_permission_monitor` async — `std::thread::spawn` is intentional; this is a forever-running background loop.

### Files to Touch

| File | Action | Why |
|------|---------|-----|
| `src-tauri/src/permissions.rs` | MODIFY | Add `PermissionChangedPayload` struct, `start_permission_monitor` function, one new test |
| `src-tauri/src/lib.rs` | MODIFY | Call `permissions::start_permission_monitor(app.handle().clone())` in setup |
| `src/index.html` | MODIFY | Replace body comment with `#permission-warning` div |
| `src/main.js` | MODIFY | Add `listen` import + revocation/restoration event handlers |
| `src/styles.css` | MODIFY | Add `#permission-warning` CSS + dark mode variant |

**Files NOT touched:**
- `src-tauri/Cargo.toml` — no new crates
- `src-tauri/tauri.conf.json` — correct from Story 1.4
- `src-tauri/capabilities/default.json` — `core:default` already covers main window
- `src-tauri/src/config.rs` — no changes
- `src/onboarding.html` / `src/onboarding.js` — no changes

### Test Count

| Scope | Count |
|-------|-------|
| Pre-existing (config + stt + is_first_launch + permissions, all stories 2.1–2.5) | 13 |
| New: `test_permission_changed_payload_fields` in `permissions.rs` | 1 |
| **Total** | **14** |

### Previous Story Intelligence (Story 2.5)

- `permissions.rs` after Story 2.5: 3 tests, exports `PermissionsStatus` + `check_all_permissions`. Add new content after `check_all_permissions` and before `#[cfg(test)]`.
- `lib.rs` after Story 2.5: setup closure calls `config::ensure_defaults()` then optionally shows onboarding. `permissions::` module is accessible via existing `pub mod permissions;`.
- Test baseline is **13 tests** after Story 2.5. This story adds 1 → **14 total**.
- `index.html` body has only a comment — safe to replace with the warning div.
- `main.js` has only 2 lines — safe to replace/expand entirely.

### References

- [Source: epics.md § Story 2.6] — User story, all 5 acceptance criteria
- [Source: architecture.md § API Patterns] — `permission_revoked` IPC event with `{ permission_name }` payload
- [Source: architecture.md § System API Boundary] — `permissions.rs` sole caller of AXIsProcessTrusted / AVCaptureDevice
- [Source: architecture.md § Naming Patterns] — snake_case IPC events; camelCase in JS after Tauri deserialization
- [Source: architecture.md § Logging Patterns] — `tracing::warn!` for revocations; `tracing::info!` for restorations
- [Source: architecture.md § Format Patterns] — IPC payloads use typed structs with `serde::Serialize`
- [Source: story 2-5 Dev Notes] — 13 tests baseline; `permissions.rs` structure after Story 2.5

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

No issues — all tasks implemented cleanly without debug iterations.

### Completion Notes List

- Added `PermissionChangedPayload { permission_name: String }` struct with `#[serde(rename_all = "camelCase")]` to `permissions.rs`; serializes to `{ "permissionName": "..." }` for JS consumers.
- Added `start_permission_monitor(app: tauri::AppHandle)` to `permissions.rs`: spawns a `std::thread::spawn` background loop (no new crate required — uses `std::thread::sleep`); polls every 5 seconds; emits `permission_revoked` or `permission_restored` Tauri event on state transitions (granted↔revoked) for both Accessibility and Microphone; uses `tauri::Emitter` trait in scope for `app.emit()`.
- Called `permissions::start_permission_monitor(app.handle().clone())` in `lib.rs` setup closure after `config::ensure_defaults()` — runs on every launch, not just first launch.
- Added `<div id="permission-warning" style="display:none">` to `index.html` body (replaces placeholder comment).
- Updated `main.js`: added `listen` import from `@tauri-apps/api/event`; `revokedPermissions` Set tracks revoked names; `updatePermissionWarning()` shows/hides `#permission-warning` div with message listing revoked permissions; listeners for `permission_revoked` (adds to set) and `permission_restored` (removes from set) events.
- Added `#permission-warning` CSS (amber/yellow warning box) and dark-mode variant to `styles.css`.
- 14/14 tests pass; `cargo clippy --all-targets -- -D warnings` — zero warnings.

### File List

- `src-tauri/src/permissions.rs` — added `PermissionChangedPayload` struct, `start_permission_monitor` function, `test_permission_changed_payload_fields` test
- `src-tauri/src/lib.rs` — added `permissions::start_permission_monitor(app.handle().clone())` call in setup closure
- `src/index.html` — replaced body comment with `<div id="permission-warning" style="display:none">`
- `src/main.js` — added `listen` import; `revokedPermissions` Set; `updatePermissionWarning()`; `permission_revoked` and `permission_restored` event listeners
- `src/styles.css` — added `#permission-warning` CSS block + dark-mode variant

### Review Findings

- [x] [Review][Patch] No initial state sync — if permission is already revoked at launch, monitor's first poll sees `last == current` (both false) and never emits `permission_revoked`; warning never appears until a grant→revoke transition [src-tauri/src/permissions.rs: start_permission_monitor]
- [x] [Review][Patch] `app.emit()` errors silently swallowed with `let _ =` — should log on failure for observability [src-tauri/src/permissions.rs: start_permission_monitor]
- [x] [Review][Patch] `#permission-warning` div missing `role="alert"` for screen reader accessibility [src/index.html]
- [x] [Review][Defer] Monitor thread has no cancellation/shutdown mechanism — `std::process::exit()` terminates all threads so not a bug today, but add graceful shutdown when teardown story arrives [src-tauri/src/permissions.rs] — deferred, future teardown story

## Change Log

- 2026-04-30: Code review complete — 3 patches applied (initial state sync, emit error logging, ARIA role), 1 deferred (thread shutdown). 14/14 tests pass.
- 2026-04-30: Story 2.6 created — Permission Revocation Monitoring ready for dev.
- 2026-04-30: Implemented Story 2.6 — `PermissionChangedPayload` + `start_permission_monitor` in `permissions.rs` (std::thread background poll loop, 5s interval, emits permission_revoked/permission_restored IPC events); monitor started in `lib.rs` setup; `#permission-warning` div + JS event listeners in menu bar popover; amber warning CSS + dark mode. 14/14 tests pass, zero clippy warnings.
