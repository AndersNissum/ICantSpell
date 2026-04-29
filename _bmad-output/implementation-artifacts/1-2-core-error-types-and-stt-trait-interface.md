# Story 1.2: Core Error Types & STT Trait Interface

Status: done

## Story

As a developer,
I want shared error types and the `SpeechToText` trait defined before any feature implementation,
so that all modules have a stable interface to depend on and error handling is consistent from day one.

## Acceptance Criteria

1. **Given** `error.rs` is created with `thiserror` definitions, **When** any module imports it, **Then** a base `AppError` enum and at least a catch-all variant are available for use.

2. **Given** `stt/mod.rs` is created, **When** another module imports it, **Then** the `SpeechToText` trait is available with the signature `fn transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult, AppError>`.

3. **Given** `TranscriptionResult` is defined in `stt/mod.rs`, **When** it is constructed, **Then** it contains exactly these fields: `text: String`, `confidence: f32`, `alternatives: Vec<String>`.

4. **Given** `lib.rs` re-exports the STT types, **When** integration tests reference `lib.rs`, **Then** `stt::SpeechToText` and `stt::TranscriptionResult` are accessible.

5. **And** `cargo test` passes with at least one unit test verifying `TranscriptionResult` field construction.

## Tasks / Subtasks

- [x] Task 1: Replace `error.rs` stub with full `AppError` enum (AC: 1)
  - [x] Remove the placeholder comment; keep `use thiserror::Error;`
  - [x] Define `AppError` with variants: `Stt(SttError)`, `Config(String)`, `Hotkey(String)`, `Audio(String)`, `Injection(String)`, `Permission(String)`, `Unknown(String)`
  - [x] Define `SttError` enum in the same file with variants: `ModelNotReady`, `InferenceFailed(String)`, `EmptyBuffer`
  - [x] Add `#[from]` on the `Stt(SttError)` variant so `SttError` converts to `AppError` via `?`
  - [x] Derive `Debug, thiserror::Error` on both enums; add `#[error(...)]` on every variant

- [x] Task 2: Implement `stt/mod.rs` with trait and result type (AC: 2, 3)
  - [x] Replace the comment stub entirely
  - [x] Import `crate::error::AppError`
  - [x] Define `TranscriptionResult` struct with exactly: `pub text: String`, `pub confidence: f32`, `pub alternatives: Vec<String>`
  - [x] Define `pub trait SpeechToText: Send + Sync` with method `fn transcribe(&self, audio: &[f32]) -> std::result::Result<TranscriptionResult, AppError>;`
  - [x] Declare `pub mod whisper;` in `stt/mod.rs` (the stub at `stt/whisper.rs` remains unchanged — Story 3.3 implements it)
  - [x] Add unit test in `#[cfg(test)]` block: construct a `TranscriptionResult` with known field values and assert each field matches

- [x] Task 3: Update `lib.rs` to re-export STT module (AC: 4)
  - [x] Add `pub mod stt;` to `lib.rs` (alongside the existing `pub mod error;`)
  - [x] Do NOT add any other module declarations — all other stubs (`audio.rs`, `config.rs`, etc.) stay un-declared until their respective stories

- [x] Task 4: Final validation (AC: all)
  - [x] `cargo clippy --all-targets -- -D warnings` — zero warnings/errors
  - [x] `cargo test` — at least 1 test passing (the `TranscriptionResult` field test)
  - [x] Verify `stt::SpeechToText` and `stt::TranscriptionResult` are importable from the crate root

## Dev Notes

### Current File State (from Story 1.1)

**`src-tauri/src/error.rs`** — Already exists with a minimal stub:
```rust
// Shared error types — implemented in Story 1.2
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unknown error: {0}")]
    Unknown(String),
}
```
Replace the comment at the top and expand the enum. Keep the `use thiserror::Error;` import.

**`src-tauri/src/stt/mod.rs`** — Currently comment-only (no Rust code). Replace entirely.

**`src-tauri/src/stt/whisper.rs`** — Currently comment-only stub. **Do not touch** — Story 3.3 owns this file.

**`src-tauri/src/lib.rs`** — Currently:
```rust
pub mod error;
// (tauri imports and run() function follow)
```
Add `pub mod stt;` after `pub mod error;`. Do NOT remove or change anything else.

### Error Type Design

