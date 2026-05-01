# Story 2.4: PTT Hotkey Configuration

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a new user,
I want to set my push-to-talk hotkey during onboarding,
so that I can start dictating immediately after setup completes.

## Acceptance Criteria

1. **Given** the onboarding wizard is on the hotkey configuration step, **When** the user clicks the "Set Hotkey" capture field and presses a key combination, **Then** the captured key combination is displayed (e.g., "⌥ Right Option" or "⌥ Space").

2. **Given** a hotkey is captured, **When** the user confirms it, **Then** it is written to `config.rs` via the Tauri store as `ptt_hotkey`.

3. **Given** the user does not set a hotkey, **When** they click "Use Default (Right Option)", **Then** the default `"AltRight"` is saved to config as `ptt_hotkey`.

4. **And** the saved hotkey persists across app restarts (verified via config roundtrip — `Settings.ptt_hotkey` survives serialize/deserialize cycle per Story 1.3 test pattern).

## Tasks / Subtasks

- [x] Task 1: Add `DEFAULT_PTT_HOTKEY` constant and `save_ptt_hotkey` Tauri command in `config.rs` (AC: 2, 3, 4)
  - [x] Add `pub const DEFAULT_PTT_HOTKEY: &str = "AltRight";` below the existing `DEFAULT_CONFIDENCE_THRESHOLD` constant
  - [x] Implement `#[tauri::command] pub async fn save_ptt_hotkey(app: tauri::AppHandle, hotkey: String) -> Result<(), String>` — loads current settings, sets `ptt_hotkey = hotkey`, saves back
  - [x] Add `#[test] fn test_default_ptt_hotkey_constant_value()` in the existing `#[cfg(test)]` block

- [x] Task 2: Register `save_ptt_hotkey` command in `lib.rs` (AC: 2)
  - [x] Add `config::save_ptt_hotkey` to the existing `invoke_handler` list (do NOT create a second `invoke_handler`)

- [x] Task 3: Add `#step-hotkey` div to `onboarding.html` (AC: 1)
  - [x] Append inside `#onboarding-app`, directly after the closing `</div>` of `#step-microphone`
  - [x] Include: `<h1>`, explanation `<p>`, `<div id="hotkey-capture-field">`, `<div id="hotkey-display">`, `<button id="btn-confirm-hotkey">`, `<button id="btn-skip-hotkey">`

- [x] Task 4: Wire hotkey step in `onboarding.js` (AC: 1, 2, 3)
  - [x] Replace both `// Story 2.4 will implement: showStep("step-hotkey");` stubs with `showStep("step-hotkey");` (there are exactly two occurrences — verify with grep before editing)
  - [x] Append hotkey step section at the end of `onboarding.js` with: display label map, `capturedHotkeyCode` variable, `isCapturing` flag, `formatHotkeyCode()` helper, capture-field click handler, document-level keydown listener, confirm button handler (invokes `save_ptt_hotkey`, then stubs next step for Story 2.5), skip button handler (invokes `save_ptt_hotkey` with `DEFAULT_HOTKEY_CODE`, then stubs next step)

- [x] Task 5: Add hotkey step CSS to `styles.css` (AC: 1)
  - [x] Add `#hotkey-capture-field` block: `border`, `padding`, `cursor: pointer`, `border-radius`, `text-align: center` styles
  - [x] Add `#hotkey-capture-field.capturing` block: distinct border/background to signal active capture
  - [x] Add `#onboarding-app #btn-skip-hotkey` margin rule (matching pattern of `#btn-skip-accessibility` and `#btn-skip-microphone`)
  - [x] Insert all hotkey rules BEFORE the `@media (prefers-color-scheme: dark)` block

- [x] Task 6: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
  - [x] `cargo test` — 12 tests pass (11 pre-existing + 1 new `test_default_ptt_hotkey_constant_value`)
  - [x] Manual: microphone step (grant or skip) → hotkey step appears → click capture field → field shows "Press a key combination…" → press Right Option → field resets, display shows "⌥ Right Option" → confirm button becomes active → confirm saves to config → skip button saves "AltRight" default

