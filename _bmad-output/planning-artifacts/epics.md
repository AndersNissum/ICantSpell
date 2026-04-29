---
stepsCompleted: [step-01-validate-prerequisites, step-02-design-epics, step-03-create-stories, step-04-final-validation]
status: complete
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
---

# ICantSpell - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for ICantSpell, decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

**Voice Input**
FR1: User can activate microphone capture by holding a global PTT hotkey
FR2: User can release the PTT hotkey to end capture and trigger transcription
FR3: User can configure the PTT hotkey (initial setup + settings)
FR4: System captures microphone audio only while the PTT hotkey is held — no background listening

**Speech Transcription**
FR5: System transcribes captured audio using a local Whisper.cpp model (via `whisper-rs`)
FR6: System produces a confidence score for each transcription output
FR7: System performs all transcription inference locally — no network calls at any point

**Text Injection**
FR8: System identifies the active text field in the frontmost macOS application
FR9: System injects transcribed text into the active text field at the current cursor position
FR10: System supports text injection across macOS applications without per-app configuration

**Transcription Error Handling**
FR11: System flags low-confidence transcription output with an amber underline indicator
FR12: User can open a correction overlay by tapping an amber-underlined word
FR13: User can re-dictate a flagged word from within the correction overlay
FR14: User can select an alternative transcription suggestion from the correction overlay
FR15: User can dismiss the correction overlay without making any change
FR16: All correction interactions are completable without keyboard input

**App Presence & Controls**
FR17: System runs as a menu bar application with no Dock icon
FR18: User can access app settings from the menu bar icon
FR19: User can enable or disable voice mode from the menu bar

**Onboarding & Permissions**
FR20: System detects first launch and presents an onboarding wizard
FR21: User can grant Accessibility permission through the onboarding flow
FR22: User can grant Microphone permission through the onboarding flow
FR23: User can set the PTT hotkey during onboarding
FR24: System validates required permissions are granted before activating voice mode
FR25: System notifies the user if a required permission is revoked after initial setup

**Privacy & Data**
FR26: System makes zero outbound network calls in all usage paths
FR27: System performs all ML inference on-device with no cloud fallback
FR28: System stores no user text, audio, or transcription history persistently

**Model Management**
FR29: User can select which Whisper model to use from app settings (tiny, base, small, medium)
FR30: User can download additional Whisper model files from within the app (local download only — no account, no cloud service)
FR31: System ships with one bundled default model (base) and loads additional models from a local directory

**v2 Capabilities (deferred)**
FR32 [v2]: System monitors keystrokes globally and detects typing pauses above a configurable threshold
FR33 [v2]: System reads surrounding text context from the active text field
FR34 [v2]: System matches typed input against a curated dyslexia error dictionary
FR35 [v2]: System disambiguates contextually similar errors using DeBERTa-v3-small via ONNX Runtime
FR36 [v2]: System presents 2–3 autocomplete suggestions in a floating pill overlay
FR37 [v2]: User can accept a suggestion by clicking it or pressing its number key
FR38 [v2]: User can dismiss the suggestion overlay by pressing Esc or continuing to type
FR39 [v2]: User can configure the pause detection threshold
FR40 [v2]: User can grant Input Monitoring permission through the onboarding flow

### NonFunctional Requirements

**Performance**
NFR1: PTT → transcription → text injection latency under 2 seconds on Apple Silicon M-series for utterances up to 15 seconds
NFR2: Whisper model loads asynchronously — menu bar icon appears immediately on launch regardless of model load state
NFR3: Correction overlay appears within 200ms of user tapping an amber-underlined word
NFR4 [v2]: Autocomplete suggestions appear within 300ms of pause detection threshold being reached

**Reliability**
NFR5: Whisper transcription failure surfaces a non-intrusive error indicator and recovers gracefully — no crash, no frozen state
NFR6: AXUIElement injection failure is reported to the user without crashing or corrupting the target app's content
NFR7: App survives macOS sleep/wake cycles without requiring a restart
NFR8: PTT hotkey registration survives app focus changes — active regardless of which app is in the foreground

**Resource Usage**
NFR9: Memory footprint under 300MB at idle (model loaded, no active transcription)
NFR10: CPU usage below 2% at idle — no background polling between PTT activations
NFR11: CPU/GPU spikes during active transcription are acceptable but must not cause fan spin-up for dictations under 30 seconds

**Privacy**
NFR12: Zero outbound network connections in any usage path — verifiable under Little Snitch or equivalent
NFR13: No audio, transcription text, or user input written to disk outside of explicit model download actions
NFR14: All OS-level permission grants are explicit, user-initiated, and documented in onboarding

### Additional Requirements

From Architecture — Technical requirements impacting story creation:

