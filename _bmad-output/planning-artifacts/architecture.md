---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
lastStep: 8
status: 'complete'
completedAt: '2026-04-27'
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
workflowType: 'architecture'
project_name: 'ICantSpell'
user_name: 'Anders'
date: '2026-04-27'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**

31 v1 FRs organized into eight areas: Voice Input (FR1–4), Speech Transcription (FR5–7), Text Injection (FR8–10), Transcription Error Handling (FR11–16), App Presence & Controls (FR17–19), Onboarding & Permissions (FR20–25), Privacy & Data (FR26–28), and Model Management (FR29–31). Nine additional v2 FRs cover the typing mode pipeline (FR32–40).

Architecturally, the FRs define two fully independent feature pipelines that share only the AXUIElement injection layer and the menu bar shell. This clean seam is intentional and must be preserved in implementation.

**Non-Functional Requirements:**

| NFR | Architectural Driver |
|---|---|
| PTT → inject latency < 2s (NFR1) | Whisper inference must run off the main thread; async pipeline required |
| Whisper loads async (NFR2) | Model loader runs independently of app startup; menu bar appears before model is ready |
| Correction overlay < 200ms (NFR3) | Overlay window must be pre-created and hidden, not spawned on demand |
| Memory < 300MB idle (NFR9) | Only one Whisper model in memory at a time; model swap = unload + reload |
| CPU < 2% idle (NFR10) | No background polling; all activation is event-driven |
| Zero network calls (NFR12) | Network stack must be disabled at build level or verified by network policy |
| No data written to disk (NFR13) | Audio buffers and transcription text are in-memory only; cleared after injection |
| Sleep/wake survival (NFR7) | PTT hotkey and CGEventTap registrations must be restored on wake notification |

**Scale & Complexity:**

- Primary domain: macOS system-level Rust + local ML inference + floating overlay UI
- Complexity level: Medium (v1), High (v2)
- Estimated architectural components: ~8 core modules (see below)

### Technical Constraints & Dependencies

- **Tauri** — Rust backend + webview UI; system tray API for menu bar; no Dock icon via `LSUIElement`
- **whisper-rs** — Rust bindings to Whisper.cpp; core v1 dependency; must be spiked early
- **ONNX Runtime** — DeBERTa-v3-small inference for v2 typing mode
- **AXUIElement** — macOS Accessibility API for active text field identification and text injection (v1 + v2)
- **CGEventTap** — system-wide keystroke monitoring for pause detection (v2 only)
- **No App Store / no sandboxing** — required OS-level APIs are sandbox-incompatible
- **Apple Silicon M-series primary** — Intel Mac not a v1 target; no cross-architecture testing burden
- **Unsigned binary** — Gatekeeper override required on first run; no notarization for v1

### Cross-Cutting Concerns Identified

1. **Permission lifecycle** — Microphone, Accessibility, and (v2) Input Monitoring grants must be checked at startup, monitored for revocation, and surfaced gracefully without crashing (FR24, FR25, NFR5, NFR6)
2. **Async model loading** — Whisper model load must not block menu bar appearance or PTT registration; all downstream consumers must handle a "model not ready" state
3. **Global event handling** — PTT hotkey and (v2) CGEventTap are process-wide, must survive app focus changes and sleep/wake cycles
4. **Privacy enforcement** — Zero network calls must be structurally enforced, not just policy; no HTTP client in the dependency tree
5. **Error recovery without crash** — Transcription failures and injection failures must surface non-intrusively and leave the target app's content intact
6. **STT abstraction boundary** — `whisper-rs` must sit behind a trait/interface to support model swapping (Journey 4) and future STT engine pluggability

## Starter Template Evaluation

### Primary Technology Domain

macOS desktop app — Tauri v2 (Rust backend + webview UI). Stack committed by PRD; this step confirms scaffolding approach.

### Starter Options Considered

| Template | Verdict |
|---|---|
| `vanilla` / `vanilla-ts` | ✅ Selected — minimal HTML/CSS/JS frontend, Rust backend does all real work |
| `leptos` / `yew` | ❌ Deferred — pure Rust UI is appealing but adds complexity for a tiny UI surface |
| `react` / `svelte` | ❌ Overkill — brings full build pipelines for what amounts to a few overlays |