## Dev Notes

### No New Crates Required

This story adds one `#[tauri::command]` and frontend JS/HTML/CSS only. Everything needed is already present:
- `config::load()` and `config::save()` already handle the Tauri store in `config.rs`
- `tauri::AppHandle` is the only parameter needed beyond the hotkey string
- No macOS FFI, no new framework links — config layer is pure Rust/Serde

### Hotkey String Format

The hotkey is stored in `ptt_hotkey` as a JS `KeyboardEvent.code`-derived string:

| User Action | Stored Value | Displayed As |
|---|---|---|
| Presses Right Option alone | `"AltRight"` | `"⌥ Right Option"` |
| Presses Left Option alone | `"AltLeft"` | `"⌥ Left Option"` |
| Presses Right Command alone | `"MetaRight"` | `"⌘ Right Command"` |
| Presses Option + Space | `"Alt+Space"` | `"⌥ Space"` |
| Uses default (skip) | `"AltRight"` | `"⌥ Right Option"` |

- For a **modifier-only key press** (code is in the MODIFIER_CODES set): store `event.code` directly.
- For a **modifier + non-modifier combo**: store as `"<ModifierName>+<event.code>"` where `ModifierName` is the logical modifier name (`"Alt"`, `"Meta"`, `"Control"`, `"Shift"`), NOT the sided code.
- Escape and Tab are explicitly blocked as PTT hotkeys (they interfere with navigation).
- This format is intentionally simple for Story 3.1 (`hotkey.rs`) to parse later.

### Current State of Files Being Modified

**`src-tauri/src/config.rs` — current state (after Story 1.3):**
```rust
pub const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.85;
const STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";

pub struct Settings {
    pub ptt_hotkey: String,          // Default::default() returns String::new()
    pub selected_model: String,      // Default: "base"
    pub confidence_threshold: f32,   // Default: 0.85
}

pub fn load(app: &tauri::AppHandle) -> Settings { ... }
pub fn save(app: &tauri::AppHandle, settings: &Settings) -> Result<(), AppError> { ... }
pub fn ensure_defaults(app: &tauri::AppHandle) -> Result<(), AppError> { ... }

#[cfg(test)]
mod tests {
    // 5 existing tests: test_default_confidence_threshold, test_default_ptt_hotkey_is_empty,
    // test_default_selected_model, test_settings_json_roundtrip, test_settings_roundtrip_with_custom_values
}
```

**Add `DEFAULT_PTT_HOTKEY` immediately after `DEFAULT_CONFIDENCE_THRESHOLD`:**
```rust
pub const DEFAULT_PTT_HOTKEY: &str = "AltRight";
```

**Add `save_ptt_hotkey` AFTER `ensure_defaults()` and BEFORE `#[cfg(test)]`:**
```rust
/// Tauri command: update the PTT hotkey binding in persistent settings.
///
/// The hotkey string format is a `KeyboardEvent.code`-derived value from the
/// onboarding frontend (e.g., `"AltRight"`, `"Alt+Space"`). Story 3.1 (`hotkey.rs`)
/// is responsible for parsing this string into a platform-specific event tap.
#[tauri::command]
pub async fn save_ptt_hotkey(app: tauri::AppHandle, hotkey: String) -> Result<(), String> {
    let mut settings = load(&app);
    settings.ptt_hotkey = hotkey;
    save(&app, &settings).map_err(|e| e.to_string())
}
```

**Add one test to the existing `#[cfg(test)]` block:**
```rust
#[test]
fn test_default_ptt_hotkey_constant_value() {
    // The default hotkey must be non-empty and match the Right Option code.
    assert_eq!(DEFAULT_PTT_HOTKEY, "AltRight");
    assert!(!DEFAULT_PTT_HOTKEY.is_empty());
}
```

