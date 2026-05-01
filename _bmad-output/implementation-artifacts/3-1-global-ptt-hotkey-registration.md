# Story 3.1: Global PTT Hotkey Registration

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user,
I want my configured PTT hotkey to be active globally regardless of which app is in focus,
so that I can start dictating from anywhere without switching to ICantSpell first.

## Acceptance Criteria

1. **Given** `hotkey.rs` registers a global event listener for the configured hotkey, **When** the user holds the hotkey in any foreground app, **Then** `hotkey.rs` fires a keydown event that signals `audio.rs` to begin capture.

2. **Given** the user releases the hotkey, **When** `hotkey.rs` fires a keyup event, **Then** it signals `audio.rs` to stop capture and hand off the buffer.

3. **Given** the user switches focus between apps while the app is running, **When** they press the PTT hotkey in any app, **Then** the hotkey fires correctly — it is not scoped to ICantSpell's window (NFR8).

4. **Given** the app reads `ptt_hotkey` from config on startup, **When** the hotkey binding is loaded, **Then** the global registration uses the user's saved hotkey, not a hardcoded value.

5. **And** `cargo test` includes a unit test verifying the hotkey event parsing logic.

## Tasks / Subtasks

- [x] Task 1: Add `rdev` dependency to `Cargo.toml` (AC: 1, 2, 3)
  - [x] Add `rdev = "0.5"` to `[dependencies]` in `src-tauri/Cargo.toml` — this is the ONLY new dependency for this story
  - [x] Verify `cargo check` compiles with the new dependency

- [x] Task 2: Implement hotkey string parsing in `hotkey.rs` (AC: 4, 5)
  - [x] Replace the stub comment in `src-tauri/src/hotkey.rs` with the full module implementation
  - [x] Implement `parse_hotkey(hotkey_str: &str) -> Result<HotkeyBinding, AppError>` that converts config strings to an internal `HotkeyBinding` representation
  - [x] `HotkeyBinding` struct: `{ key: rdev::Key, modifiers: Vec<rdev::Key> }` — for bare modifiers like `"AltRight"`, `modifiers` is empty and `key` is the modifier itself; for combos like `"Alt+Space"`, `modifiers` contains `[Key::Alt]` and `key` is `Key::Space`
  - [x] Mapping from config strings to `rdev::Key` (the onboarding stores `KeyboardEvent.code` values):
    - `"AltRight"` → `Key::AltGr` (macOS Right Option)
    - `"AltLeft"` → `Key::Alt`
    - `"MetaLeft"` → `Key::MetaLeft`
    - `"MetaRight"` → `Key::MetaRight`
    - `"ControlLeft"` → `Key::ControlLeft`
    - `"ControlRight"` → `Key::ControlRight`
    - `"ShiftLeft"` → `Key::ShiftLeft`
    - `"ShiftRight"` → `Key::ShiftRight`
    - `"Space"` → `Key::Space`
    - `"Enter"` → `Key::Return`
    - `"F1"`–`"F12"` → `Key::F1`–`Key::F12`
    - For combos: split on `"+"`, parse modifiers (e.g., `"Alt"` → `Key::Alt`) and the final segment as the key
  - [x] Return `AppError::Hotkey(format!("Unknown hotkey: {}", s))` for unrecognized strings

- [x] Task 3: Implement PTT event channel types (AC: 1, 2)
  - [x] Define `pub enum PttEvent { Pressed, Released }` in `hotkey.rs` — this is the signal that `audio.rs` (Story 3.2) will receive
  - [x] Define `pub type PttSender = std::sync::mpsc::Sender<PttEvent>` and `pub type PttReceiver = std::sync::mpsc::Receiver<PttEvent>` — use `std::sync::mpsc` (not `tokio::sync::mpsc`) since the listener runs on a `std::thread` and `audio.rs` will also use a `std::thread`

