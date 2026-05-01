# Story 2.3: Microphone Permission Request

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a new user,
I want the onboarding flow to explain and request Microphone permission,
so that audio capture works when I first try to dictate.

## Acceptance Criteria

1. **Given** the onboarding wizard is on the Microphone permission step, **When** it renders, **Then** a clear rationale is shown ("needed to capture your voice during PTT hold; no background listening").

2. **Given** the user clicks the grant button, **When** the app requests microphone access via `AVCaptureDevice.requestAccess`, **Then** the macOS system permission dialog appears (status `notDetermined`) OR System Settings > Privacy & Security > Microphone opens (status `denied`).

3. **Given** the user grants Microphone permission, **When** the wizard polls permission status, **Then** the step is marked complete and the wizard advances to the next step (Story 2.4 placeholder comment).

4. **Given** the user denies Microphone permission, **When** the wizard checks permission status, **Then** it shows a non-blocking warning that voice mode requires microphone access, with a link to System Settings.

## Tasks / Subtasks

- [x] Task 1: Extend `permissions.rs` with microphone check and request functions (AC: 2, 3, 4)
  - [x] Add `#[link(name = "AVFoundation", kind = "framework")]` extern block with `AVMediaTypeAudio` static
  - [x] Add `extern "C"` declarations for `objc_getClass`, `sel_registerName`, and typed `objc_msgSend` variants
  - [x] Add `_NSConcreteGlobalBlock` extern and `BlockLayout` / `BlockDescriptor` structs for completion handler
  - [x] Implement `pub fn check_microphone() -> bool` (queries `AVAuthorizationStatus` via `objc_msgSend`, returns `true` only for `Authorized`)
  - [x] Implement `pub fn request_microphone_access()` (calls `requestAccessForMediaType:completionHandler:` for `NotDetermined`, opens System Settings URL for `Denied`/`Restricted`)
  - [x] Add `#[tauri::command] pub async fn check_microphone_permission() -> Result<bool, String>`
  - [x] Add `#[tauri::command] pub async fn request_microphone_permission() -> Result<(), String>`
  - [x] Add `#[cfg(test)]` block with `test_check_microphone_returns_bool` (no-panic test)

- [x] Task 2: Register microphone commands in `lib.rs` (AC: 2)
  - [x] Add `permissions::check_microphone_permission` and `permissions::request_microphone_permission` to the existing `invoke_handler` list

- [x] Task 3: Add Microphone step UI to `onboarding.html` (AC: 1)
  - [x] Append `<div class="step" id="step-microphone">` after `#step-accessibility`, containing: heading, two explanation paragraphs, `#microphone-status` div, `#btn-grant-microphone` button, `#btn-skip-microphone` button

- [x] Task 4: Wire microphone step in `onboarding.js` (AC: 2–4)
  - [x] Replace both `// Story 2.3 will implement: showStep("step-microphone");` stubs with `showStep("step-microphone");`
  - [x] Add `// ── Microphone Step` section (mirroring Accessibility step pattern) with `stopMicrophonePolling`, `setMicrophoneStatus`, `startMicrophonePolling` functions
  - [x] Wire `#btn-grant-microphone`: `invoke("request_microphone_permission")`, then `startMicrophonePolling()`
  - [x] Wire `#btn-skip-microphone`: stop polling, set warning status, stub `// Story 2.4 will implement: showStep("step-hotkey")`

- [x] Task 5: Add Microphone button CSS to `styles.css` (AC: 1)
  - [x] Add `#onboarding-app #btn-skip-microphone` margin rule (matching `#btn-skip-accessibility`)

- [x] Task 6: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
  - [x] `cargo test` — 11 tests pass (10 pre-existing + 1 new `test_check_microphone_returns_bool`)
  - [x] Manual: accessibility step advances to microphone step → "Grant Microphone" shows system dialog (first launch) or opens System Settings (if denied) → granting shows confirmation → "Skip for Now" shows non-blocking warning

## Dev Notes

### No New Crates Required