### Selected Starter: `cargo create-tauri-app`

**Rationale:** The app's UI surface is small (menu bar popover, correction overlay, onboarding wizard). A vanilla webview keeps frontend complexity near zero while the Rust backend owns all ML inference, system API calls, and business logic. No JS framework lock-in to manage.

**Reference starter for system tray config:** `ahkohd/tauri-macos-menubar-app-example` — menu bar only, no Dock icon, exact use case.

**Initialization Command:**

```bash
cargo install create-tauri-app --locked
cargo create-tauri-app icantspell --template vanilla
```

**Tauri Version:** v2.10.3 (stable)

**Architectural Decisions Provided by Starter:**

**Language & Runtime:**
Rust (backend, all business logic) + HTML/CSS/JS (frontend, UI only). TypeScript optional via `vanilla-ts` variant — can be added if overlay UI complexity warrants it.

**Build Tooling:**
Cargo (Rust) + Tauri CLI. No webpack/vite unless frontend complexity grows.

**Project Structure:**
- `src/` — Webview frontend (HTML/CSS/JS overlays, onboarding wizard)
- `src-tauri/src/` — All Rust: PTT handler, audio capture, Whisper pipeline, AXUIElement injection, system tray, model management
- `src-tauri/tauri.conf.json` — System tray config, LSUIElement (no Dock icon), capabilities/permissions

**Testing Framework:**
Rust: `cargo test` (unit + integration). Frontend: none required at this scale.

**Code Organization:**
Rust modules in `src-tauri/src/` will be the primary architectural concern. Frontend is treated as a thin rendering layer.

**Development Experience:**
`cargo tauri dev` — hot-reload webview + Rust rebuild on change.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- STT trait abstraction (required before any Whisper integration work)
- Correction overlay window strategy (required before error handling UI)
- Tauri IPC pattern (required before any frontend-backend work)

**Important Decisions (Shape Architecture):**
- Config persistence approach
- Audio capture library
- CI/CD scope

**Deferred Decisions (Post-MVP):**
- ONNX Runtime integration details (v2)
- CGEventTap pause detection tuning (v2)
- Code signing / notarization (only if distributed to others)

### Data Architecture

**Config Persistence: Tauri `store` plugin**
- Storage: JSON file in `~/Library/Application Support/icantspell/`
- Persisted: PTT hotkey binding, selected Whisper model, (v2) pause threshold
- Not persisted: audio buffers, transcription text, user input (FR28/NFR13)
- Rationale: Idiomatic Tauri v2 approach; zero custom serialization code

**Model File Storage:**
- Bundled default: `base` model (~150MB) included in app bundle
- Additional models: downloaded to `~/Library/Application Support/icantspell/models/` on demand
- One model in memory at a time; swap = unload + reload

### Authentication & Security

N/A — no user authentication. Security posture is entirely OS-level permission grants.

**Permission Lifecycle:**
- Check on startup: Microphone + Accessibility (v1); + Input Monitoring (v2)
- Monitor for revocation: Surface non-intrusive indicator if permission is revoked post-setup (FR25)
- No crash path: All permission failures handled gracefully (NFR5, NFR6)

**Privacy Enforcement:**
- Zero network calls: enforced by absence of any HTTP client in the dependency tree
- Verifiable under Little Snitch (NFR12)
- No `reqwest`, `hyper`, or similar crates anywhere in the build

### API & Communication Patterns

**Internal IPC: Tauri commands + events**
- Frontend → Backend: `#[tauri::command]` (e.g. user taps correction overlay option)
- Backend → Frontend: `app.emit()` events (e.g. "transcription_ready", "show_overlay", "hide_overlay")
- No WebSocket, no REST API — this is a local single-webview app

**Key IPC events (planned):**

| Direction | Event/Command | Payload |
|---|---|---|
| BE → FE | `transcription_ready` | `{ text, confidence, position }` |
| BE → FE | `show_correction_overlay` | `{ word, alternatives, screen_position }` |
| BE → FE | `hide_correction_overlay` | — |
| BE → FE | `permission_revoked` | `{ permission_name }` |
| FE → BE | `accept_correction` | `{ selected_text }` |
| FE → BE | `redictate` | — |
| FE → BE | `dismiss_overlay` | — |