- [x] Task 4: Implement global event listener loop (AC: 1, 2, 3)
  - [x] Implement `pub fn start_hotkey_listener(hotkey_str: &str, tx: PttSender) -> Result<(), AppError>` that:
    1. Calls `parse_hotkey(hotkey_str)` to get the `HotkeyBinding`
    2. Spawns a `std::thread::spawn` with a descriptive thread name `"ptt-hotkey"`
    3. Inside the thread: calls `rdev::listen(callback)` where the callback:
       - On `EventType::KeyPress(key)`: if the pressed key matches the binding's key AND all required modifiers are currently held → send `PttEvent::Pressed` via `tx` (only send once per hold cycle — track `is_pressed: bool` state to avoid key-repeat flooding)
       - On `EventType::KeyRelease(key)`: if the released key matches the binding's key → send `PttEvent::Released` via `tx` and reset `is_pressed = false`
    4. For modifier-only bindings (e.g., bare `"AltRight"`): treat the modifier key itself as the trigger — no additional key needed
    5. For combo bindings (e.g., `"Alt+Space"`): track modifier held state via a `HashSet<rdev::Key>`, and fire `Pressed` only when the final key is pressed while all modifiers are held
  - [x] Use `move` closure to capture `tx` and `binding` into the spawned thread
  - [x] If `rdev::listen` returns an error (e.g., Accessibility permission not granted), log `tracing::error!` — do NOT panic; the thread will simply exit and PTT will be non-functional (Epic 3.5 handles this gracefully)