`AVCaptureDevice` is an Objective-C class, but its permission APIs can be called via direct FFI using:
- `AVMediaTypeAudio` — exported by `AVFoundation.framework` as a C-linkage symbol (an `NSString *` constant)
- `authorizationStatusForMediaType:` — called via `objc_msgSend` with typed alias
- `requestAccessForMediaType:completionHandler:` — called via `objc_msgSend` with a manually-constructed global ObjC block struct (no captured variables → safe global block)

This mirrors exactly what Story 2.2 did for `AXIsProcessTrusted()` — direct framework FFI, no new `Cargo.toml` entries.

### Current State of Files Being Modified

**`src-tauri/src/permissions.rs` — current state (after Story 2.2):**
```rust
//! macOS permission checking and request flows.
//! This is the ONLY module that directly calls AXIsProcessTrusted().
//! Revocation monitoring is added in Story 2.6.

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn check_accessibility() -> bool { ... }
pub fn open_accessibility_settings() { ... }

#[tauri::command]
pub async fn check_accessibility_permission() -> Result<bool, String> { ... }

#[tauri::command]
pub async fn request_accessibility_permission() -> Result<(), String> { ... }

#[cfg(test)]
mod tests {
    #[test]
    fn test_check_accessibility_returns_bool() { ... }
}
```
**Add all microphone functions BELOW the existing accessibility code, ABOVE the `#[cfg(test)]` block.**

**`src-tauri/src/lib.rs` — current `invoke_handler` (after Story 2.2):**
```rust
.invoke_handler(tauri::generate_handler![
    permissions::check_accessibility_permission,
    permissions::request_accessibility_permission,
])
```
Add two microphone commands to this existing list — do NOT create a second `invoke_handler`.

**`src/onboarding.html` — current body (after Story 2.2):**
```html
<div id="onboarding-app">
  <div class="step active" id="step-welcome"> ... </div>
  <div class="step" id="step-accessibility"> ... </div>
</div>
```
Append `#step-microphone` inside `#onboarding-app`, after `#step-accessibility`.

**`src/onboarding.js` — current state (after Story 2.2):**
Two stub comments exist that Story 2.3 must replace:
```js
// Story 2.3 will implement: showStep("step-microphone");
```
One appears inside `startAccessibilityPolling` (on grant success), one inside `btnSkipAccessibility` click handler. Replace BOTH stubs with the real call. Then ADD the microphone section at the bottom of the file.

**`src/styles.css` — current onboarding rules (after Story 2.2):**
```css
#onboarding-app #btn-skip-accessibility {
  margin-top: 0.5rem;
  margin-left: 0.5rem;
}
```
Add the matching microphone skip button rule after this. The `@media (prefers-color-scheme: dark)` block comes after — insert before it.

---

### Task 1: Exact `permissions.rs` Additions

Add the following AFTER `open_accessibility_settings()` and BEFORE `// Tauri command: check if Accessibility...`:

```rust
// ── Microphone Permission ────────────────────────────────────────────────────

// AVFoundation exports AVMediaTypeAudio as a C-linkage NSString* constant.
#[cfg(target_os = "macos")]
#[link(name = "AVFoundation", kind = "framework")]
extern "C" {
    /// NSString constant "soun" — the media type for audio capture.
    static AVMediaTypeAudio: *mut std::ffi::c_void;
}

#[cfg(target_os = "macos")]
extern "C" {
    fn objc_getClass(name: *const u8) -> *mut std::ffi::c_void;
    fn sel_registerName(name: *const u8) -> *mut std::ffi::c_void;

    /// objc_msgSend variant: class method returning NSInteger (isize).
    /// Used for: [AVCaptureDevice authorizationStatusForMediaType:]
    #[link_name = "objc_msgSend"]
    fn msg_send_authorization_status(
        receiver: *mut std::ffi::c_void,
        sel: *mut std::ffi::c_void,
        media_type: *mut std::ffi::c_void,
    ) -> isize;

    /// objc_msgSend variant: class method taking object + block pointer, returning void.
    /// Used for: [AVCaptureDevice requestAccessForMediaType:completionHandler:]
    #[link_name = "objc_msgSend"]
    fn msg_send_request_access(
        receiver: *mut std::ffi::c_void,
        sel: *mut std::ffi::c_void,
        media_type: *mut std::ffi::c_void,
        block: *const BlockLayout,
    );

    /// Global block class — used as the `isa` pointer for blocks with no captured variables.
    static _NSConcreteGlobalBlock: *const std::ffi::c_void;
}

/// ObjC block layout for a completion handler block with no captured variables.
/// This is the ABI-defined struct layout for `^(BOOL granted) {}` blocks.
/// See: https://clang.llvm.org/docs/Block-ABI-Apple.html
#[cfg(target_os = "macos")]
#[repr(C)]
struct BlockLayout {
    isa: *const std::ffi::c_void,
    flags: i32,
    reserved: i32,
    invoke: unsafe extern "C" fn(*const BlockLayout, bool),
    descriptor: *const BlockDescriptor,
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct BlockDescriptor {
    reserved: usize,
    size: usize,
}

/// Safety: BlockLayout contains raw pointers, but we only ever use it as a
/// static (BLOCK_IS_GLOBAL = 1 << 28). No mutable state or aliasing.
#[cfg(target_os = "macos")]
unsafe impl Sync for BlockLayout {}

#[cfg(target_os = "macos")]
unsafe extern "C" fn microphone_block_invoke(_block: *const BlockLayout, _granted: bool) {
    // Intentionally empty — the frontend polls check_microphone_permission after calling this.
}

#[cfg(target_os = "macos")]
static MICROPHONE_BLOCK_DESCRIPTOR: BlockDescriptor = BlockDescriptor {
    reserved: 0,
    size: std::mem::size_of::<BlockLayout>(),
};

// AVAuthorizationStatus values (from AVFoundation headers):
// NotDetermined = 0, Restricted = 1, Denied = 2, Authorized = 3
const AV_AUTHORIZATION_STATUS_AUTHORIZED: isize = 3;
const AV_AUTHORIZATION_STATUS_NOT_DETERMINED: isize = 0;

/// Returns true if Microphone permission has been granted.
/// On non-macOS platforms, always returns false.
pub fn check_microphone() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe {
            let cls = objc_getClass(b"AVCaptureDevice\0".as_ptr());
            let sel = sel_registerName(b"authorizationStatusForMediaType:\0".as_ptr());
            let status = msg_send_authorization_status(cls, sel, AVMediaTypeAudio);
            status == AV_AUTHORIZATION_STATUS_AUTHORIZED
        }
    }
    #[cfg(not(target_os = "macos"))]
    false
}

/// Requests microphone access from the OS.
///
/// - Status `NotDetermined`: calls `requestAccessForMediaType:completionHandler:`
///   which shows the macOS system permission dialog.
/// - Status `Denied` or `Restricted`: opens System Settings > Privacy & Security > Microphone
///   (the OS does not allow re-prompting once denied).
///
/// The Tauri command returns immediately; the frontend polls `check_microphone_permission`
/// to detect when the user has responded.
pub fn request_microphone_access() {
    #[cfg(target_os = "macos")]
    {
        unsafe {
            let cls = objc_getClass(b"AVCaptureDevice\0".as_ptr());
            let status_sel = sel_registerName(b"authorizationStatusForMediaType:\0".as_ptr());
            let status = msg_send_authorization_status(cls, status_sel, AVMediaTypeAudio);

            if status == AV_AUTHORIZATION_STATUS_NOT_DETERMINED {
                // Show the macOS system permission dialog via a global block.
                let block = BlockLayout {
                    isa: _NSConcreteGlobalBlock,
                    flags: 1 << 28, // BLOCK_IS_GLOBAL
                    reserved: 0,
                    invoke: microphone_block_invoke,
                    descriptor: &MICROPHONE_BLOCK_DESCRIPTOR,
                };
                let request_sel =
                    sel_registerName(b"requestAccessForMediaType:completionHandler:\0".as_ptr());
                msg_send_request_access(cls, request_sel, AVMediaTypeAudio, &block);
            } else {
                // Denied or Restricted — OS won't re-show dialog; open Settings instead.
                let result = std::process::Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
                    .spawn();
                if let Err(e) = result {
                    tracing::warn!("Failed to open Microphone settings: {}", e);
                }
            }
        }
    }
}

/// Tauri command: check if Microphone permission is granted.
#[tauri::command]
pub async fn check_microphone_permission() -> Result<bool, String> {
    Ok(check_microphone())
}

/// Tauri command: request Microphone permission or open System Settings if denied.
#[tauri::command]
pub async fn request_microphone_permission() -> Result<(), String> {
    request_microphone_access();
    Ok(())
}
```