**`src-tauri/src/lib.rs` — current `invoke_handler` (after Story 2.3):**
```rust
.invoke_handler(tauri::generate_handler![
    permissions::check_accessibility_permission,
    permissions::request_accessibility_permission,
    permissions::check_microphone_permission,
    permissions::request_microphone_permission,
])
```
Add `config::save_ptt_hotkey` to this list — **do NOT create a second `invoke_handler`.**

**`src/onboarding.html` — current body (after Story 2.3):**
```html
<div id="onboarding-app">
  <div class="step active" id="step-welcome"> ... </div>
  <div class="step" id="step-accessibility"> ... </div>
  <div class="step" id="step-microphone"> ... </div>
</div>
```
Append `#step-hotkey` inside `#onboarding-app`, after `#step-microphone`.

**`src/onboarding.js` — current state (after Story 2.3):**
Two stub comments exist that Story 2.4 must replace:
```js
// Story 2.4 will implement: showStep("step-hotkey");
```
One appears inside `startMicrophonePolling` (on grant success, line ~141), one inside `btnSkipMicrophone` click handler (line ~177). Replace BOTH stubs with the real call. Then ADD the hotkey section at the end of the file.

**`src/styles.css` — current onboarding rules (after Story 2.3):**
```css
#onboarding-app #btn-skip-microphone {
  margin-top: 0.5rem;
  margin-left: 0.5rem;
}

@media (prefers-color-scheme: dark) { ... }
```
Insert hotkey CSS rules AFTER `#btn-skip-microphone` and BEFORE `@media (prefers-color-scheme: dark)`.

---

### Task 1: Exact `config.rs` Additions

After `DEFAULT_CONFIDENCE_THRESHOLD`:
```rust
pub const DEFAULT_PTT_HOTKEY: &str = "AltRight";
```

After `ensure_defaults()`, before `#[cfg(test)]`:
```rust
/// Tauri command: update the PTT hotkey binding in persistent settings.
///
/// The hotkey string format is a `KeyboardEvent.code`-derived value from the
/// onboarding frontend (e.g., `"AltRight"`, `"Alt+Space"`). Story 3.1 (`hotkey.rs`)
/// is responsible for parsing this string into a platform-specific event tap.
#[tauri::command]
pub async fn save_ptt_hotkey(app: tauri::AppHandle, hotkey: String) -> Result<(), String> {
    let mut settings = load(&app);
    settings.ptt_hotkey = hotkey;
    save(&app, &settings).map_err(|e| e.to_string())
}
```

In the `#[cfg(test)]` block, add after the last existing test:
```rust
    #[test]
    fn test_default_ptt_hotkey_constant_value() {
        assert_eq!(DEFAULT_PTT_HOTKEY, "AltRight");
        assert!(!DEFAULT_PTT_HOTKEY.is_empty());
    }
```

---

### Task 2: `lib.rs` Changes

Modify the existing `invoke_handler` — add one command:
```rust
.invoke_handler(tauri::generate_handler![
    permissions::check_accessibility_permission,
    permissions::request_accessibility_permission,
    permissions::check_microphone_permission,
    permissions::request_microphone_permission,
    config::save_ptt_hotkey,
])
```
**Nothing else in `lib.rs` changes.**

---

### Task 3: `onboarding.html` Addition

Append inside `#onboarding-app`, directly after the closing `</div>` of `#step-microphone`:
```html
    <div class="step" id="step-hotkey">
      <h1>Push-to-Talk Hotkey</h1>
      <p>Choose the key you'll hold to start dictating. Most people use Right Option — it doesn't interfere with normal typing.</p>
      <p>Click the field below, then press any key or combination.</p>
      <div id="hotkey-capture-field" tabindex="0">Click to capture hotkey</div>
      <div id="hotkey-display"></div>
      <button id="btn-confirm-hotkey" disabled>Use This Hotkey</button>
      <button id="btn-skip-hotkey">Use Default (Right Option)</button>
    </div>
```