- **Starter template:** Initialize project with `cargo create-tauri-app icantspell --template vanilla` (Tauri v2.10.3); reference `ahkohd/tauri-macos-menubar-app-example` for system tray config
- **STT abstraction first:** `SpeechToText` trait + `TranscriptionResult` type in `stt/mod.rs` must exist before any Whisper integration work begins — everything else depends on this interface
- **Correction overlay pre-creation:** Overlay window must be created at app startup in a hidden state (`visible: false`, `always_on_top: true`, borderless) — never spawned on demand — to satisfy NFR3 (< 200ms appearance)
- **Async model loading:** `WhisperBackend::load()` runs on a background thread via `tokio::task::spawn_blocking`; menu bar appears and PTT registers before model is ready
- **Audio capture threading:** `cpal` capture uses a dedicated `std::thread::spawn` thread; `tokio::sync::oneshot` channel signals PTT start/stop; buffer moved to inference thread on PTT release
- **AXUIElement bridge:** `injection.rs` must expose both `inject_text(text: &str) -> Result<()>` and `focused_word_screen_rect() -> Result<CGRect>` for overlay positioning
- **Sleep/wake recovery:** `hotkey.rs` subscribes to `NSWorkspace.didWakeNotification` via `objc2` crate to re-register PTT hotkey; `permissions.rs` re-checks Accessibility on wake
- **Confidence threshold constant:** `DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.85` defined in `config.rs`; stored in user settings as `confidence_threshold`
- **IPC naming:** All Tauri IPC event names use `snake_case` on both Rust emit and JS listen sides (e.g., `transcription_ready`, `show_correction_overlay`)
- **Tauri command pattern:** All `#[tauri::command]` functions return `Result<T, String>`; never `unwrap()` or panic in command handlers
- **Error handling:** Module-level typed errors via `thiserror`; `anyhow` at application call sites only
- **Logging:** `tracing` + `tracing-subscriber` only — never `println!` or `eprintln!` in library code
- **Test placement:** Unit tests inline with `#[cfg(test)]` in each module; integration tests in `src-tauri/tests/`
- **Privacy enforcement:** No HTTP client crates (`reqwest`, `hyper`, `ureq`, `surf`, etc.) in `Cargo.toml` — enforced by CI
- **CI/CD:** GitHub Actions workflow: `cargo clippy --all-targets -- -D warnings` + `cargo test` on push to `main` and PRs
- **Three Tauri windows:** `main` (menu bar popover, hidden by default), `overlay` (correction overlay, always-on-top, transparent, hidden by default), `onboarding` (first-launch wizard, hidden by default)
- **Implementation sequence (from Architecture):** error.rs → stt/mod.rs → stt/whisper.rs → audio.rs + hotkey.rs → injection.rs → Tauri IPC schema → overlay.rs + frontend → config.rs → permissions.rs + onboarding → menu bar controls → CI workflow

### UX Design Requirements

No UX Design document present for this project.

### FR Coverage Map

| FR | Epic | Description |
|---|---|---|
| FR1 | Epic 3 | PTT hotkey activates mic capture |
| FR2 | Epic 3 | PTT release triggers transcription |
| FR3 | Epic 2 | PTT hotkey configured during onboarding |
| FR4 | Epic 3 | Mic capture only during PTT hold |
| FR5 | Epic 3 | Whisper.cpp transcription via whisper-rs |
| FR6 | Epic 3 | Confidence score on transcription output |
| FR7 | Epic 3 | Local-only inference, zero network |
| FR8 | Epic 3 | Identify active text field via AXUIElement |
| FR9 | Epic 3 | Inject transcribed text at cursor |
| FR10 | Epic 3 | Works across apps without configuration |
| FR11 | Epic 4 | Amber underline on low-confidence output |
| FR12 | Epic 4 | Tap underline to open correction overlay |
| FR13 | Epic 4 | Re-dictate from correction overlay |
| FR14 | Epic 4 | Select alternative suggestion from overlay |
| FR15 | Epic 4 | Dismiss overlay without change |
| FR16 | Epic 4 | All corrections completable without keyboard |
| FR17 | Epic 1 | Menu bar app, no Dock icon |
| FR18 | Epic 5 | Access settings from menu bar |
| FR19 | Dropped (v1) | Enable/disable voice mode — quit app instead; adds UI complexity for no benefit |
| FR20 | Epic 2 | First launch triggers onboarding wizard |
| FR21 | Epic 2 | Grant Accessibility permission in onboarding |
| FR22 | Epic 2 | Grant Microphone permission in onboarding |
| FR23 | Epic 2 | Set PTT hotkey in onboarding |
| FR24 | Epic 2 | Validate permissions before activating voice |
| FR25 | Epic 2 | Notify if permission revoked post-setup |
| FR26 | Epic 3 | Zero outbound network calls |
| FR27 | Epic 3 | All ML inference on-device |
| FR28 | Epic 3 | No persistent storage of audio/text |
| FR29 | Epic 5 | Select Whisper model in settings |
| FR30 | Epic 5 | Download additional models locally |
| FR31 | Epic 5 | Bundled base model + local model directory |
| FR32–40 | Deferred (v2) | Typing mode pipeline |