Then add one test to the existing `#[cfg(test)]` block:
```rust
    #[test]
    fn test_check_microphone_returns_bool() {
        // Verifies the FFI call (or non-macOS stub) does not panic.
        // Does not assert true/false — depends on system TCC grant state.
        let _ = check_microphone();
    }
```

**Important: The `BlockLayout` struct is stack-allocated inside `request_microphone_access()`. This is safe because `msg_send_request_access` is synchronous for the purposes of the call itself (the completion handler may run async, but the block pointer is valid for the duration of the `requestAccessForMediaType:completionHandler:` call dispatch). The block IS used asynchronously by the OS, but because `microphone_block_invoke` only captures `_granted` (no external state), and the `BlockDescriptor` and function pointer are static, the OS can safely call the block after our frame returns.**

**Actually: Stack-allocated block is NOT safe for async callbacks.** Use a `Box::leak` approach OR make the block static:

```rust
// Replace the stack-allocated block in request_microphone_access() with a static:
static MICROPHONE_REQUEST_BLOCK: std::sync::OnceLock<BlockLayout> = std::sync::OnceLock::new();

// In request_microphone_access(), inside the NotDetermined branch:
let block = MICROPHONE_REQUEST_BLOCK.get_or_init(|| BlockLayout {
    isa: unsafe { _NSConcreteGlobalBlock },
    flags: 1 << 28, // BLOCK_IS_GLOBAL
    reserved: 0,
    invoke: microphone_block_invoke,
    descriptor: &MICROPHONE_BLOCK_DESCRIPTOR,
});
msg_send_request_access(cls, request_sel, AVMediaTypeAudio, block as *const _);
```

This ensures the block lives for the entire program lifetime. `OnceLock` guarantees single initialization. Since `BlockLayout` implements `Sync` (declared above), this is sound.

---

### Task 2: `lib.rs` Changes

**Modify the existing `invoke_handler`** — add two commands to the existing list:
```rust
.invoke_handler(tauri::generate_handler![
    permissions::check_accessibility_permission,
    permissions::request_accessibility_permission,
    permissions::check_microphone_permission,
    permissions::request_microphone_permission,
])
```
**Nothing else in `lib.rs` changes.**

---

### Task 3: `onboarding.html` Addition

Append inside `#onboarding-app`, directly after the closing `</div>` of `#step-accessibility`:
```html
    <div class="step" id="step-microphone">
      <h1>Microphone Permission</h1>
      <p>ICantSpell needs Microphone permission to capture your voice while you hold the push-to-talk key. There is no background listening — the microphone is active only while the key is held.</p>
      <p>Click "Grant Microphone" to allow access. If the system dialog does not appear, click it again to open System Settings.</p>
      <div id="microphone-status"></div>
      <button id="btn-grant-microphone">Grant Microphone</button>
      <button id="btn-skip-microphone">Skip for Now</button>
    </div>
```

---

### Task 4: `onboarding.js` Changes

**Change 1 — Replace both stubs** (there are exactly two occurrences of `// Story 2.3 will implement: showStep("step-microphone");`):

Inside `startAccessibilityPolling`, in the `if (granted)` block:
```js
// BEFORE:
console.debug("[ICantSpell] onboarding: accessibility granted — mic step pending Story 2.3");
// Story 2.3 will implement: showStep("step-microphone");

// AFTER:
showStep("step-microphone");
```