---

### Task 4: `onboarding.js` Changes

**Change 1 — Replace both stubs** (there are exactly two occurrences of `// Story 2.4 will implement: showStep("step-hotkey");`):

Inside `startMicrophonePolling`, in the `if (granted)` block:
```js
// BEFORE:
// Story 2.4 will implement: showStep("step-hotkey");

// AFTER:
showStep("step-hotkey");
```

Inside `btnSkipMicrophone` click handler:
```js
// BEFORE:
// Story 2.4 will implement: showStep("step-hotkey");

// AFTER:
showStep("step-hotkey");
```

**Change 2 — Append hotkey section** at the end of `onboarding.js` (after all microphone code):

```js
// ── Hotkey Step ───────────────────────────────────────────────────────────────

const DEFAULT_HOTKEY_CODE = "AltRight";
const DEFAULT_HOTKEY_DISPLAY = "⌥ Right Option";

// Modifier-only key codes — pressing these alone is a valid PTT hotkey.
const MODIFIER_CODES = new Set([
  "AltLeft", "AltRight",
  "MetaLeft", "MetaRight",
  "ControlLeft", "ControlRight",
  "ShiftLeft", "ShiftRight",
]);

// Human-readable labels for common key codes.
const KEY_DISPLAY_MAP = {
  AltLeft: "⌥ Left Option",
  AltRight: "⌥ Right Option",
  MetaLeft: "⌘ Left Command",
  MetaRight: "⌘ Right Command",
  ControlLeft: "⌃ Left Control",
  ControlRight: "⌃ Right Control",
  ShiftLeft: "⇧ Left Shift",
  ShiftRight: "⇧ Right Shift",
  Space: "Space",
  Enter: "Return",
  Backspace: "Backspace",
};

// Modifier name → symbol for combo display (e.g. "Alt+Space" → "⌥ Space")
const MODIFIER_SYMBOLS = {
  Alt: "⌥",
  Meta: "⌘",
  Control: "⌃",
  Shift: "⇧",
};

function formatHotkeyCode(code) {
  // Single key (including standalone modifiers)
  if (KEY_DISPLAY_MAP[code]) return KEY_DISPLAY_MAP[code];
  // Modifier+key combo like "Alt+Space"
  const parts = code.split("+");
  return parts
    .map((p) => MODIFIER_SYMBOLS[p] || KEY_DISPLAY_MAP[p] || p)
    .join(" ");
}

let capturedHotkeyCode = null;
let isCapturing = false;

const captureField = document.getElementById("hotkey-capture-field");
const hotkeyDisplay = document.getElementById("hotkey-display");
const btnConfirmHotkey = document.getElementById("btn-confirm-hotkey");
const btnSkipHotkey = document.getElementById("btn-skip-hotkey");

function setCapturedHotkey(code) {
  capturedHotkeyCode = code;
  if (hotkeyDisplay) hotkeyDisplay.textContent = formatHotkeyCode(code);
  if (btnConfirmHotkey) btnConfirmHotkey.disabled = false;
}

if (captureField) {
  captureField.addEventListener("click", () => {
    isCapturing = true;
    captureField.textContent = "Press a key combination…";
    captureField.classList.add("capturing");
  });
}

function handleHotkeyKeydown(e) {
  if (!isCapturing) return;
  // Block Escape (cancels capture) and Tab (navigation)
  if (e.key === "Escape" || e.key === "Tab") {
    isCapturing = false;
    if (captureField) {
      captureField.classList.remove("capturing");
      captureField.textContent = capturedHotkeyCode
        ? "Click to change"
        : "Click to capture hotkey";
    }
    return;
  }
  e.preventDefault();
  e.stopPropagation();

  let code;
  if (MODIFIER_CODES.has(e.code)) {
    // Solo modifier key — store the sided code directly (e.g. "AltRight")
    code = e.code;
  } else {
    // Build modifier prefix from active modifier keys (logical, not sided)
    const mods = [];
    if (e.altKey) mods.push("Alt");
    if (e.metaKey) mods.push("Meta");
    if (e.ctrlKey) mods.push("Control");
    if (e.shiftKey) mods.push("Shift");
    code = mods.length > 0 ? mods.join("+") + "+" + e.code : e.code;
  }

  isCapturing = false;
  if (captureField) {
    captureField.classList.remove("capturing");
    captureField.textContent = "Click to change";
  }
  setCapturedHotkey(code);
}

document.addEventListener("keydown", handleHotkeyKeydown);

if (btnConfirmHotkey) {
  btnConfirmHotkey.addEventListener("click", async () => {
    if (!capturedHotkeyCode) return;
    try {
      await invoke("save_ptt_hotkey", { hotkey: capturedHotkeyCode });
      console.debug("[ICantSpell] onboarding: hotkey saved:", capturedHotkeyCode);
      // Story 2.5 will implement: showStep("step-validation");
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error saving hotkey:", e);
    }
  });
}

if (btnSkipHotkey) {
  btnSkipHotkey.addEventListener("click", async () => {
    try {
      await invoke("save_ptt_hotkey", { hotkey: DEFAULT_HOTKEY_CODE });
      console.debug("[ICantSpell] onboarding: default hotkey saved:", DEFAULT_HOTKEY_CODE);
      // Story 2.5 will implement: showStep("step-validation");
    } catch (e) {
      console.warn("[ICantSpell] onboarding: error saving default hotkey:", e);
    }
  });
}
```