### Frontend Architecture

**Approach: Vanilla HTML/CSS/JS — thin rendering layer**
- No state management framework; minimal JS to handle IPC events and render overlay/onboarding UI
- Pre-created hidden `WebviewWindow` for correction overlay (satisfies NFR3 < 200ms appearance)
- Menu bar popover: separate Tauri window shown/hidden from system tray click
- Onboarding wizard: shown as primary window on first launch only

### Audio Pipeline Architecture

**Audio Capture: `cpal`**
- Active only during PTT hold (FR4) — no background audio capture
- Captures mic input to in-memory `Vec<f32>` buffer
- On PTT release: buffer passed to STT pipeline; buffer dropped after injection (NFR13)
- Rationale: Battle-tested, good macOS CoreAudio support, preserves cross-platform path

### STT Abstraction

**Pattern: Rust `trait SpeechToText`**

```rust
trait SpeechToText: Send + Sync {
    fn transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult>;
}

struct TranscriptionResult {
    text: String,
    confidence: f32,
    alternatives: Vec<String>,
}
```

- `WhisperBackend` is the sole v1 implementation (wraps `whisper-rs`)
- Model loading is async — `WhisperBackend::load()` runs on a background thread; PTT activations queue until ready
- Swap a model = construct a new `WhisperBackend` with different model path

### Overlay Window Architecture

**Strategy: Pre-created always-on-top Tauri WebviewWindow**
- Window created at app startup, hidden (`visible: false`)
- `always_on_top: true`, borderless, no title bar
- Positioned programmatically near the amber-underlined word's screen coordinates (from AXUIElement)
- Shown via `window.show()` on transcription event; hidden via `window.hide()` on dismiss
- Rationale: Pre-creation satisfies NFR3 (< 200ms); keeps overlay in same rendering context as rest of UI

### Infrastructure & Deployment