Inside `btnSkipAccessibility` click handler:
```js
// BEFORE:
console.debug("[ICantSpell] onboarding: accessibility skipped — mic step pending Story 2.3");
// Story 2.3 will implement: showStep("step-microphone");

// AFTER:
showStep("step-microphone");
```

**Change 2 — Append microphone section** at the end of `onboarding.js` (after all accessibility code):

```js
// ── Microphone Step ──────────────────────────────────────────────────────────

let microphonePollInterval = null;

function stopMicrophonePolling() {
  if (microphonePollInterval !== null) {
    clearInterval(microphonePollInterval);
    microphonePollInterval = null;
  }
}

function setMicrophoneStatus(message, isWarning = false) {
  const statusEl = document.getElementById("microphone-status");
  if (statusEl) {
    statusEl.textContent = message;
    statusEl.className = isWarning ? "status-warning" : "status-info";
  }
}

async function startMicrophonePolling() {
  const btn = document.getElementById("btn-grant-microphone");
  if (btn) btn.disabled = true;
  setMicrophoneStatus("Waiting for permission…");

  let elapsed = 0;
  const POLL_INTERVAL_MS = 1000;
  const TIMEOUT_MS = 30000;

  microphonePollInterval = setInterval(async () => {
    elapsed += POLL_INTERVAL_MS;
    try {
      const granted = await invoke("check_microphone_permission");
      if (granted) {
        stopMicrophonePolling();
        if (btn) btn.disabled = false;
        setMicrophoneStatus("Microphone permission granted ✓");
        // Story 2.4 will implement: showStep("step-hotkey");
      } else if (elapsed >= TIMEOUT_MS) {
        stopMicrophonePolling();
        if (btn) btn.disabled = false;
        setMicrophoneStatus(
          "Permission not yet granted. Voice typing will not work until you grant it in System Settings.",
          true
        );
      }
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error checking microphone permission:", e);
    }
  }, POLL_INTERVAL_MS);
}

const btnGrantMicrophone = document.getElementById("btn-grant-microphone");
if (btnGrantMicrophone) {
  btnGrantMicrophone.addEventListener("click", async () => {
    try {
      await invoke("request_microphone_permission");
      await startMicrophonePolling();
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error requesting microphone permission:", e);
    }
  });
}

const btnSkipMicrophone = document.getElementById("btn-skip-microphone");
if (btnSkipMicrophone) {
  btnSkipMicrophone.addEventListener("click", () => {
    stopMicrophonePolling();
    setMicrophoneStatus(
      "Skipped. Voice typing will not work until Microphone permission is granted in System Settings.",
      true
    );
    // Story 2.4 will implement: showStep("step-hotkey");
  });
}
```

---

### Task 5: `styles.css` Addition

Add after the `#onboarding-app #btn-skip-accessibility` block, before `@media (prefers-color-scheme: dark)`:

```css
#onboarding-app #btn-skip-microphone {
  margin-top: 0.5rem;
  margin-left: 0.5rem;
}
```

---

### Architecture Compliance

- `permissions.rs` remains the **only** module calling `AVCaptureDevice` or any permission API — architecture rule enforced.
- All Tauri commands are `async fn` returning `Result<T, String>` — matches architecture pattern.
- `AVMediaTypeAudio` extern static is linked from `AVFoundation.framework` (no new Cargo crate).
- `objc_msgSend` typed aliases use `#[link_name = "objc_msgSend"]` — correct Rust FFI pattern for ObjC dispatch.
- IPC command names `check_microphone_permission` and `request_microphone_permission` use `snake_case` — matches naming rules.
- `tracing::warn!` used for errors — no `println!` or `unwrap()` outside tests.
- Frontend uses `invoke()` from `@tauri-apps/api/core` — correct Tauri v2 import.

### What NOT to Do