## Epic List

### Epic 1: App Shell & Foundation
The Tauri app scaffolds, compiles, and runs as a macOS menu bar utility with no Dock icon. Menu bar icon appears, basic popover window opens, three Tauri windows (popover, overlay, onboarding) are wired up, and the CI pipeline is in place. Core infrastructure — error types, config skeleton, STT trait interface — is ready for all feature epics to build on.

**FRs covered:** FR17
**Architecture deliverables:** Starter template init, `error.rs`, `stt/mod.rs` (trait + types), `config.rs` skeleton, three Tauri window setup, GitHub Actions CI (clippy + test)

---

### Epic 2: First Launch & Onboarding
A new user installs the app and is guided through a wizard that explains what the app does, requests Accessibility and Microphone permissions with clear rationale, and lets them set their PTT hotkey. After completion, required permissions are validated and the app is ready to use. If a permission is later revoked, the user is notified gracefully.

**FRs covered:** FR3, FR20, FR21, FR22, FR23, FR24, FR25
**NFRs addressed:** NFR14

---

### Epic 3: Voice Dictation Pipeline
The core user experience: user holds the PTT hotkey, speaks, releases, and the transcribed text appears in whichever text field is active — in any macOS app, without configuration. Whisper model loads asynchronously in the background. All audio stays in-memory and is discarded after injection. Zero network calls.

**FRs covered:** FR1, FR2, FR4, FR5, FR6, FR7, FR8, FR9, FR10, FR26, FR27, FR28
**NFRs addressed:** NFR1, NFR2, NFR7, NFR8, NFR9, NFR10, NFR11, NFR12, NFR13

---

### Epic 4: Transcription Error Recovery
When Whisper output falls below the confidence threshold, an amber underline appears under the flagged word. The user taps it to open a correction overlay showing re-dictate and alternative suggestions. Every interaction — accept, re-dictate, dismiss — is one tap; no keyboard needed. Failures recover gracefully without crashing or corrupting the target app.

**FRs covered:** FR11, FR12, FR13, FR14, FR15, FR16
**NFRs addressed:** NFR3, NFR5, NFR6

---

### Epic 5: Settings & Model Management
User can access a settings panel from the menu bar, toggle voice mode on/off, choose from Whisper model sizes (tiny/base/small/medium), and trigger local downloads of additional models. The bundled `base` model is available from first launch.

**FRs covered:** FR18, FR29, FR30, FR31
**Note:** FR19 (voice mode toggle) dropped from v1 — quit the app to disable; toggle adds UI complexity for no benefit.

---

## Epic 1: App Shell & Foundation

The Tauri app scaffolds, compiles, and runs as a macOS menu bar utility with no Dock icon. Menu bar icon appears, basic popover window opens, three Tauri windows (popover, overlay, onboarding) are wired up, and the CI pipeline is in place. Core infrastructure — error types, config skeleton, STT trait interface — is ready for all feature epics to build on.

### Story 1.1: Tauri Project Scaffold & Menu Bar Shell

As a developer,
I want the Tauri project initialized and running as a macOS menu bar application with no Dock icon,
So that there is a working build baseline for all future feature development.

**Acceptance Criteria:**

**Given** the developer runs `cargo create-tauri-app icantspell --template vanilla`,
**When** the app builds and launches,
**Then** the menu bar icon appears and no Dock icon is shown.

**Given** `tauri.conf.json` is configured with `"activationPolicy": "accessory"`,
**When** the app launches,
**Then** the app has no Dock presence and no visible window on startup.

**Given** the app is running,
**When** the user clicks the menu bar icon,
**Then** a minimal tray menu appears with at least a Quit option.

**Given** the developer runs `cargo tauri dev`,
**When** they modify a file in `src/`,
**Then** the webview hot-reloads without a full Rust rebuild.

**And** the project directory structure matches the Architecture spec (`src/`, `src-tauri/src/`, `src-tauri/tauri.conf.json`, etc.).

---

### Story 1.2: Core Error Types & STT Trait Interface

As a developer,
I want shared error types and the `SpeechToText` trait defined before any feature implementation,
So that all modules have a stable interface to depend on and error handling is consistent from day one.

**Acceptance Criteria:**

**Given** `error.rs` is created with `thiserror` definitions,
**When** any module imports it,
**Then** a base `AppError` enum and at least a catch-all variant are available for use.

**Given** `stt/mod.rs` is created,
**When** another module imports it,
**Then** the `SpeechToText` trait is available with the signature `fn transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult>`.

**Given** `TranscriptionResult` is defined in `stt/mod.rs`,
**When** it is constructed,
**Then** it contains exactly these fields: `text: String`, `confidence: f32`, `alternatives: Vec<String>`.