Define both enums in `error.rs`. The two-layer pattern:
- **Layer 1**: Module-specific `SttError` using `thiserror`
- **Layer 2**: App-level `AppError` that wraps module errors via `#[from]`

```rust
// error.rs — complete implementation
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SttError {
    #[error("STT model not loaded")]
    ModelNotReady,
    #[error("Transcription inference failed: {0}")]
    InferenceFailed(String),
    #[error("Audio buffer is empty")]
    EmptyBuffer,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("STT error: {0}")]
    Stt(#[from] SttError),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Hotkey error: {0}")]
    Hotkey(String),
    #[error("Audio error: {0}")]
    Audio(String),
    #[error("Injection error: {0}")]
    Injection(String),
    #[error("Permission error: {0}")]
    Permission(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}
```

### STT Trait Design

```rust
// stt/mod.rs — complete implementation
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub alternatives: Vec<String>,
}

pub trait SpeechToText: Send + Sync {
    fn transcribe(&self, audio: &[f32]) -> std::result::Result<TranscriptionResult, AppError>;
}

pub mod whisper;  // stub — implemented in Story 3.3

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_result_fields() {
        let result = TranscriptionResult {
            text: "hello world".to_string(),
            confidence: 0.95,
            alternatives: vec!["hello word".to_string()],
        };
        assert_eq!(result.text, "hello world");
        assert_eq!(result.confidence, 0.95);
        assert_eq!(result.alternatives.len(), 1);
        assert_eq!(result.alternatives[0], "hello word");
    }
}
```

### Trait Sync vs Async