---

### Task 5: `styles.css` Addition

Add after `#onboarding-app #btn-skip-microphone`, before `@media (prefers-color-scheme: dark)`:

```css
#onboarding-app #hotkey-capture-field {
  margin-top: 1rem;
  padding: 0.75rem 1rem;
  border: 2px dashed #aaa;
  border-radius: 8px;
  cursor: pointer;
  text-align: center;
  color: #555;
  font-size: 0.95em;
  user-select: none;
}

#onboarding-app #hotkey-capture-field.capturing {
  border-color: #396cd8;
  color: #396cd8;
  background-color: #f0f4ff;
}

#onboarding-app #hotkey-display {
  min-height: 1.5rem;
  margin: 0.5rem 0;
  font-weight: 600;
  text-align: center;
  color: #0f0f0f;
  font-size: 1.1em;
}

#onboarding-app #btn-confirm-hotkey {
  margin-top: 0.75rem;
}

#onboarding-app #btn-skip-hotkey {
  margin-top: 0.5rem;
  margin-left: 0.5rem;
}
```

Also add inside the existing `@media (prefers-color-scheme: dark)` block (after the last existing dark-mode rule):
```css
  #onboarding-app #hotkey-capture-field {
    border-color: #666;
    color: #aaa;
  }

  #onboarding-app #hotkey-capture-field.capturing {
    border-color: #7b9ef0;
    color: #7b9ef0;
    background-color: #1a2040;
  }

  #onboarding-app #hotkey-display {
    color: #f6f6f6;
  }
```

---

### Architecture Compliance

- `config.rs` remains the **only** module that reads/writes `ptt_hotkey` to the Tauri store — architecture rule enforced.
- `save_ptt_hotkey` is `async fn` returning `Result<(), String>` — matches Tauri command pattern.
- IPC command name `save_ptt_hotkey` uses `snake_case` — matches naming rules.
- `tracing::debug!` used for success logging — no `println!` or `unwrap()` outside tests.
- Frontend uses `invoke("save_ptt_hotkey", { hotkey: ... })` from `@tauri-apps/api/core` — correct Tauri v2 pattern.
- No new crates, no HTTP client, no framework links added.

### What NOT to Do