**Given** `lib.rs` re-exports the STT types,
**When** integration tests reference `lib.rs`,
**Then** `stt::SpeechToText` and `stt::TranscriptionResult` are accessible.

**And** `cargo test` passes with at least one unit test verifying `TranscriptionResult` field construction.

---

### Story 1.3: Configuration Persistence Foundation

As a developer,
I want a config module that persists user settings using the Tauri store plugin,
So that PTT hotkey binding, model selection, and confidence threshold survive app restarts.

**Acceptance Criteria:**

**Given** `config.rs` wraps the Tauri store plugin,
**When** the app starts for the first time,
**Then** it creates `~/Library/Application Support/icantspell/settings.json` with default values.

**Given** the `Settings` struct is defined,
**When** it is serialized to the store,
**Then** it contains `ptt_hotkey: String`, `selected_model: String`, and `confidence_threshold: f32`.

**Given** `DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.85` is defined as a constant in `config.rs`,
**When** no settings file exists,
**Then** the constant is used as the default value for `confidence_threshold`.

**Given** a setting is written via `config.rs`,
**When** the app is fully quit and relaunched,
**Then** the written setting is read back correctly.

**And** no audio, transcription text, or user input is ever written to the config store (NFR13/FR28).

---

### Story 1.4: Three-Window Tauri Architecture

As a developer,
I want the three Tauri windows — menu bar popover, correction overlay, and onboarding wizard — pre-created at startup in hidden state,
So that each can be shown instantly without spawn delay when needed.

**Acceptance Criteria:**

**Given** `tauri.conf.json` defines windows labeled `main`, `overlay`, and `onboarding`,
**When** the app starts,
**Then** all three windows are created but hidden (`visible: false`).

**Given** the `overlay` window is configured,
**When** it is defined in `tauri.conf.json`,
**Then** it has `alwaysOnTop: true`, `decorations: false`, and `transparent: true`.

**Given** `index.html`, `overlay.html`, and `onboarding.html` exist in `src/`,
**When** the app builds,
**Then** each window loads its corresponding HTML file without errors.

**Given** `main.js`, `overlay.js`, and `onboarding.js` exist in `src/`,
**When** each window loads,
**Then** its JS file is included and the browser console shows no errors.

**And** `styles.css` in `src/` is linked in all three HTML files.

---

### Story 1.5: CI Pipeline

As a developer,
I want a GitHub Actions workflow that lints and tests the codebase on every push and PR,
So that regressions are caught automatically and code quality is enforced.

**Acceptance Criteria:**

**Given** `.github/workflows/ci.yml` exists,
**When** a commit is pushed to `main` or a pull request is opened,
**Then** the workflow triggers automatically.

**Given** the workflow runs,
**When** `cargo clippy --all-targets -- -D warnings` executes,
**Then** any clippy warning causes the build to fail.

**Given** the workflow runs,
**When** `cargo test` executes,
**Then** all unit and integration tests must pass for the workflow to succeed.

**And** the workflow completes successfully on the initial scaffold without requiring additional code changes.

---

## Epic 2: First Launch & Onboarding

A new user installs the app and is guided through a wizard that explains what the app does, requests Accessibility and Microphone permissions with clear rationale, and lets them set their PTT hotkey. After completion, required permissions are validated and the app is ready to use. If a permission is later revoked, the user is notified gracefully.

### Story 2.1: First-Launch Detection & Onboarding Wizard Shell

As a new user,
I want the app to detect my first launch and open an onboarding wizard automatically,
So that I know what to do next without having to find any documentation.

**Acceptance Criteria:**

**Given** no settings file exists in `~/Library/Application Support/icantspell/`,
**When** the app launches,
**Then** the onboarding window is shown automatically.

**Given** the onboarding wizard opens,
**When** it renders,
**Then** it presents a welcome screen explaining what ICantSpell does in plain language (voice dictation, local-only, no data leaves device).

**Given** a settings file already exists from a previous launch,
**When** the app launches,
**Then** the onboarding window is NOT shown.

**And** the onboarding wizard is implemented in `onboarding.html` / `onboarding.js` using the existing window from Story 1.4.

---

### Story 2.2: Accessibility Permission Request

As a new user,
I want the onboarding flow to explain and request Accessibility permission,
So that I understand why it's needed before I grant it and can do so without leaving the wizard.

**Acceptance Criteria:**

**Given** the onboarding wizard is on the Accessibility permission step,
**When** the user reads the explanation,
**Then** a clear rationale is shown (e.g., "needed to inject text into other apps").

**Given** the user clicks the grant button,
**When** the app calls `AXIsProcessTrusted()` with a prompt,
**Then** macOS opens System Settings > Privacy & Security > Accessibility automatically.

**Given** the user grants Accessibility permission and returns to the app,
**When** the wizard checks permission status,
**Then** the step is marked complete and the wizard advances.

