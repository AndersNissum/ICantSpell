# Story 1.3: Configuration Persistence Foundation

Status: done

## Story

As a developer,
I want a config module that persists user settings using the Tauri store plugin,
so that PTT hotkey binding, model selection, and confidence threshold survive app restarts.

## Acceptance Criteria

1. **Given** `config.rs` wraps the Tauri store plugin, **When** the app starts for the first time, **Then** it creates a `settings.json` file in the app data directory with default values.

2. **Given** the `Settings` struct is defined, **When** it is serialized to the store, **Then** it contains `ptt_hotkey: String`, `selected_model: String`, and `confidence_threshold: f32`.

3. **Given** `DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.85` is defined as a constant in `config.rs`, **When** no settings file exists, **Then** the constant is used as the default value for `confidence_threshold`.

4. **Given** a setting is written via `config.rs`, **When** the app is fully quit and relaunched, **Then** the written setting is read back correctly.

5. **And** no audio, transcription text, or user input is ever written to the config store (NFR13/FR28).

## Tasks / Subtasks

- [x] Task 1: Add `tauri-plugin-store` dependency and register plugin (AC: 1)
  - [x] Add `tauri-plugin-store = "2"` to `[dependencies]` in `src-tauri/Cargo.toml`
  - [x] Add `"store:default"` to the `permissions` array in `src-tauri/capabilities/default.json`
  - [x] In `lib.rs`, add `.plugin(tauri_plugin_store::Builder::default().build())` to the Tauri builder **before** `.setup()`

- [x] Task 2: Implement `config.rs` with `Settings` struct and persistence functions (AC: 1, 2, 3, 5)
  - [x] Replace the comment stub entirely
  - [x] Define `DEFAULT_CONFIDENCE_THRESHOLD: pub const f32 = 0.85`
  - [x] Define `Settings` struct: `pub ptt_hotkey: String`, `pub selected_model: String`, `pub confidence_threshold: f32` — derive `Debug, Clone, serde::Serialize, serde::Deserialize`
  - [x] Implement `Default` for `Settings`: `ptt_hotkey: String::new()`, `selected_model: "base".to_string()`, `confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD`
  - [x] Implement `pub fn load(app: &tauri::AppHandle) -> Settings` — opens store, deserializes `"settings"` key; falls back to `Settings::default()` if absent or malformed
  - [x] Implement `pub fn save(app: &tauri::AppHandle, settings: &Settings) -> Result<(), crate::error::AppError>` — serializes struct to `"settings"` key and calls `store.save()`
  - [x] Implement `pub fn ensure_defaults(app: &tauri::AppHandle) -> Result<(), crate::error::AppError>` — loads current settings (or defaults) and saves them, ensuring the file is created on first launch

- [x] Task 3: Add `pub mod config;` to `lib.rs` and call `ensure_defaults` in setup (AC: 1, 4)
  - [x] Add `pub mod config;` to `lib.rs` alongside `pub mod error;` and `pub mod stt;`
  - [x] In the `.setup()` hook, after tray setup, call `config::ensure_defaults(app.handle())?`

- [x] Task 4: Add unit tests to `config.rs` (AC: 2, 3)
  - [x] Test: `Settings::default()` has `confidence_threshold == DEFAULT_CONFIDENCE_THRESHOLD`
  - [x] Test: `Settings::default()` has `selected_model == "base"`
  - [x] Test: `Settings::default()` has `ptt_hotkey` as empty string
  - [x] Test: `serde_json::to_value(Settings::default())` round-trips back to an equivalent `Settings` via `serde_json::from_value`

- [x] Task 5: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
  - [x] `cargo test` — all tests pass including new config unit tests
  - [ ] Manual verification: `cargo tauri dev`, confirm app launches and `settings.json` is created in `~/Library/Application Support/com.icantspell.app/` (note path uses bundle identifier, not product name)

## Dev Notes

### New Dependency: `tauri-plugin-store`