- Do NOT register the global hotkey — that is Story 3.1 (`hotkey.rs`). This story only saves the string.
- Do NOT add a "Finish" or "Done" button — that is Story 2.5.
- Do NOT add `showStep("step-validation")` calls — `#step-validation` does not exist yet; stub with comment.
- Do NOT modify `tauri.conf.json`, `capabilities/default.json`, or `Cargo.toml`.
- Do NOT create a second `invoke_handler` in `lib.rs`.
- Do NOT use `Box<dyn Fn>` or capture external state in the keydown listener — plain function reference is sufficient.
- Do NOT implement revocation monitoring — that is Story 2.6.
- Do NOT implement permission re-checking in this step — just save the hotkey.
- Do NOT use `objc_msgSend` or any FFI in this story.

### Files to Touch

| File | Action | Why |
|------|---------|-----|
| `src-tauri/src/config.rs` | MODIFY | Add `DEFAULT_PTT_HOTKEY` constant, `save_ptt_hotkey` Tauri command, one new test |
| `src-tauri/src/lib.rs` | MODIFY | Add `config::save_ptt_hotkey` to existing `invoke_handler` |
| `src/onboarding.html` | MODIFY | Append `#step-hotkey` div inside `#onboarding-app` |
| `src/onboarding.js` | MODIFY | Replace two `showStep` stubs + append hotkey step section |
| `src/styles.css` | MODIFY | Add `#hotkey-capture-field`, `#hotkey-display`, `#btn-confirm-hotkey`, `#btn-skip-hotkey` rules + dark-mode variants |

**Files NOT touched:**
- `src-tauri/Cargo.toml` — no new crates
- `src-tauri/tauri.conf.json` — correct from Story 1.4
- `src-tauri/capabilities/default.json` — `core:default` already covers onboarding window
- `src-tauri/src/permissions.rs` — no changes
- `src-tauri/src/error.rs`, `stt/` — no changes
- All stub modules (`audio.rs`, `hotkey.rs`, `injection.rs`, `overlay.rs`, `models.rs`)

### Test Count

| Scope | Count |
|-------|-------|
| Pre-existing (config + stt + is_first_launch + accessibility + microphone) | 11 |
| New: `test_default_ptt_hotkey_constant_value` in `config.rs` | 1 |
| **Total** | **12** |

### Previous Story Intelligence (Story 2.3)

- `onboarding.js` after Story 2.3: two `// Story 2.4 will implement: showStep("step-hotkey");` stub comments are present. Verify locations by grepping before editing.
- `lib.rs` after Story 2.3: `invoke_handler` has 4 commands (accessibility x2 + microphone x2). Add `config::save_ptt_hotkey` as the 5th — do NOT recreate the list.
- `styles.css` after Story 2.3: `#btn-skip-microphone` rule is the last onboarding rule before `@media (prefers-color-scheme: dark)`. Insert all new hotkey CSS after it, before the media query.
- Test baseline is **11 tests** after Story 2.3. This story adds 1 → **12 total**. All 11 existing tests must pass with zero clippy warnings.
- `config.rs` already has `pub fn save()` which `save_ptt_hotkey` calls. Import path in `save_ptt_hotkey` is `save(&app, &settings)` — no `use` statement needed since it's in the same module.

### References