**Given** the user skips or denies the permission,
**When** the wizard checks permission status,
**Then** it shows a non-blocking warning that voice mode will not work until permission is granted.

---

### Story 2.3: Microphone Permission Request

As a new user,
I want the onboarding flow to explain and request Microphone permission,
So that audio capture works when I first try to dictate.

**Acceptance Criteria:**

**Given** the onboarding wizard is on the Microphone permission step,
**When** it renders,
**Then** a clear rationale is shown (e.g., "needed to capture your voice during PTT hold; no background listening").

**Given** the user clicks the grant button,
**When** the app requests microphone access via `AVCaptureDevice.requestAccess`,
**Then** the macOS system permission dialog appears.

**Given** the user grants Microphone permission,
**When** the wizard checks permission status,
**Then** the step is marked complete and the wizard advances.

**Given** the user denies Microphone permission,
**When** the wizard checks permission status,
**Then** it shows a non-blocking warning that voice mode requires microphone access, with a link to System Settings.

---

### Story 2.4: PTT Hotkey Configuration

As a new user,
I want to set my push-to-talk hotkey during onboarding,
So that I can start dictating immediately after setup completes.

**Acceptance Criteria:**

**Given** the onboarding wizard is on the hotkey configuration step,
**When** the user clicks the "Set Hotkey" capture field and presses a key combination,
**Then** the captured key combination is displayed (e.g., "⌥Space").

**Given** a hotkey is captured,
**When** the user confirms it,
**Then** it is written to `config.rs` via the Tauri store as `ptt_hotkey`.

**Given** the user does not set a hotkey,
**When** they proceed past this step,
**Then** a sensible default hotkey (e.g., Right Option) is saved to config.

**And** the saved hotkey persists across app restarts (verified via config roundtrip test from Story 1.3).

---

### Story 2.5: Permission Validation & Onboarding Completion

As a new user,
I want the wizard to validate that required permissions are in place before finishing,
So that the app is confirmed ready to use when onboarding closes.

**Acceptance Criteria:**

**Given** the onboarding wizard reaches its final step,
**When** it checks Accessibility and Microphone permission status,
**Then** it displays a clear "Ready" state if both are granted, or a warning listing any missing permissions.

**Given** both permissions are granted,
**When** the user clicks "Finish",
**Then** the onboarding window closes, the menu bar popover is ready, and a system notification confirms the app is active.

**Given** one or more permissions are missing,
**When** the user clicks "Finish anyway",
**Then** the onboarding closes but voice mode remains disabled until the missing permission is granted (FR24).

**And** after successful onboarding completion, the settings file exists so onboarding does not show on next launch (Story 2.1).

---

### Story 2.6: Permission Revocation Monitoring

As a user who has completed onboarding,
I want the app to notify me if I revoke a required permission after initial setup,
So that I understand why voice mode stopped working without the app crashing or freezing.

**Acceptance Criteria:**

**Given** the app is running with all permissions granted,
**When** the user revokes Accessibility or Microphone permission in System Settings,
**Then** `permissions.rs` detects the revocation within a reasonable polling interval (≤5 seconds).

**Given** a permission is revoked,
**When** detection occurs,
**Then** the app emits a `permission_revoked` IPC event with the permission name.

**Given** the `permission_revoked` event is received by the frontend,
**When** it renders,
**Then** a non-intrusive indicator appears in the menu bar popover explaining which permission was lost.

**Given** Accessibility permission is revoked,
**When** the user attempts PTT dictation,
**Then** voice mode does not crash — it surfaces a graceful error (NFR5, NFR6).

**And** re-granting the permission in System Settings and returning to the app clears the warning without requiring a restart.

---

## Epic 3: Voice Dictation Pipeline

The core user experience: user holds the PTT hotkey, speaks, releases, and the transcribed text appears in whichever text field is active — in any macOS app, without configuration. Whisper model loads asynchronously in the background. All audio stays in-memory and is discarded after injection. Zero network calls.

### Story 3.1: Global PTT Hotkey Registration

As a user,
I want my configured PTT hotkey to be active globally regardless of which app is in focus,
So that I can start dictating from anywhere without switching to ICantSpell first.

**Acceptance Criteria:**

**Given** `hotkey.rs` registers a global event tap for the configured hotkey,
**When** the user holds the hotkey in any foreground app,
**Then** `hotkey.rs` fires a keydown event that signals `audio.rs` to begin capture.

**Given** the user releases the hotkey,
**When** `hotkey.rs` fires a keyup event,
**Then** it signals `audio.rs` to stop capture and hand off the buffer.

**Given** the user switches focus between apps while the app is running,
**When** they press the PTT hotkey in any app,
**Then** the hotkey fires correctly — it is not scoped to ICantSpell's window (NFR8).

**Given** the app reads `ptt_hotkey` from config on startup,
**When** the hotkey binding is loaded,
**Then** the global registration uses the user's saved hotkey, not a hardcoded value.