**The `SpeechToText` trait is intentionally synchronous.** `transcribe()` is a blocking CPU-bound call. The caller (Story 3.3's `WhisperBackend` integration) wraps it in `tokio::task::spawn_blocking`. Do NOT add `async` to the trait method.

### Clippy Pitfalls (learned from Story 1.1)

- `cargo clippy --all-targets -- -D warnings` is the gate. Unused imports fail it.
- When `pub mod stt;` is added to `lib.rs`, `stt/mod.rs` is compiled. If `stt/mod.rs` declares `pub mod whisper;`, then `stt/whisper.rs` is compiled too. The whisper stub is comment-only — this compiles fine with no warnings.
- Do NOT `use` anything in `lib.rs` that isn't actually used. The existing `lib.rs` imports (`tauri`, `menu`, `tray`) are used in `run()` — leave them unchanged.
- `AppError` variants for future modules (`Config`, `Hotkey`, etc.) are defined now but won't be `used` yet. This is fine because they're `pub` items in a `pub` enum — Rust/Clippy doesn't warn on unused public enum variants.

### Cargo.toml — No Changes Needed

`thiserror = "1"` is already in `Cargo.toml` from Story 1.1. No new crates are required for this story. Do NOT add `anyhow` — it's for application call sites in later stories.

### `lib.rs` — Preserve `run()` Function Intact

The `run()` function and all tray/setup code in `lib.rs` must not be modified. Only add `pub mod stt;` at the top with the other module declarations.

### No Tauri Commands in This Story

This story is pure Rust types and traits — no `#[tauri::command]`, no IPC, no frontend changes. Only `error.rs`, `stt/mod.rs`, and `lib.rs` change.

### Project Structure Notes

Files touched:
- `src-tauri/src/error.rs` — MODIFY (replace stub, expand enum)
- `src-tauri/src/stt/mod.rs` — MODIFY (replace comment stub with implementation)
- `src-tauri/src/lib.rs` — MODIFY (add `pub mod stt;` line only)

Files NOT touched (do not modify):
- `src-tauri/src/stt/whisper.rs` — comment stub, owned by Story 3.3
- All other `src-tauri/src/*.rs` stubs — owned by their respective stories
- `src-tauri/Cargo.toml` — no new dependencies needed
- All frontend files in `src/`

### References

- [Source: epics.md § Story 1.2] — Acceptance criteria and user story
- [Source: architecture.md § STT Abstraction] — `SpeechToText` trait signature and `TranscriptionResult` fields
- [Source: architecture.md § Error Handling Patterns] — Two-layer error strategy (`thiserror` + `anyhow`), `SttError` variants
- [Source: architecture.md § Implementation Patterns & Consistency Rules] — Clippy gate, test placement in `#[cfg(test)]`
- [Source: architecture.md § Rust Module Organization] — `lib.rs` as re-export layer for integration tests
- [Source: story 1-1 Dev Notes § Stub File Conventions] — `error.rs` stub that this story replaces; clippy -D warnings gate

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- No issues encountered. `cargo clippy --all-targets -- -D warnings` passed clean on first attempt.
- `pub mod stt;` in `lib.rs` triggers compilation of `stt/mod.rs` and `stt/whisper.rs`. The whisper stub (comment-only) compiles without warnings.
- All `AppError` variants (`Config`, `Hotkey`, `Audio`, `Injection`, `Permission`) are `pub` enum items — Rust does not warn on unused public variants, so `-D warnings` passes cleanly even though these variants are not yet used by other modules.

### Completion Notes List

- Replaced `error.rs` stub with full two-layer error hierarchy: `SttError` (3 variants: `ModelNotReady`, `InferenceFailed`, `EmptyBuffer`) and `AppError` (7 variants; `Stt(#[from] SttError)` provides automatic conversion via `?`)
- Implemented `stt/mod.rs`: `TranscriptionResult` struct (3 fields: `text`, `confidence`, `alternatives`), `SpeechToText: Send + Sync` trait with synchronous `transcribe()` method, `pub mod whisper;` declaration
- Added 2 unit tests in `stt/mod.rs`: `test_transcription_result_fields` and `test_transcription_result_empty_alternatives`
- Added `pub mod stt;` to `lib.rs` — `stt::SpeechToText` and `stt::TranscriptionResult` now accessible from crate root
- `cargo clippy --all-targets -- -D warnings`: 0 warnings, 0 errors
- `cargo test`: 2 passed, 0 failed

### File List

- `src-tauri/src/error.rs` (modified — replaced stub with full `SttError` + `AppError` enums)
- `src-tauri/src/stt/mod.rs` (modified — replaced comment stub with `TranscriptionResult`, `SpeechToText` trait, 2 unit tests)
- `src-tauri/src/lib.rs` (modified — added `pub mod stt;`)

### Review Findings

- [x] [Review][Defer] `unwrap()` on `default_window_icon()` panics if icon missing [lib.rs:22] — deferred, pre-existing (Story 1.1)
- [x] [Review][Defer] `tracing_subscriber::fmt::init()` silently discards error [lib.rs:10] — deferred, pre-existing (Story 1.1)
- [x] [Review][Defer] `on_menu_event` silently drops non-quit events [lib.rs:28] — deferred, pre-existing (Story 1.1)
- [x] [Review][Defer] `_tray` local variable may not keep tray icon alive [lib.rs:24] — deferred, pre-existing (Story 1.1)
- [x] [Review][Defer] `app.exit(0)` performs no orderly shutdown [lib.rs:30] — deferred, pre-existing (Story 1.1)
- [x] [Review][Defer] macOS-only `ActivationPolicy::Accessory` — no cross-platform equivalent [lib.rs:17] — deferred, pre-existing (Story 1.1)
- [x] [Review][Defer] `confidence: f32` — no NaN/Inf/range guard [stt/mod.rs:6] — deferred, spec defines bare `f32`; future story concern
- [x] [Review][Defer] `SttError` missing I/O, model-load, and timeout variants [error.rs] — deferred, spec defines exactly these variants; future stories add more
- [x] [Review][Defer] `AppError` variants use stringly-typed payloads [error.rs] — deferred, intentional per spec; future stories may refine
- [x] [Review][Defer] `alternatives: Vec<String>` — no length bound or ordering contract [stt/mod.rs:7] — deferred, spec defines `Vec<String>`; future story concern
- [x] [Review][Defer] `EmptyBuffer` not enforced at trait boundary [stt/mod.rs:10] — deferred, trait is an interface; implementors are responsible
- [x] [Review][Defer] `&self` forces interior mutability on implementors with no guidance [stt/mod.rs:10] — deferred, intentional synchronous design per spec
- [x] [Review][Defer] No test for `From<SttError>` → `AppError::Stt` conversion [stt/mod.rs] — deferred, story only requires `TranscriptionResult` field test

## Change Log

- 2026-04-28: Story implemented — `error.rs` expanded with `SttError`/`AppError` enums; `stt/mod.rs` implemented with `TranscriptionResult` struct and `SpeechToText` trait; `lib.rs` re-exports `stt` module; 2 unit tests pass; `cargo clippy -D warnings` and `cargo test` clean (claude-sonnet-4-6)