- [Source: epics.md § Story 2.4] — User story, all 4 acceptance criteria
- [Source: architecture.md § Config Persistence] — Tauri store plugin; `ptt_hotkey` is a persisted field in `Settings`
- [Source: architecture.md § Format Patterns] — `async fn`, `Result<T, String>` at Tauri command boundary
- [Source: architecture.md § Naming Patterns] — `snake_case` IPC command names; `SCREAMING_SNAKE_CASE` constants
- [Source: architecture.md § Logging Patterns] — `tracing::debug!` for success milestones
- [Source: architecture.md § Frontend Architecture] — `invoke()` from `@tauri-apps/api/core`; no direct OS API from JS
- [Source: story 2-3 Completion Notes] — 11 tests baseline; two `showStep("step-hotkey")` stubs in `onboarding.js`; `lib.rs` invoke_handler has 4 commands
- [Source: story 2-3 Dev Notes § What NOT to Do] — "Do NOT implement PTT hotkey configuration — that is Story 2.4"
- [Source: story 2-3 Dev Notes § Task 4] — `showStep` function signature; microphone polling pattern (same structure mirrored here for consistency)
- [Source: story 1-3 Dev Notes] — `Settings.ptt_hotkey` field exists; `config::load()` / `config::save()` are the correct functions to call
- [Source: epics.md § Story 3.1] — `hotkey.rs` will parse `ptt_hotkey` from config — store format must be parseable; `event.code` values are well-defined Web standard strings

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

No issues encountered — all tasks implemented cleanly without debug iterations.

### Completion Notes List

- Added `DEFAULT_PTT_HOTKEY: &str = "AltRight"` constant to `config.rs` immediately after `DEFAULT_CONFIDENCE_THRESHOLD`.
- Added `save_ptt_hotkey` Tauri command to `config.rs`: loads current settings via `config::load()`, updates `ptt_hotkey`, saves back via `config::save()`. No new dependencies required.
- Registered `config::save_ptt_hotkey` as the 5th command in the existing `invoke_handler` in `lib.rs`.
- Added `#step-hotkey` div to `onboarding.html` with: heading, two explanation paragraphs, `#hotkey-capture-field` (tabindex=0, dashed border), `#hotkey-display`, `#btn-confirm-hotkey` (initially disabled), `#btn-skip-hotkey`.
- Replaced both `// Story 2.4 will implement: showStep("step-hotkey");` stubs in `onboarding.js` (one in `startMicrophonePolling` on grant, one in `btnSkipMicrophone` click handler).
- Appended hotkey step section to `onboarding.js`: `MODIFIER_CODES` Set, `KEY_DISPLAY_MAP` and `MODIFIER_SYMBOLS` tables, `formatHotkeyCode()` helper, `capturedHotkeyCode`/`isCapturing` state, capture-field click→capture mode, document-level `handleHotkeyKeydown` listener (Escape/Tab cancel; modifier-only keys stored as sided code e.g. "AltRight"; modifier+key combos stored as "Alt+Space"), confirm/skip button handlers invoking `save_ptt_hotkey` with Story 2.5 stubs.
- Added `styles.css` rules for `#hotkey-capture-field` (dashed border, pointer cursor), `.capturing` state (blue border/background), `#hotkey-display` (bold, centered), `#btn-confirm-hotkey` margin, `#btn-skip-hotkey` margin; plus dark mode variants for all three field elements.
- 12/12 tests pass; `cargo clippy --all-targets -- -D warnings` — zero warnings.

### File List

- `src-tauri/src/config.rs` — added `DEFAULT_PTT_HOTKEY` constant, `save_ptt_hotkey` Tauri command, `test_default_ptt_hotkey_constant_value` test
- `src-tauri/src/lib.rs` — added `config::save_ptt_hotkey` to `invoke_handler`
- `src/onboarding.html` — appended `#step-hotkey` step div
- `src/onboarding.js` — replaced two `showStep` stubs; appended hotkey step section
- `src/styles.css` — added hotkey capture field, display, and button rules including dark mode variants

## Change Log

- 2026-04-29: Story 2.4 created — PTT Hotkey Configuration ready for dev.
- 2026-04-30: Implemented Story 2.4 — `DEFAULT_PTT_HOTKEY` constant + `save_ptt_hotkey` Tauri command in `config.rs`; command registered in `lib.rs`; `#step-hotkey` onboarding step added with click-to-capture JS (modifier-only and modifier+key combos, Escape to cancel, `formatHotkeyCode` display map); two Story 2.3 stubs replaced; CSS with `.capturing` state and dark mode. 12/12 tests pass, zero clippy warnings.