**And** `cargo test` includes a unit test verifying the hotkey event parsing logic.

---

### Story 3.2: Microphone Audio Capture Pipeline

As a user,
I want audio to be captured from my microphone only while I hold the PTT hotkey,
So that there is no background listening and my privacy is protected.

**Acceptance Criteria:**

**Given** `audio.rs` owns a dedicated `std::thread::spawn` thread for the `cpal` stream,
**When** the PTT keydown signal arrives via a `tokio::sync::mpsc` channel (carrying a `CaptureCommand::Start` variant),
**Then** mic capture starts and audio samples are written into an in-memory `Vec<f32>` buffer.

**Given** the PTT keyup signal arrives as a `CaptureCommand::Stop` on the same mpsc channel,
**When** capture stops,
**Then** the completed `Vec<f32>` buffer is moved to the STT pipeline thread and no copy remains in `audio.rs`.

**Given** the PTT hotkey is not held,
**When** the app is running at idle,
**Then** no audio is captured and CPU usage attributable to audio is effectively zero (NFR10).

**Given** the audio buffer is passed to the STT pipeline,
**When** transcription completes (success or failure),
**Then** the buffer is dropped from memory — not written to disk at any point (NFR13/FR28).

**And** the mpsc channel supports repeated Start/Stop cycles across multiple PTT presses without requiring channel recreation.

---

### Story 3.3: Whisper Backend & Async Model Loading

As a user,
I want Whisper to load in the background so the menu bar icon appears immediately,
So that app startup feels instant even though the model is large.

**Acceptance Criteria:**

**Given** `stt/whisper.rs` implements `WhisperBackend` wrapping `whisper-rs`,
**When** `WhisperBackend::load(model_path)` is called,
**Then** it runs on a `tokio::task::spawn_blocking` thread and does not block the Tauri runtime.

**Given** the app starts,
**When** model loading is in progress,
**Then** the menu bar icon is already visible and the PTT hotkey is registered (NFR2).

**Given** `WhisperBackend` is ready,
**When** `transcribe(&self, audio: &[f32])` is called,
**Then** it returns a `TranscriptionResult` with `text`, `confidence`, and `alternatives`.

**Given** the PTT hotkey is pressed while the model is still loading,
**When** the keydown event fires,
**Then** the activation is queued or gracefully declined with a non-intrusive indicator — no crash.

**And** only the bundled `base` model (`src-tauri/models/ggml-base.bin`) is loaded by default; no network call is made at any point (FR7/NFR12).

---

### Story 3.4: AXUIElement Text Injection

As a user,
I want transcribed text to be injected directly into whichever text field I'm typing in,
So that my words appear where I intended without any copy-paste step.

**Acceptance Criteria:**

**Given** `injection.rs` exposes `inject_text(text: &str) -> Result<()>`,
**When** called with a non-empty string,
**Then** the text is inserted at the current cursor position in the frontmost app's active text field via AXUIElement.

**Given** `injection.rs` exposes `focused_word_screen_rect() -> Result<CGRect>`,
**When** called,
**Then** it returns the screen coordinates of the currently focused word (used later by the overlay for positioning).

**Given** the active app does not expose an accessible text field,
**When** `inject_text` is called,
**Then** it returns an `Err` — it does not crash and does not corrupt the target app's content (NFR6).

**Given** text injection succeeds,
**When** the operation completes,
**Then** the audio buffer is dropped from memory immediately after (NFR13).

**And** `injection.rs` is the only module in the codebase that calls AXUIElement APIs.

---

### Story 3.5: End-to-End Voice Pipeline Integration

As a user,
I want the full PTT → transcription → inject flow to work end-to-end within 2 seconds,
So that dictation feels immediate and doesn't interrupt my writing flow.

**Acceptance Criteria:**

**Given** the user holds the PTT hotkey and speaks a sentence up to 15 seconds,
**When** they release the hotkey,
**Then** the transcribed text appears in the active text field within 2 seconds on Apple Silicon M-series (NFR1).

**Given** the full pipeline runs (PTT → audio capture → Whisper inference via `spawn_blocking` → AXUIElement inject),
**When** transcription succeeds and confidence is at or above `DEFAULT_CONFIDENCE_THRESHOLD` (0.85),
**Then** text is injected directly with no overlay shown.

**Given** the app is at idle between PTT activations,
**When** no dictation is in progress,
**Then** CPU usage remains below 2% and no background audio capture occurs (NFR10).

**Given** the app is running during a macOS sleep/wake cycle,
**When** the machine wakes,
**Then** `hotkey.rs` re-registers the PTT hotkey via `NSWorkspace.didWakeNotification` and dictation works without a restart (NFR7).

**Given** transcription confidence is below `DEFAULT_CONFIDENCE_THRESHOLD` (0.85),
**When** the pipeline runs,
**Then** the text is still injected and a `show_correction_overlay` IPC event is emitted — the amber underline and overlay tap-to-correct behaviour are implemented in Epic 4 and are additive; no Epic 3 code changes are required when Epic 4 is added.