- Do NOT add `objc2`, `objc2-av-foundation`, `block2`, or any new crate — all ObjC dispatch is via `objc_msgSend` FFI.
- Do NOT create a second `invoke_handler` in `lib.rs` — add to the existing one.
- Do NOT implement PTT hotkey configuration — that is Story 2.4.
- Do NOT add a "Finish" or "Done" button — that is Story 2.5.
- Do NOT add `showStep("step-hotkey")` calls — `#step-hotkey` does not exist yet; stub with comment.
- Do NOT add revocation monitoring — that is Story 2.6.
- Do NOT change `AVMediaTypeAudio` to a string literal — use the `extern "C"` static which is the real `NSString *`.
- Do NOT use `Box::new(BlockLayout { ... })` for the block — use `OnceLock` static to guarantee lifetime.
- Do NOT modify `tauri.conf.json` or `capabilities/default.json` — already correct from Stories 1.4 / 2.2.
- Do NOT add `Cargo.toml` entries.

### Files to Touch

| File | Action | Why |
|------|---------|-----|
| `src-tauri/src/permissions.rs` | MODIFY | Add microphone FFI block, check/request functions, two Tauri commands, one test |
| `src-tauri/src/lib.rs` | MODIFY | Add two microphone commands to existing `invoke_handler` |
| `src/onboarding.html` | MODIFY | Append `#step-microphone` div inside `#onboarding-app` |
| `src/onboarding.js` | MODIFY | Replace two `showStep` stubs + append microphone step section |
| `src/styles.css` | MODIFY | Add `#btn-skip-microphone` margin rule |

**Files NOT touched:**
- `src-tauri/Cargo.toml` — no new crates
- `src-tauri/tauri.conf.json` — correct from Story 1.4
- `src-tauri/capabilities/default.json` — `core:default` already covers onboarding window
- `src-tauri/src/config.rs`, `error.rs`, `stt/` — no changes
- All other stub modules (`audio.rs`, `hotkey.rs`, `injection.rs`, `overlay.rs`, `models.rs`)

### Test Count

| Scope | Count |
|-------|-------|
| Pre-existing (config + stt + is_first_launch + accessibility) | 10 |
| New: `test_check_microphone_returns_bool` in `permissions.rs` | 1 |
| **Total** | **11** |

The new test runs on all platforms (macOS returns actual TCC result, non-macOS returns `false`). It verifies the `AVCaptureDevice` FFI chain does not panic.

### Previous Story Intelligence (Story 2.2)

- `permissions.rs` after Story 2.2: `check_accessibility`, `open_accessibility_settings`, `check_accessibility_permission`, `request_accessibility_permission`, one test. Module comment says "ONLY module that directly calls AXIsProcessTrusted()" — update the doc comment to also mention `AVCaptureDevice`.
- `lib.rs` after Story 2.2: `pub mod permissions;` exists; `invoke_handler` has 2 commands. Add to the existing list — do NOT recreate it.
- `onboarding.js` after Story 2.2: the two `// Story 2.3 will implement: showStep("step-microphone");` stub comments are present and MUST be replaced. Verify them by grepping before editing.
- `styles.css` after Story 2.2: `#onboarding-app #btn-skip-accessibility` rule exists just before the `@media (prefers-color-scheme: dark)` block. Insert microphone rule after it.
- Test baseline is **10 tests** after Story 2.2 (9 from 2.1 + 1 new accessibility test from 2.2). This story adds 1 more → **11 total**.
- All 10 existing tests must continue passing with zero clippy warnings. `cargo clippy --all-targets -- -D warnings` is the gate.

### References