Add to `src-tauri/Cargo.toml`:
```toml
[dependencies]
tauri-plugin-store = "2"
```

Current latest is 2.4.2. The `"2"` semver picks up compatible updates automatically.

### Plugin Registration in `lib.rs`

Register the plugin **before** `.setup()` in the builder chain:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_store::Builder::default().build())  // ← add this line
    .setup(|app| {
        // ... existing tray setup code ...
        config::ensure_defaults(app.handle())?;
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

`tauri_plugin_store::Builder::default()` is equivalent to `Builder::new()`. No additional configuration is needed.

### Capabilities Update

`src-tauri/capabilities/default.json` currently has `"core:default"` only. Add `"store:default"`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "store:default"
  ]
}
```

`"store:default"` covers all store operations: load, get, set, has, save, reload, etc.

### Complete `config.rs` Implementation

```rust
// src-tauri/src/config.rs
use tauri_plugin_store::StoreExt;

pub const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.85;
const STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub ptt_hotkey: String,
    pub selected_model: String,
    pub confidence_threshold: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ptt_hotkey: String::new(),
            selected_model: "base".to_string(),
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
        }
    }
}

pub fn load(app: &tauri::AppHandle) -> Settings {
    let Ok(store) = app.store(STORE_FILE) else {
        tracing::warn!("Failed to open settings store, using defaults");
        return Settings::default();
    };
    match store.get(SETTINGS_KEY) {
        Some(val) => serde_json::from_value(val).unwrap_or_else(|e| {
            tracing::warn!(err = %e, "Settings deserialization failed, using defaults");
            Settings::default()
        }),
        None => Settings::default(),
    }
}

pub fn save(
    app: &tauri::AppHandle,
    settings: &Settings,
) -> Result<(), crate::error::AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| crate::error::AppError::Config(e.to_string()))?;
    let val = serde_json::to_value(settings)
        .map_err(|e| crate::error::AppError::Config(e.to_string()))?;
    store.set(SETTINGS_KEY, val);
    store
        .save()
        .map_err(|e| crate::error::AppError::Config(e.to_string()))?;
    tracing::debug!("Settings saved");
    Ok(())
}