**And** a `src-tauri/tests/stt_pipeline.rs` integration test exercises the audio buffer → `WhisperBackend::transcribe` → `TranscriptionResult` path.

---

## Epic 4: Transcription Error Recovery

When Whisper output falls below the confidence threshold, an amber underline appears under the flagged word. The user taps it to open a correction overlay showing re-dictate and alternative suggestions. Every interaction — accept, re-dictate, dismiss — is one tap; no keyboard needed. Failures recover gracefully without crashing or corrupting the target app.

### Story 4.1: Amber Underline Indicator

As a user,
I want low-confidence transcribed words to be visually flagged with an amber underline,
So that I know at a glance when Whisper wasn't sure about something.

**Acceptance Criteria:**

**Given** `TranscriptionResult.confidence` is below `DEFAULT_CONFIDENCE_THRESHOLD` (0.85),
**When** the pipeline completes,
**Then** the text is injected into the active text field via `injection.rs` (same as the high-confidence path), AND the backend emits a `show_correction_overlay` IPC event with the word's screen coordinates.

**Given** the `show_correction_overlay` event is received by the overlay window,
**When** it renders,
**Then** an amber underline is displayed over the injected word, positioned using the screen coordinates from `focused_word_screen_rect()` — the overlay does not open automatically; it waits for a user tap (handled in Story 4.2).

**Given** the amber underline is shown,
**When** confidence is above threshold for a subsequent dictation,
**Then** the underline is not shown and text injects silently as normal.

**And** the underline indicator is rendered in the `overlay` window (pre-created in Story 1.4), not by spawning a new window.

---

### Story 4.2: Correction Overlay — Display & Dismiss

As a user,
I want to open a correction overlay by tapping an amber-underlined word,
So that I can review alternatives or re-dictate without stopping my flow.

**Acceptance Criteria:**

**Given** an amber underline is visible over a low-confidence word,
**When** the user taps it,
**Then** the correction overlay opens within 200ms showing the flagged word, up to 3 alternative transcriptions, and a re-dictate option (NFR3).

**Given** the overlay is open,
**When** the user taps anywhere outside it or presses Escape,
**Then** the overlay closes via a `dismiss_overlay` Tauri command and no text change is made (FR15).

**Given** the overlay closes,
**When** dismissed without a selection,
**Then** the original injected text remains unchanged in the target app.

**And** the overlay is positioned near the amber-underlined word using screen coordinates from `focused_word_screen_rect()` — not at a fixed screen position.

---

### Story 4.3: Accept Alternative Suggestion

As a user,
I want to tap an alternative transcription suggestion in the correction overlay,
So that I can fix a Whisper mistake in one tap without touching the keyboard.

**Acceptance Criteria:**

**Given** the correction overlay is open with alternative suggestions,
**When** the user taps one of the alternatives,
**Then** the frontend invokes `accept_correction` Tauri command with the selected text.

**Given** `accept_correction` is invoked,
**When** the backend receives it,
**Then** `injection.rs` replaces the flagged word in the target app with the selected alternative via AXUIElement.

**Given** the replacement succeeds,
**When** the operation completes,
**Then** the overlay closes, the amber underline disappears, and the cursor is positioned after the corrected word.

**Given** `injection.rs` fails to replace the word (e.g., target app lost focus),
**When** the error is returned,
**Then** the overlay closes gracefully and a non-intrusive error indicator is shown — no crash, no corrupt content (NFR6).

**And** the entire interaction (tap alternative → replacement visible) completes without any keyboard input (FR16).

---

### Story 4.4: Re-Dictate from Overlay

As a user,
I want to re-dictate a flagged word directly from the correction overlay,
So that I can fix a Whisper mistake by speaking again rather than selecting from a list.

**Acceptance Criteria:**

**Given** the correction overlay is open,
**When** the user taps the re-dictate button,
**Then** the frontend invokes the `redictate` Tauri command and the overlay transitions to a "listening" state.

**Given** the overlay is in listening state,
**When** the user speaks and releases,
**Then** audio is captured and sent through `WhisperBackend::transcribe` exactly as in the normal PTT pipeline.

**Given** re-dictation produces a result,
**When** transcription completes,
**Then** the flagged word is replaced with the new transcription via `injection.rs` and the overlay closes.

**Given** re-dictation produces another low-confidence result,
**When** transcription completes,
**Then** the overlay updates with the new word and new alternatives — the flow does not loop infinitely.

**And** re-dictation requires no keyboard input at any point (FR16).

---

### Story 4.5: Transcription Failure Recovery

As a user,
I want Whisper failures and injection failures to surface quietly without crashing the app,
So that a bad dictation attempt doesn't interrupt my work or leave the target app in a broken state.