- [x] Task 5: Wire `start_hotkey_listener` into `lib.rs` startup (AC: 4)
  - [x] In `lib.rs` setup closure, after `permissions::start_permission_monitor(...)`:
    1. Load settings: `let settings = config::load(app.handle());`
    2. Create channel: `let (ptt_tx, _ptt_rx) = std::sync::mpsc::channel();` — the receiver will be used by `audio.rs` in Story 3.2; for now, create it but drop it (the sender will still work, sends will return `Err` which is fine)
    3. Call `hotkey::start_hotkey_listener(&settings.ptt_hotkey, ptt_tx).unwrap_or_else(|e| tracing::error!("Failed to start hotkey listener: {}", e));`
  - [x] Add `pub mod hotkey;` to `lib.rs` module declarations (it's NOT currently declared — only `config`, `error`, `permissions`, `stt` are declared)
  - [x] NOTE: The `_ptt_rx` receiver is intentionally unused in this story. Story 3.2 will refactor startup to pass the receiver to the audio capture pipeline. Do NOT create a Tauri command for PTT — it's an internal pipeline, not a frontend-invoked action.

- [x] Task 6: Add unit tests for hotkey parsing (AC: 5)
  - [x] Add `#[cfg(test)] mod tests` block in `hotkey.rs` with:
    - `test_parse_bare_modifier_alt_right`: parse `"AltRight"` → key is `Key::AltGr`, no modifiers
    - `test_parse_bare_modifier_meta_left`: parse `"MetaLeft"` → key is `Key::MetaLeft`, no modifiers
    - `test_parse_combo_alt_space`: parse `"Alt+Space"` → key is `Key::Space`, modifiers contain `Key::Alt`
    - `test_parse_combo_control_shift_f5`: parse `"Control+Shift+F5"` → key is `Key::F5`, modifiers contain `Key::ControlLeft` and `Key::ShiftLeft`
    - `test_parse_unknown_key_returns_error`: parse `"FooBar"` → returns `Err(AppError::Hotkey(..))`
    - `test_parse_empty_string_returns_error`: parse `""` → returns `Err(AppError::Hotkey(..))`
  - [x] Do NOT test `start_hotkey_listener` directly — it requires Accessibility permission and spawns an OS event tap. The parsing logic is the testable unit.

- [x] Task 7: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings
  - [x] `cargo test` — all existing tests pass (14 from Stories 1.x–2.6) + new hotkey parsing tests
  - [x] Verify `hotkey.rs` compiles on macOS — `rdev` uses `CGEventTap` under the hood which requires the ApplicationServices framework (already linked for `AXIsProcessTrusted` in `permissions.rs`)

### Review Findings

- [x] [Review][Patch] Empty hotkey triggers misleading error log on first launch — `Settings::default()` sets `ptt_hotkey` to empty string; `start_hotkey_listener("")` fails with `AppError::Hotkey("Empty hotkey string")` logged at `error!` level. Should skip listener startup when hotkey is empty and log at `info` level instead. [src-tauri/src/lib.rs:89]
- [x] [Review][Patch] Generic modifier left/right mismatch in combo bindings — combo `"Alt+Space"` maps modifier `"Alt"` to `Key::Alt` (left), but if user holds right Alt, rdev fires `Key::AltGr` which is NOT in `held_modifiers`. Combo never activates with right-side modifier. Fix: accept either Left or Right variant when checking held modifiers. [src-tauri/src/hotkey.rs:168-172]
- [x] [Review][Patch] Whitespace not trimmed in `parse_hotkey` — `hotkey_str.split('+')` passes segments directly to `map_key`; strings like `"Alt + Space"` from manual config edits fail. Add `.trim()` to each segment. [src-tauri/src/hotkey.rs:109]
- [x] [Review][Patch] `save_ptt_hotkey` does not validate hotkey string — frontend can capture key codes not in `map_key` (Digit keys, arrow keys, numpad); invalid string is persisted and only fails on next launch. Call `hotkey::parse_hotkey()` before persisting to catch unsupported keys early. [src-tauri/src/config.rs:74]
- [x] [Review][Defer] No shutdown/re-registration mechanism for hotkey listener — changing hotkey in settings requires app restart; `rdev::listen` thread has no cancellation. [src-tauri/src/hotkey.rs] — deferred, future story for live settings reload
- [x] [Review][Defer] `rdev::listen` failure has no recovery path — if CGEventTap is invalidated (e.g., permission revoked at runtime), thread exits with no retry or user notification. [src-tauri/src/hotkey.rs:193] — deferred, Story 3.5 integration handles graceful degradation

## Dev Notes

### Library Choice: `rdev` (not `tauri-plugin-global-shortcut`)

The architecture mentions "rdev or tauri global shortcut plugin." We use `rdev` because:

1. **Bare modifier key support**: The PTT hotkey can be a single modifier like `"AltRight"`. `tauri-plugin-global-shortcut` requires a key+modifier combination — it cannot register a bare modifier as a shortcut.
2. **KeyPress + KeyRelease events**: `rdev::listen()` provides both `EventType::KeyPress` and `EventType::KeyRelease` for any key, which maps directly to PTT hold/release semantics.
3. **No left/right distinction in Tauri plugin**: `tauri-plugin-global-shortcut` uses `Modifiers::ALT` (no left/right), but the onboarding stores sided codes like `"AltRight"` vs `"AltLeft"`.
4. **Accessibility permission**: `rdev` requires Accessibility permission on macOS, which the app already requests in onboarding (Story 2.2). No additional permission needed.

**Do NOT use `tauri-plugin-global-shortcut`** — it cannot satisfy the requirements for bare modifier PTT keys.

### Hotkey String Format (from onboarding.js)

The onboarding wizard stores `KeyboardEvent.code` values in config. The format is:
- **Bare modifier**: `"AltRight"`, `"MetaLeft"`, `"ShiftRight"`, etc.
- **Combo**: `"Alt+Space"`, `"Control+Shift+F5"` — logical modifier names joined with `+`, final segment is the key code
- **Bare key**: `"F5"`, `"Space"` — a single non-modifier key

The modifier names in combos are logical (unsided): `"Alt"`, `"Meta"`, `"Control"`, `"Shift"`. When parsing a combo's modifier segment, map to the LEFT variant by convention: `"Alt"` → `Key::Alt`, `"Control"` → `Key::ControlLeft`, `"Shift"` → `Key::ShiftLeft`, `"Meta"` → `Key::MetaLeft`.

### rdev Key Repeat Behavior

`rdev::listen()` fires repeated `KeyPress` events when a key is held (OS key repeat). The listener MUST track `is_pressed: bool` state and only send `PttEvent::Pressed` on the FIRST press event, ignoring subsequent repeats until a `KeyRelease` is received. Without this guard, `audio.rs` would receive hundreds of `Pressed` signals during a single PTT hold.

### Thread Model

```
main thread (Tauri)
  └── std::thread "ptt-hotkey"  ← rdev::listen() blocks this thread forever
        └── on match: tx.send(PttEvent::Pressed | Released)
              └── received by audio.rs thread (Story 3.2) via rx
```

`rdev::listen()` is a blocking call that installs a CGEventTap and runs a CFRunLoop. It MUST run on its own dedicated `std::thread`. Do NOT use `tokio::spawn` or `tokio::task::spawn_blocking` — the CGEventTap requires a persistent CFRunLoop that runs for the lifetime of the app.

### Sleep/Wake Recovery (NFR7)

On macOS, CGEventTap registrations survive sleep/wake cycles — the OS automatically re-activates them on wake. No explicit `NSWorkspace.didWakeNotification` handler is needed in this story for the hotkey itself. However, if the CGEventTap becomes disabled (e.g., due to permission revocation), `rdev::listen` will return an error. Story 3.5 (integration) will handle re-registration if needed.

### Error Handling

- `parse_hotkey` returns `Result<HotkeyBinding, AppError>` using `AppError::Hotkey(String)`.
- `start_hotkey_listener` returns `Result<(), AppError>` — errors from parsing propagate; errors from `rdev::listen` are logged but not propagated (the thread has already been spawned).
- If `tx.send()` returns `Err` (receiver dropped), silently ignore — this means `audio.rs` hasn't started yet or has shut down. Not an error condition during startup.

### Current State of Files Being Modified

**`src-tauri/src/hotkey.rs` — current stub (3 lines):**
```rust
// Global PTT hotkey registration — implemented in Story 3.1
// Uses: rdev or tauri global shortcut plugin
// See architecture.md § Hotkey Architecture
```
Replace entirely with the full module implementation.

**`src-tauri/src/lib.rs` — current module declarations (line 1–4):**
```rust
pub mod config;
pub mod error;
pub mod permissions;
pub mod stt;
```
Add `pub mod hotkey;` after `pub mod error;` (alphabetical order).

**`src-tauri/src/lib.rs` — current setup closure (after line 84):**
```rust
config::ensure_defaults(app.handle())?;
permissions::start_permission_monitor(app.handle().clone());

if first_launch { ... }
```
Add hotkey listener startup between `start_permission_monitor` and `if first_launch`.

**`src-tauri/Cargo.toml` — current dependencies (lines 18–25):**
```toml
[dependencies]
tauri = { version = "2", features = ["macos-private-api", "tray-icon"] }
tauri-plugin-store = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "1"
```
Add `rdev = "0.5"` after `thiserror`.

### Architecture Compliance

- `hotkey.rs` is the ONLY module that registers global event listeners — no other module touches the event loop (architecture § System API Boundary).
- IPC: This story does NOT emit any IPC events to the frontend. PTT signals are internal (`std::sync::mpsc` channel between Rust threads). The `transcription_ready` event is emitted in Story 3.5 after the full pipeline runs.
- Naming: `snake_case` functions (`parse_hotkey`, `start_hotkey_listener`), `PascalCase` types (`PttEvent`, `HotkeyBinding`), `SCREAMING_SNAKE_CASE` constants (none needed in this story).
- Error boundary: `start_hotkey_listener` returns `Result<(), AppError>` — errors are converted to `String` only at Tauri command boundaries, which this module does not have.

### What NOT to Do

- Do NOT add `tauri-plugin-global-shortcut` to `Cargo.toml` — it cannot handle bare modifier PTT keys.
- Do NOT add `tokio` features or use `tokio::spawn` — `rdev::listen` requires a persistent `std::thread`.
- Do NOT create any Tauri commands (`#[tauri::command]`) in `hotkey.rs` — PTT is an internal pipeline, not frontend-invoked.
- Do NOT implement audio capture or transcription — that is Stories 3.2 and 3.3.
- Do NOT modify `capabilities/default.json` — no new plugin permissions needed.
- Do NOT add hotkey re-registration on wake — CGEventTap survives sleep/wake. Story 3.5 handles edge cases.
- Do NOT consume the `PttReceiver` in this story — just create it and let it be unused (`_ptt_rx`). Story 3.2 will wire it to the audio pipeline.

### Files to Touch

| File | Action | Why |
|------|--------|-----|
| `src-tauri/Cargo.toml` | MODIFY | Add `rdev = "0.5"` dependency |
| `src-tauri/src/hotkey.rs` | REPLACE | Replace 3-line stub with full module |
| `src-tauri/src/lib.rs` | MODIFY | Add `pub mod hotkey;` + startup wiring |

**Files NOT touched:**
- `src-tauri/src/audio.rs` — Story 3.2 will consume the `PttReceiver`
- `src-tauri/src/config.rs` — read-only usage; no changes needed
- `src-tauri/src/error.rs` — `AppError::Hotkey(String)` already exists
- `src-tauri/src/permissions.rs` — no changes
- `src-tauri/capabilities/default.json` — no new plugin permissions
- `src/` frontend files — no frontend changes in this story

### Test Count

| Scope | Count |
|-------|-------|
| Pre-existing (config + stt + is_first_launch + permissions) | 14 |
| New: hotkey parsing tests (6 tests) | 6 |
| **Total** | **20** |

### Previous Story Intelligence (Story 2.6)

- `permissions.rs` uses `std::thread::spawn` for background monitoring — same pattern should be used for the hotkey listener thread.
- `lib.rs` setup closure runs sequentially: `ensure_defaults` → `start_permission_monitor` → (new: `start_hotkey_listener`) → `if first_launch`.
- `serde_json` is already a dependency — available for test assertions.
- Previous code review found that `let _ =` silently swallows errors. Use `if let Err(e) =` with `tracing::warn!` for `tx.send()` failures that indicate a real problem (but `SendError` when receiver is dropped is expected during startup and can be silently ignored).
- `tracing` patterns: `tracing::info!` for startup, `tracing::error!` for failures, `tracing::debug!` for normal operation events.

### References

- [Source: epics.md § Story 3.1] — User story, acceptance criteria
- [Source: architecture.md § System API Boundary] — hotkey.rs is sole owner of global event tap
- [Source: architecture.md § Primary v1 Data Flow] — hotkey.rs detects keydown → signals audio.rs
- [Source: architecture.md § Implementation Patterns] — naming conventions, error boundary patterns
- [Source: architecture.md § Resolved Decision 3] — Sleep/wake: re-register PTT on wake (CGEventTap survives natively)
- [Source: onboarding.js lines 196–297] — Hotkey capture format: KeyboardEvent.code values, MODIFIER_CODES set, combo format with "+" separator
- [Source: config.rs] — `Settings.ptt_hotkey: String`, `DEFAULT_PTT_HOTKEY = "AltRight"`
- [Source: error.rs] — `AppError::Hotkey(String)` variant already defined

## Dev Agent Record

### Agent Model Used

claude-opus-4-6

### Debug Log References

No issues — all tasks implemented cleanly without debug iterations.

### Completion Notes List

- Added `rdev = "0.5"` dependency to `Cargo.toml` (rdev v0.5.3 resolved). Uses CGEventTap on macOS for global key event listening.
- Implemented `hotkey.rs` module: `parse_hotkey()` maps config strings (KeyboardEvent.code format from onboarding) to `rdev::Key` variants. Supports bare modifiers (`"AltRight"` → `Key::AltGr`), bare keys (`"F5"` → `Key::F5`), and combos (`"Alt+Space"` → modifiers `[Key::Alt]` + key `Key::Space`). Also maps all letter keys (`KeyA`–`KeyZ`).
- Defined `PttEvent { Pressed, Released }` enum and `PttSender`/`PttReceiver` type aliases using `std::sync::mpsc`.
- Implemented `start_hotkey_listener()`: spawns named `"ptt-hotkey"` thread running `rdev::listen()`. Tracks `is_pressed` state to prevent key-repeat flooding. For combo bindings, uses `HashSet<Key>` to track held modifiers. Errors from `rdev::listen` are logged via `tracing::error!`, not panicked.
- Wired into `lib.rs`: added `pub mod hotkey;`, loads settings, creates mpsc channel, starts listener in setup closure after `start_permission_monitor()`. Receiver is intentionally unused (`_ptt_rx`) — Story 3.2 will consume it.
- 6 new unit tests for hotkey parsing. 20/20 tests pass total. Zero clippy warnings.

### File List

- `src-tauri/Cargo.toml` — added `rdev = "0.5"` dependency
- `src-tauri/src/hotkey.rs` — replaced 3-line stub with full module (HotkeyBinding, PttEvent, parse_hotkey, start_hotkey_listener, map_key, is_modifier, 6 tests)
- `src-tauri/src/lib.rs` — added `pub mod hotkey;` declaration + hotkey listener startup wiring in setup closure

## Change Log

- 2026-04-30: Implemented Story 3.1 — Global PTT hotkey registration via rdev. parse_hotkey() maps config strings to rdev::Key; start_hotkey_listener() spawns CGEventTap listener thread; PttEvent channel for audio pipeline. 20/20 tests pass, zero clippy warnings.
- 2026-05-01: Code review complete — 4 patches applied (empty hotkey skip, modifier left/right matching, whitespace trimming, save_ptt_hotkey validation), 2 deferred (shutdown mechanism, rdev failure recovery). 20/20 tests pass.