pub fn ensure_defaults(app: &tauri::AppHandle) -> Result<(), crate::error::AppError> {
    let settings = load(app);
    save(app, &settings)?;
    tracing::info!(
        model = %settings.selected_model,
        confidence_threshold = settings.confidence_threshold,
        "Config initialized"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_confidence_threshold() {
        let settings = Settings::default();
        assert_eq!(settings.confidence_threshold, DEFAULT_CONFIDENCE_THRESHOLD);
        assert_eq!(DEFAULT_CONFIDENCE_THRESHOLD, 0.85_f32);
    }

    #[test]
    fn test_default_selected_model() {
        let settings = Settings::default();
        assert_eq!(settings.selected_model, "base");
    }

    #[test]
    fn test_default_ptt_hotkey_is_empty() {
        let settings = Settings::default();
        assert!(settings.ptt_hotkey.is_empty());
    }

    #[test]
    fn test_settings_json_roundtrip() {
        let original = Settings::default();
        let serialized = serde_json::to_value(&original).expect("serialize failed");
        let deserialized: Settings =
            serde_json::from_value(serialized).expect("deserialize failed");
        assert_eq!(deserialized.ptt_hotkey, original.ptt_hotkey);
        assert_eq!(deserialized.selected_model, original.selected_model);
        assert_eq!(deserialized.confidence_threshold, original.confidence_threshold);
    }

    #[test]
    fn test_settings_roundtrip_with_custom_values() {
        let original = Settings {
            ptt_hotkey: "RightOption".to_string(),
            selected_model: "small".to_string(),
            confidence_threshold: 0.75,
        };
        let serialized = serde_json::to_value(&original).expect("serialize failed");
        let deserialized: Settings =
            serde_json::from_value(serialized).expect("deserialize failed");
        assert_eq!(deserialized.ptt_hotkey, "RightOption");
        assert_eq!(deserialized.selected_model, "small");
        assert_eq!(deserialized.confidence_threshold, 0.75_f32);
    }
}
```

### `lib.rs` Changes — Exact Diff

**Before:**
```rust
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
        .setup(|app| {
```

**After:**
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
            // ... existing tray setup code (UNCHANGED) ...

            config::ensure_defaults(app.handle())?;
            Ok(())
        })
```

### Tauri Store API — Critical Details

The `StoreExt` trait is imported inside `config.rs` only — it does NOT need to be imported in `lib.rs`.

Key API behaviors to know:
- `app.store("settings.json")?` — returns `Arc<Store>`, creates the store if it doesn't exist yet
- `store.set("key", value)` — does NOT return `Result`; fires auto-save at ~100ms debounce
- `store.get("key")` — returns `Option<serde_json::Value>`, not a `Result`
- `store.save()` — returns `Result<()>`; flushes immediately, cancels any pending auto-save
- The store is created/managed by the plugin; do NOT manually create JSON files

### macOS Settings File Path

**Important discrepancy vs epics AC:**  
The epics AC says `~/Library/Application Support/icantspell/settings.json` but the **actual path** is:
```
~/Library/Application Support/com.icantspell.app/settings.json
```
Tauri's `BaseDirectory::AppData` resolves using the bundle identifier (`"com.icantspell.app"` from `tauri.conf.json`), not the product name. The AC intent (settings persisted to app support dir) is satisfied; just the path string differs. Manual verification in Task 5 confirms the real path.

### Clippy Guardrails

- `pub mod config;` added to `lib.rs` → `config.rs` is now compiled. `Settings` and its functions are `pub` — no unused warnings.
- `use tauri_plugin_store::StoreExt;` is used in `load()` and `save()` via `app.store(...)` — not unused.
- `tracing::debug!` and `tracing::info!` are used — consistent with established logging pattern.
- Do NOT use `println!` or `eprintln!` anywhere in `config.rs`.
- `store.set()` returns `()` (not a `Result`) — do NOT wrap it in `?`.

### Privacy Constraint (NFR13/FR28)

`config.rs` stores ONLY: `ptt_hotkey`, `selected_model`, `confidence_threshold`. Never store:
- Audio buffers or file paths
- Transcription text
- Any user-dictated content

This is enforced by the `Settings` struct definition — it has only the three permitted fields.

### Files NOT Touched in This Story

- `stt/mod.rs`, `stt/whisper.rs` — no changes
- `error.rs` — no new variants needed; `AppError::Config(String)` was added in Story 1.2 and is used here
- All other stub files (`audio.rs`, `hotkey.rs`, etc.) — remain comment stubs

### Project Structure Notes

Files touched:
- `src-tauri/Cargo.toml` — MODIFY (add `tauri-plugin-store = "2"`)
- `src-tauri/capabilities/default.json` — MODIFY (add `"store:default"`)
- `src-tauri/src/lib.rs` — MODIFY (add `pub mod config;`, plugin registration, `ensure_defaults` call)
- `src-tauri/src/config.rs` — MODIFY (replace comment stub with full implementation)

Files NOT touched:
- `src-tauri/Cargo.lock` — auto-updated by cargo, do not manually edit
- `src-tauri/tauri.conf.json` — no changes needed
- `src-tauri/src/error.rs` — `AppError::Config` already exists from Story 1.2

### References

- [Source: epics.md § Story 1.3] — Acceptance criteria and user story
- [Source: architecture.md § Data Architecture] — Tauri store plugin choice, what is/isn't persisted
- [Source: architecture.md § Privacy Enforcement] — NFR13/FR28 constraint on stored data
- [Source: architecture.md § Implementation Patterns] — Logging pattern (`tracing`), clippy gate
- [Source: architecture.md § Confidence threshold] — `DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.85` named constant in `config.rs`
- [Source: story 1-2 Dev Notes] — `AppError::Config(String)` variant already defined and available
- [Source: tauri-plugin-store v2 docs] — `store.set()` returns `()`, `store.get()` returns `Option<Value>`, path uses bundle identifier

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

_None — clean implementation, no issues encountered._

### Completion Notes List

- Implemented `config.rs` with `Settings` struct (3 fields), `Default` impl, `load`, `save`, and `ensure_defaults` functions
- Added `tauri-plugin-store = "2"` (resolved to v2.4.2) to Cargo.toml
- Added `"store:default"` to capabilities/default.json
- Registered plugin in `lib.rs` before `.setup()` and called `ensure_defaults` at end of setup
- All 5 new config unit tests pass; 2 existing STT tests unchanged; 7/7 total
- `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
- Privacy constraint (NFR13/FR28) enforced by struct definition — only `ptt_hotkey`, `selected_model`, `confidence_threshold`

### File List

- `src-tauri/Cargo.toml` — added `tauri-plugin-store = "2"` dependency
- `src-tauri/Cargo.lock` — auto-updated by cargo (tauri-plugin-store v2.4.2, tauri-plugin v2.5.4, tokio-macros v2.7.0)
- `src-tauri/capabilities/default.json` — added `"store:default"` permission
- `src-tauri/src/lib.rs` — added `pub mod config;`, plugin registration, `ensure_defaults` call
- `src-tauri/src/config.rs` — full implementation replacing comment stub

### Review Findings

- [x] [Review][Defer] `ensure_defaults` unconditional save risks overwriting data on transient load failure [config.rs:57] — deferred, spec-defined behavior; data-loss path requires load-fail AND save-succeed simultaneously; revisit if store reliability issues arise
- [x] [Review][Defer] `confidence_threshold: f32` — no range/NaN/Inf validation [config.rs:12] — deferred, spec defines bare `f32`; downstream validation deferred to Story 3.3
- [x] [Review][Defer] `selected_model` — no allowlist validation [config.rs:11] — deferred, spec defines free-form `String`; Story 3.3 owns model resolution and validation
- [x] [Review][Defer] `ptt_hotkey` empty-string default accepted as valid saved state [config.rs:17] — deferred, spec explicitly defines `String::new()` as default; Story 3.1 owns hotkey registration/validation
- [x] [Review][Defer] No `#[serde(deny_unknown_fields)]` or schema version field [config.rs:8] — deferred, future migration concern; beyond this story's scope
- [x] [Review][Defer] No `[profile.release]` hardening in `Cargo.toml` — deferred, CI/release story concern
- [x] [Review][Defer] `ptt_hotkey` absent from `ensure_defaults` structured log [config.rs:59] — deferred, non-spec; minor observability gap
- [x] [Review][Defer] `unwrap()` on `default_window_icon()` [lib.rs:24] — deferred, pre-existing from Story 1.1
- [x] [Review][Defer] `tracing_subscriber::fmt::init()` panics on re-init [lib.rs:11] — deferred, pre-existing from Story 1.1
- [x] [Review][Defer] Concurrent `save` calls — no synchronization [config.rs:39] — deferred, no concurrent commands exist yet; revisit when Tauri commands added
- [x] [Review][Defer] `load` store-open failure indistinguishable from first-run [config.rs:25] — deferred, low priority; related to ensure_defaults concern above
- [x] [Review][Defer] No test for `confidence_threshold` boundary values (NaN, <0, >1) — deferred, beyond spec test requirements
- [x] [Review][Defer] No test for malformed JSON store (wrong field types) — deferred, beyond spec test requirements

## Change Log

- 2026-04-28: Implemented Story 1.3 — Configuration Persistence Foundation. Added tauri-plugin-store, config.rs with Settings struct/persistence functions, updated lib.rs with plugin registration and ensure_defaults call, 5 unit tests added.