**CI/CD: GitHub Actions — lint + test gate**
- Triggers: push to `main`, pull requests
- Jobs: `cargo clippy --all-targets -- -D warnings`, `cargo test`
- No release pipeline for v1 (personal use; manual `cargo tauri build`)
- Code signing: none for v1 (unsigned binary; Gatekeeper one-time override on developer's own machine)

### Decision Impact Analysis

**Implementation Sequence (order matters):**
1. `SpeechToText` trait + `TranscriptionResult` type — everything else depends on this interface
2. `WhisperBackend` impl + async model loader — core v1 dependency; spike early (PRD risk table)
3. PTT hotkey registration + `cpal` audio capture pipeline
4. AXUIElement text injection module
5. Tauri IPC event schema + command handlers
6. Correction overlay window + amber underline indicator
7. Config persistence (Tauri store plugin)
8. Onboarding wizard + permission request flow
9. Menu bar controls (enable/disable, model selection, settings)
10. CI: clippy + test GitHub Actions workflow

**Cross-Component Dependencies:**
- Overlay positioning depends on AXUIElement returning word screen coordinates
- Correction overlay IPC depends on `TranscriptionResult.confidence` + `alternatives`
- Model management (FR29–31) depends on `WhisperBackend` accepting a model path at construction
- All v2 features (CGEventTap, DeBERTa/ONNX) are additive — no v1 components need modification

## Implementation Patterns & Consistency Rules

### Critical Conflict Points Identified

6 areas where AI agents could make inconsistent choices: IPC event naming, error handling strategy, async/CPU-bound patterns, logging, test location, and Tauri command return types.

### Naming Patterns

**IPC Event Naming: `snake_case` everywhere**
- Both Rust (emit side) and JavaScript (listen side) use `snake_case`
- One rule, no translation layer at the boundary
- Examples: `transcription_ready`, `show_correction_overlay`, `hide_correction_overlay`, `permission_revoked`
- Anti-pattern: ❌ `transcriptionReady`, ❌ `transcription-ready`

**Rust Code Naming: Standard Rust conventions**
- Structs/Enums/Traits: `PascalCase` — `WhisperBackend`, `SpeechToText`, `TranscriptionResult`, `AppError`
- Functions/methods/variables: `snake_case` — `transcribe()`, `audio_buffer`, `model_path`
- Modules/files: `snake_case` — `stt/mod.rs`, `injection.rs`, `hotkey.rs`
- Constants: `SCREAMING_SNAKE_CASE` — `DEFAULT_MODEL_NAME`, `MAX_AUDIO_BUFFER_SECS`

**IPC Payload Fields: `snake_case` in Rust structs, `camelCase` in JS**
- Tauri automatically serializes Rust `snake_case` struct fields to `camelCase` JSON for the frontend
- Rust side: `screen_position`, `confidence_score`
- JS side (after deserialization): `screenPosition`, `confidenceScore`
- Do NOT manually rename fields — rely on Tauri's serde rename behavior

### Structure Patterns

**Rust Module Organization (`src-tauri/src/`):**
```
src-tauri/src/
├── main.rs              # App entry point, Tauri builder setup, window creation
├── lib.rs               # Re-exports for integration tests
├── stt/
│   ├── mod.rs           # SpeechToText trait + TranscriptionResult type
│   └── whisper.rs       # WhisperBackend impl
├── audio.rs             # cpal capture pipeline, PTT buffer management
├── hotkey.rs            # Global PTT hotkey registration + event loop
├── injection.rs         # AXUIElement text injection
├── overlay.rs           # Correction overlay window management (show/hide/position)
├── permissions.rs       # macOS permission checking + revocation monitoring
├── models.rs            # Whisper model discovery, download, selection
├── config.rs            # Tauri store plugin wrapper, settings types
└── error.rs             # Shared AppError type, thiserror definitions
```

**Frontend Organization (`src/`):**
```
src/
├── index.html           # Main app shell (menu bar popover)
├── overlay.html         # Correction overlay window
├── onboarding.html      # Onboarding wizard
├── main.js              # Menu bar popover logic + IPC listeners
├── overlay.js           # Correction overlay logic + IPC listeners
├── onboarding.js        # Onboarding wizard step logic
└── styles.css           # Shared styles
```

### Format Patterns

**Tauri Command Return Type: `Result<T, String>`**
```rust
// ✅ Correct pattern at Tauri command boundary
#[tauri::command]
async fn select_model(model_name: String) -> Result<(), String> {
    do_something().await.map_err(|e| e.to_string())
}

// ❌ Never panic or unwrap in command handlers
#[tauri::command]
fn bad_command() -> String {
    do_something().unwrap() // DO NOT DO THIS
}
```

Typed errors (`thiserror` enums) live inside modules. Only `String` crosses the Tauri command boundary.

**IPC Event Payload: Rust structs with `serde::Serialize`**
```rust
// ✅ Emit with a typed struct, never ad-hoc values
#[derive(serde::Serialize, Clone)]
struct TranscriptionReadyPayload {
    text: String,
    confidence: f32,
    alternatives: Vec<String>,
    screen_position: (f64, f64),
}
app.emit("transcription_ready", payload)?;
```

### Error Handling Patterns

**Two-layer error strategy:**

**Layer 1 — Module errors (`thiserror`):**
```rust
// In error.rs or within each module
#[derive(Debug, thiserror::Error)]
pub enum SttError {
    #[error("Model not loaded")]
    ModelNotReady,
    #[error("Transcription failed: {0}")]
    InferenceFailed(String),
    #[error("Audio buffer empty")]
    EmptyBuffer,
}
```

**Layer 2 — Application boundary (`anyhow`):**
```rust
// In non-command application code that calls multiple modules
use anyhow::{Context, Result};

async fn run_voice_pipeline(audio: Vec<f32>) -> Result<()> {
    let result = backend.transcribe(&audio)
        .context("STT transcription failed")?;
    inject_text(&result.text)
        .context("AXUIElement injection failed")?;
    Ok(())
}
```

**Never:**
- `unwrap()` or `expect()` outside of tests
- Silently swallow errors — always propagate or log
- Panic in response to recoverable errors (NFR5, NFR6)

### Async Patterns

**CPU-bound work (Whisper inference): always `spawn_blocking`**
```rust
// ✅ Correct — Whisper inference is CPU-bound, must not block async runtime
let result = tokio::task::spawn_blocking(move || {
    backend.transcribe(&audio_buffer)
}).await??;

// ❌ Wrong — blocks the Tokio thread pool
let result = backend.transcribe(&audio_buffer).await;
```

**All Tauri commands are `async fn`** — even if the implementation is synchronous, for consistency and future-proofing.

**Runtime:** Tauri v2 uses Tokio. Do not introduce `async-std` or any other async runtime.

### Logging Patterns

**Crate: `tracing` + `tracing-subscriber`**

```rust
use tracing::{info, warn, error, debug};

// ✅ Structured log with fields
info!(model = %model_name, confidence = confidence, "Transcription complete");
warn!(app = "injection", "AXUIElement target not found — skipping");
error!(err = %e, "Whisper inference failed");

// ❌ Avoid plain println! in library code
println!("Transcription done"); // DO NOT DO THIS
```

**Log levels:**
- `error` — failures that surface to the user (injection fail, model load fail)
- `warn` — recoverable issues (low confidence, permission borderline)
- `info` — pipeline milestones (PTT activated, transcription complete, text injected)
- `debug` — internal state (audio buffer size, model path, overlay position)

### Test Patterns

**Unit tests: inline `#[cfg(test)]` in each module file**
```rust
// At the bottom of injection.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injection_skips_empty_text() {
        // ...
    }
}
```

**Integration tests: `src-tauri/tests/` directory**
```
src-tauri/tests/
├── stt_pipeline.rs      # Full STT pipeline integration tests
└── config_roundtrip.rs  # Config persistence read/write tests
```

Integration tests in `tests/` use the public API via `lib.rs` re-exports.

### Enforcement Guidelines

**All AI agents MUST:**
- Use `snake_case` for all IPC event names (both emit and listen sides)
- Use `Result<T, String>` (never panic) in all `#[tauri::command]` functions
- Use `tokio::task::spawn_blocking` for all Whisper inference calls
- Use `tracing` macros — never `println!` in non-test code
- Define module-level errors with `thiserror`; use `anyhow` only at application call sites
- Place unit tests inline with `#[cfg(test)]`; integration tests in `src-tauri/tests/`
- Never add HTTP client crates (`reqwest`, `hyper`, `ureq`, etc.) to `Cargo.toml`

**Anti-Patterns to Reject in Code Review:**
- `unwrap()` / `expect()` outside `#[cfg(test)]` blocks
- `println!` / `eprintln!` in library modules
- Blocking calls inside `async fn` without `spawn_blocking`
- IPC event names in `camelCase` or `kebab-case`
- Ad-hoc `serde_json::json!({})` payloads in `emit()` calls — always use typed structs

## Project Structure & Boundaries

### Complete Project Directory Structure

```
icantspell/
├── README.md
├── .gitignore
├── .github/
│   └── workflows/
│       └── ci.yml                    # cargo clippy + cargo test on push/PR
│
├── src/                              # Webview frontend (thin rendering layer)
│   ├── index.html                    # Menu bar popover UI
│   ├── overlay.html                  # Correction overlay window
│   ├── onboarding.html               # First-launch onboarding wizard
│   ├── main.js                       # Popover IPC listeners + menu bar controls
│   ├── overlay.js                    # Correction overlay IPC + tap interactions
│   ├── onboarding.js                 # Onboarding step logic + permission prompts
│   └── styles.css                    # Shared styles (all windows)
│
└── src-tauri/
    ├── Cargo.toml                    # Rust dependencies
    ├── Cargo.lock
    ├── build.rs                      # Tauri build script
    ├── tauri.conf.json               # App config: system tray, LSUIElement, windows
    ├── capabilities/
    │   └── default.json              # Tauri v2 capability definitions
    ├── icons/                        # App icons (menu bar icon variants)
    │   ├── icon.png
    │   ├── icon@2x.png
    │   └── icon_template.png         # macOS menu bar template image (dark/light)
    ├── models/                       # Bundled default Whisper model
    │   └── ggml-base.bin             # ~150MB, bundled at build time
    └── src/
        ├── main.rs                   # App entry point, Tauri builder, window setup
        ├── lib.rs                    # Re-exports for integration tests
        ├── error.rs                  # Shared error types (thiserror enums)
        ├── config.rs                 # Tauri store plugin wrapper, Settings struct
        ├── permissions.rs            # macOS permission checking + revocation monitor
        ├── hotkey.rs                 # Global PTT hotkey registration (FR1-3, NFR8)
        ├── audio.rs                  # cpal mic capture pipeline, PTT buffer (FR4, NFR13)
        ├── injection.rs              # AXUIElement text injection (FR8-10, NFR6)
        ├── overlay.rs                # Correction overlay window: show/hide/position (FR11-16, NFR3)
        ├── models.rs                 # Whisper model discovery, selection, download (FR29-31)
        ├── stt/
        │   ├── mod.rs                # SpeechToText trait + TranscriptionResult type
        │   └── whisper.rs            # WhisperBackend impl wrapping whisper-rs (FR5-7)
        └── tests/                    # Integration tests
            ├── stt_pipeline.rs       # Full STT pipeline: audio → transcription → result
            └── config_roundtrip.rs   # Config persistence: write → reload → verify
```

### Architectural Boundaries

**Tauri IPC Boundary (Rust ↔ Frontend):**
- All cross-boundary communication goes through Tauri commands and events
- Rust emits typed structs via `app.emit()`; JS listens via `listen()`
- Frontend never calls OS APIs directly — always via `invoke()` to a `#[tauri::command]`
- No shared state between windows — each window communicates through the Rust backend

**STT Abstraction Boundary:**
- Callers (`audio.rs` pipeline, `overlay.rs` re-dictate) depend only on `stt::SpeechToText` trait
- `stt::whisper::WhisperBackend` is the only implementation; never imported directly outside `stt/`
- Model path is passed at `WhisperBackend` construction; no global model state

**System API Boundary:**
- `injection.rs` is the only module that calls AXUIElement — all text injection goes through it
- `hotkey.rs` is the only module that registers global event taps — no other module touches the event loop
- `permissions.rs` is the only module that queries `AXIsProcessTrusted()`, `AVCaptureDevice`, etc.

**Privacy Boundary:**
- `Cargo.toml` must never include: `reqwest`, `hyper`, `ureq`, `surf`, or any general-purpose HTTP client crate
- Enforced by CI: `cargo deny` (or manual audit) on dependency tree
- **Model download exception (FR30):** In-app Whisper model downloads are implemented via macOS `URLSession` through the existing `objc2` FFI dependency — no new HTTP crate required. This is the sole permitted outbound network operation; it is always user-initiated and scoped exclusively to `models.rs`. All dictation, transcription, and injection usage paths remain zero-network (NFR12).

### Requirements to Structure Mapping

| FR Category | FRs | Primary Files |
|---|---|---|
| Voice Input | FR1–4 | `hotkey.rs`, `audio.rs` |
| Speech Transcription | FR5–7 | `stt/whisper.rs`, `stt/mod.rs` |
| Text Injection | FR8–10 | `injection.rs` |
| Error Handling / Overlay | FR11–16 | `overlay.rs`, `src/overlay.html`, `src/overlay.js` |
| App Presence & Controls | FR17–19 | `main.rs` (tray setup), `src/index.html`, `src/main.js` |
| Onboarding & Permissions | FR20–25 | `permissions.rs`, `src/onboarding.html`, `src/onboarding.js` |
| Privacy & Data | FR26–28 | `audio.rs` (no persistence), `Cargo.toml` (no HTTP deps) |
| Model Management | FR29–31 | `models.rs`, `src-tauri/models/` |
| v2: Typing Mode | FR32–40 | Future: `keymonitor.rs`, `stt/deberta.rs` (additive, no v1 changes) |

**Cross-Cutting Concerns:**

| Concern | Location |
|---|---|
| Shared error types | `error.rs` |
| User settings persistence | `config.rs` (Tauri store plugin) |
| App startup orchestration | `main.rs` |
| IPC event definitions | Documented in architecture; types in each owning module |
| Sleep/wake recovery (NFR7) | `hotkey.rs` + `permissions.rs` (re-register on wake notification) |

### Integration Points & Data Flow

**Primary v1 Data Flow (Voice Mode):**
```
User holds PTT hotkey
  → hotkey.rs detects keydown, signals audio.rs
  → audio.rs starts cpal capture into Vec<f32> buffer
  → User releases PTT
  → hotkey.rs signals audio.rs to stop capture
  → audio.rs passes buffer to stt::WhisperBackend via spawn_blocking
  → TranscriptionResult { text, confidence, alternatives } returned
  → If confidence ≥ threshold: injection.rs injects text via AXUIElement
  → If confidence < threshold: overlay.rs shows correction overlay with alternatives
  → User taps alternative / re-dictates / dismisses
  → Audio buffer dropped from memory (NFR13)
```

**App Startup Flow:**
```
main.rs builds Tauri app
  → permissions.rs checks Accessibility + Microphone grants
  → models.rs discovers bundled model, loads WhisperBackend async (spawn_blocking)
  → hotkey.rs registers global PTT hotkey
  → overlay.rs pre-creates hidden correction overlay window
  → System tray icon appears (NFR2: before model load completes)
  → config.rs loads stored settings (hotkey binding, selected model)
```

**Configuration Flow:**
```
User changes setting (model selection, hotkey) via menu bar popover
  → JS invokes Tauri command in config.rs
  → config.rs writes to Tauri store (JSON in ~/Library/Application Support/icantspell/)
  → Dependent modules notified via app event if live reload needed
```

### Tauri Window Configuration

Three windows defined at startup:

| Window Label | Visible at Start | Purpose |
|---|---|---|
| `main` | false (shown on tray click) | Menu bar popover — settings, status |
| `overlay` | false (shown on low-confidence transcription) | Correction overlay — always-on-top, borderless |
| `onboarding` | false (shown on first launch only) | Permission grants + hotkey config |

Key `tauri.conf.json` flags:
- `"activationPolicy": "accessory"` — no Dock icon (LSUIElement equivalent)
- `"systemTray": true` — menu bar icon
- Overlay window: `"alwaysOnTop": true`, `"decorations": false`, `"transparent": true`

## Architecture Validation Results

### Coherence Validation

**Decision Compatibility:** ✅
All technology choices are compatible: Tauri v2 + whisper-rs (both Rust), cpal + CoreAudio (first-class macOS support), thiserror + anyhow (standard Rust pairing), tracing (shared with Tauri internals), Tauri store plugin (native Tauri v2 plugin). No version conflicts.

**Pattern Consistency:** ✅
snake_case IPC events, Result<T,String> command boundary, and tracing logging rules are consistent across all defined modules. STT trait abstraction is coherent with `spawn_blocking` for inference calls.

**Structure Alignment:** ✅
Project structure maps 1:1 with architectural boundaries. Each module owns its system API surface (injection.rs → AXUIElement only; hotkey.rs → event tap only; permissions.rs → permission APIs only). Frontend is correctly scoped to rendering only.

### Requirements Coverage Validation

**Functional Requirements:** ✅ 31/31 v1 FRs covered
All FR categories map to specific files (see Requirements to Structure Mapping). v2 FRs (FR32–40) are deferred and additive — no v1 module modifications required.

**Non-Functional Requirements:** ✅ All addressed
- NFR1 (latency): spawn_blocking for Whisper inference; async pipeline
- NFR2 (async model load): WhisperBackend loads on background thread
- NFR3 (overlay < 200ms): pre-created hidden window
- NFR5–6 (graceful failure): thiserror + anyhow error handling patterns
- NFR7 (sleep/wake): re-registration on `NSWorkspace.didWakeNotification`
- NFR8 (hotkey survives focus): global event tap, process-wide registration
- NFR9–10 (resource limits): event-driven, one model in memory, no polling
- NFR12–14 (privacy): enforced by dependency tree + in-memory-only audio

### Gap Analysis & Resolutions

**Critical — Resolved in this document:**

1. **cpal audio stream lifecycle** — The `spawn_blocking` rule applies to Whisper inference only. Audio capture uses a dedicated `std::thread::spawn` thread that owns the cpal stream for the duration of PTT hold. A `tokio::sync::oneshot` channel signals start/stop. Audio buffer is moved to the inference thread on PTT release.

2. **AXUIElement word position bridge** — `injection.rs` exposes two functions: `inject_text(text: &str) -> Result<()>` and `focused_word_screen_rect() -> Result<CGRect>`. `overlay.rs` calls the latter to position the overlay window before showing it.

**Important — Resolved in this document:**

3. **Sleep/wake mechanism** — `hotkey.rs` subscribes to macOS `NSWorkspace.didWakeNotification` via the `objc2` crate. On wake: re-register PTT hotkey. `permissions.rs` re-checks Accessibility grant on wake (it can be revoked while sleeping).

4. **Confidence threshold** — Named constant `DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.85` in `config.rs`. Stored in user settings as `confidence_threshold`; configurable in settings UI (nice-to-have for v1, required for v2 tuning).

**Nice-to-Have — Deferred:**

5. `cargo deny` config to enforce no-HTTP-crates rule in CI
6. `onboarding.js` first-launch detection: check for absence of config store file in `~/Library/Application Support/icantspell/`

### Architecture Completeness Checklist

**✅ Requirements Analysis**
- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed (Medium v1, High v2)
- [x] Technical constraints identified (macOS-only APIs, no sandbox, no App Store)
- [x] Cross-cutting concerns mapped (permissions, async model load, privacy, error recovery)

**✅ Architectural Decisions**
- [x] Critical decisions documented (STT trait, overlay strategy, IPC pattern)
- [x] Technology stack fully specified with versions (Tauri v2.10.3, whisper-rs, cpal, etc.)
- [x] Integration patterns defined (Tauri commands + events)
- [x] Performance considerations addressed (spawn_blocking, pre-created windows, event-driven)

**✅ Implementation Patterns**
- [x] Naming conventions established (snake_case IPC, Rust idioms)
- [x] Structure patterns defined (module organization, frontend layout)
- [x] Communication patterns specified (typed IPC payloads, event table)
- [x] Process patterns documented (error handling, async, logging, tests)

**✅ Project Structure**
- [x] Complete directory structure defined
- [x] Component boundaries established (single-responsibility per module)
- [x] Integration points mapped (data flow diagrams)
- [x] Requirements to structure mapping complete (FR table)

### Architecture Readiness Assessment

**Overall Status: READY FOR IMPLEMENTATION**

**Confidence Level: High**

**Key Strengths:**
- Clean pipeline separation: voice mode and typing mode share nothing except AXUIElement injection
- STT trait ensures `whisper-rs` is never a direct dependency outside `stt/`
- Privacy is structurally enforced (no HTTP crates), not policy-dependent
- Pre-created overlay window satisfies NFR3 without runtime complexity
- Implementation sequence (10 steps) provides clear agent ordering

**Areas for Future Enhancement (post-v1):**
- `cargo deny` CI enforcement for dependency privacy audit
- Confidence threshold exposed in settings UI
- Windows cross-platform path (Tauri supports it; no v1 commitment)
- Custom Whisper fine-tuning for dyslexic speech patterns

### Implementation Handoff

**First Implementation Step:**
```bash
cargo install create-tauri-app --locked
cargo create-tauri-app icantspell --template vanilla
```

**Agent Implementation Order:**
1. `error.rs` → `stt/mod.rs` (trait + types) — foundation everything else builds on
2. `stt/whisper.rs` (WhisperBackend) — spike this early; it's the primary v1 risk
3. `audio.rs` + `hotkey.rs` (PTT pipeline)
4. `injection.rs` (AXUIElement text injection + word rect)
5. Tauri IPC commands + event schema in `main.rs`
6. `overlay.rs` + `src/overlay.html/js` (correction overlay)
7. `config.rs` (Tauri store + Settings struct)
8. `permissions.rs` + `src/onboarding.html/js`
9. Menu bar controls (`src/index.html/js`, tray menu)
10. `.github/workflows/ci.yml` (clippy + test)