**Acceptance Criteria:**

**Given** `WhisperBackend::transcribe` returns an `Err` (e.g., empty audio, inference failure),
**When** the error is caught in the pipeline,
**Then** a non-intrusive error indicator appears (e.g., a brief icon change in the menu bar) and no text is injected (NFR5).

**Given** `inject_text` returns an `Err` (e.g., no accessible text field found),
**When** the error is caught,
**Then** the error is logged via `tracing::warn!` and a non-intrusive indicator is shown — the target app's content is untouched (NFR6).

**Given** any pipeline failure occurs,
**When** the error is handled,
**Then** the app returns to idle state and the next PTT press works normally — no frozen state, no restart required.

**And** all error paths use `thiserror`-defined error types internally and convert to `String` only at the Tauri command boundary.

---

## Epic 5: Settings & Model Management

User can access a settings panel from the menu bar and choose from available Whisper model sizes, with the option to download additional models locally. The bundled `base` model is available from first launch.

**Note:** FR19 (voice mode on/off toggle) dropped from v1 — the app is passive by design (only activates on PTT hold); quit the app to disable.

### Story 5.1: Menu Bar Popover & Settings Panel

As a user,
I want to click the menu bar icon to open a settings popover showing app status and controls,
So that I can access model selection and see whether the app is ready without hunting through menus.

**Acceptance Criteria:**

**Given** the app is running,
**When** the user clicks the menu bar icon,
**Then** the `main` window (popover) becomes visible showing app status (ready / model loading / permission issue) and available controls (FR18).

**Given** the popover is open,
**When** the app is fully initialised with permissions granted and model loaded,
**Then** the status reads "Ready" and the model currently in use is displayed.

**Given** the popover is open,
**When** the user clicks outside it or presses Escape,
**Then** the popover closes and returns to the hidden state.

**And** the popover is implemented in `index.html` / `main.js` using the existing `main` window from Story 1.4.

---

### Story 5.2: Whisper Model Selection

As a user,
I want to select which Whisper model size the app uses from the settings panel,
So that I can trade off speed vs. accuracy based on my hardware and needs.

**Acceptance Criteria:**

**Given** the settings popover is open,
**When** the model selection UI is shown,
**Then** it lists all locally available models (at minimum the bundled `base` model) with their size labels (tiny, base, small, medium) (FR29).

**Given** the user selects a different model,
**When** they confirm the selection,
**Then** the frontend invokes a Tauri command that writes the new `selected_model` to `config.rs`.

**Given** a new model is selected,
**When** the next PTT dictation is triggered,
**Then** `WhisperBackend` is constructed with the new model path — the old model is unloaded first, then the new one loads.

**Given** the selected model file does not exist on disk,
**When** the app attempts to load it,
**Then** it falls back to the bundled `base` model and surfaces a non-intrusive warning — no crash.

**And** only one Whisper model is held in memory at a time (NFR9).

---

### Story 5.3: Bundled Model Discovery & Local Model Directory

As a user,
I want the app to ship with the `base` Whisper model ready to use out of the box,
So that dictation works immediately after install with no download step.

**Acceptance Criteria:**

**Given** the app is built,
**When** `src-tauri/models/ggml-base.bin` is included in the bundle,
**Then** `models.rs` can locate and load it without any user action (FR31).

**Given** `models.rs` scans for available models,
**When** it runs on first launch,
**Then** it discovers the bundled `base` model and any additional models in `~/Library/Application Support/icantspell/models/`.

**Given** the user has not downloaded any additional models,
**When** the model list is shown in settings,
**Then** only the bundled `base` model appears as available.

**And** `models.rs` is the only module that reads from or writes to the model directory.

---

### Story 5.4: Local Model Download

As a user,
I want to download additional Whisper model sizes from within the app,
So that I can try a larger or smaller model without leaving the settings panel.

**Acceptance Criteria:**

**Given** the settings popover shows undownloaded model sizes (tiny, small, medium),
**When** the user clicks "Download" next to a model,
**Then** a Tauri command in `models.rs` begins downloading the model file to `~/Library/Application Support/icantspell/models/` (FR30).

**Given** a download is in progress,
**When** the UI updates,
**Then** a progress indicator is shown and the user can continue using voice mode with the current model during the download.

**Given** the download completes successfully,
**When** the model file is verified,
**Then** it appears in the model selection list as available and can be selected immediately.

**Given** the download fails or is interrupted,
**When** the error is caught,
**Then** the partial file is cleaned up, an error message is shown, and the previously selected model remains active — no crash.

**And** the download uses no external account, no cloud service, and no authentication — only a direct file fetch from a documented public source (FR30).

**And** the download is implemented via macOS `URLSession` through the existing `objc2` FFI dependency — no new Rust HTTP client crate is added to `Cargo.toml`. This is the sole permitted network operation in the app; all dictation/transcription usage paths remain zero-network (NFR12).