- [Source: epics.md § Story 2.3] — User story, all 4 acceptance criteria
- [Source: architecture.md § Permissions Architecture] — `permissions.rs` is sole `AVCaptureDevice` caller
- [Source: architecture.md § Format Patterns] — `async fn`, `Result<T, String>` at command boundary
- [Source: architecture.md § Naming Patterns] — `snake_case` IPC command names
- [Source: architecture.md § Logging Patterns] — `tracing::warn!` for recoverable issues
- [Source: architecture.md § Frontend Architecture] — `invoke()` from `@tauri-apps/api/core`; no direct OS API calls from JS
- [Source: story 2-2 Completion Notes] — 10 tests baseline; `permissions.rs` current structure; two `showStep("step-microphone")` stubs in `onboarding.js`
- [Source: story 2-2 Dev Notes § What NOT to Do] — "Do NOT implement microphone permission request — that is Story 2.3"
- [Source: story 2-2 Dev Notes § Task 4] — `showStep` function signature and accessibility polling pattern to mirror exactly
- [Clang Block ABI] — https://clang.llvm.org/docs/Block-ABI-Apple.html — `BlockLayout` struct definition, `BLOCK_IS_GLOBAL` flag value (`1 << 28`), `_NSConcreteGlobalBlock` isa pointer

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- Fixed `unsafe impl Send` required for `OnceLock<BlockLayout>` — `Sync` alone was insufficient; raw pointers in `BlockLayout` don't auto-implement `Send`, so explicit `unsafe impl Send` added (safe: all contained pointers are static).
- Fixed clippy `manual_c_str_literals` — changed `b"...\0".as_ptr()` → `c"...".as_ptr()` throughout; updated extern declarations from `*const u8` to `*const std::ffi::c_char` to match.
- Fixed `clashing_extern_declarations` — split the two `objc_msgSend` typed aliases into separate `extern "C"` blocks; applied `#[allow(clashing_extern_declarations)]` to the second (the 4-arg `msg_send_request_access` variant).

### Completion Notes List

- Extended `permissions.rs` with full microphone permission flow: `AVFoundation` framework linked for `AVMediaTypeAudio` constant; ObjC runtime accessed via `objc_getClass` / `sel_registerName` / `objc_msgSend` FFI (no new crates).
- `check_microphone()` calls `[AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio]` via `msg_send_authorization_status`; returns `true` only for `AVAuthorizationStatusAuthorized` (value 3).
- `request_microphone_access()` checks status first: `NotDetermined` → dispatches `requestAccessForMediaType:completionHandler:` with a static global ObjC block (`OnceLock<BlockLayout>` with `BLOCK_IS_GLOBAL` flag); `Denied`/`Restricted` → opens `x-apple.systempreferences:...Privacy_Microphone` URL.
- Two Tauri commands registered: `check_microphone_permission` and `request_microphone_permission`, added to existing `invoke_handler` in `lib.rs`.
- `onboarding.html`: appended `#step-microphone` div with heading, two explanation paragraphs, status div, Grant and Skip buttons.
- `onboarding.js`: replaced both `showStep("step-microphone")` stubs from Story 2.2; added full microphone section — `stopMicrophonePolling`, `setMicrophoneStatus`, `startMicrophonePolling` (1s poll, 30s timeout matching accessibility pattern), grant/skip button wiring; next-step navigation stubbed with comment for Story 2.4.
- `styles.css`: added `#btn-skip-microphone` margin rule matching `#btn-skip-accessibility`.
- 11/11 tests pass; `cargo clippy --all-targets -- -D warnings` — zero warnings.

### File List

- `src-tauri/src/permissions.rs` — added AVFoundation framework link, ObjC runtime FFI, `BlockLayout`/`BlockDescriptor` structs, `OnceLock` static block, `check_microphone`, `request_microphone_access`, two Tauri commands, one new test
- `src-tauri/src/lib.rs` — added `check_microphone_permission` and `request_microphone_permission` to `invoke_handler`
- `src/onboarding.html` — appended `#step-microphone` step div
- `src/onboarding.js` — replaced two `showStep` stubs; appended microphone step section
- `src/styles.css` — added `#btn-skip-microphone` margin rule

## Change Log

- 2026-04-29: Story 2.3 created — Microphone Permission Request ready for dev.
- 2026-04-29: Implemented Story 2.3 — AVFoundation FFI for microphone permission check/request via objc_msgSend; ObjC block ABI for requestAccess completion handler (OnceLock static); onboarding.js microphone step wired with 1s polling/30s timeout; showStep stubs from Story 2.2 replaced. 11/11 tests pass, zero clippy warnings.
