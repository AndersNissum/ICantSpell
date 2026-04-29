---
stepsCompleted: [step-01-init, step-02-discovery, step-02b-vision, step-02c-executive-summary, step-03-success, step-04-journeys, step-05-domain, step-06-innovation, step-07-project-type, step-08-scoping, step-09-functional, step-10-nonfunctional, step-11-polish]
releaseMode: phased
inputDocuments:
  - _bmad-output/brainstorming/brainstorming-session-2026-04-27-0740.md
workflowType: 'prd'
classification:
  projectType: desktop_app
  domain: general-accessibility
  complexity: medium
  projectContext: greenfield
---

# Product Requirements Document — ICantSpell

**Author:** Anders
**Date:** 2026-04-27
**Type:** macOS desktop app (Tauri) · Greenfield · Open source · Personal use

## Executive Summary

ICantSpell is a macOS menu bar utility for dyslexic users that eliminates flow paralysis — the cognitive freeze that occurs when a user sees a broken word but cannot produce the correct spelling. It operates through two independent pipelines: a voice-first mode (push-to-talk → Whisper.cpp transcription → low-friction correction overlay) that bypasses spelling entirely, and a typing mode (pause-triggered → context-aware autocomplete via hybrid rule/DeBERTa-v3-small) that intercepts the freeze before it occurs. All processing is 100% local — no data leaves the device. The app lives in the menu bar with no Dock presence and activates invisibly across all macOS applications via AXUIElement and CGEventTap.

### What Makes This Special

Every design decision optimizes against a single metric: preventing the freeze. Voice is the primary intervention, not a convenience feature. Autocomplete is phonetically-driven and context-aware — built to catch "from/form" and "there/their" errors that are spelled correctly but semantically wrong. Correction interactions are one-tap, never keyboard-driven. The privacy guarantee ("your text never leaves your device") is architectural, not a setting — it serves psychological safety as much as data security. The 400–600ms pause trigger uses the user's own hesitation rhythm as the input signal, requiring no explicit invocation.

## Success Criteria

### User Success

The defining success moment: a user completes a piece of writing — an email, a message, a document — and realizes they never stopped once to fight a word. No freeze. No hunting. The thought made it out intact.

Secondary indicators:
- Voice transcription is accurate enough that error correction is the exception, not the workflow
- Whisper correction interactions complete in one tap and under 3 seconds
- The app is invisible until needed and disappears immediately after — zero cognitive overhead between uses

### Business Success

Personal, mission-driven project — built by a dyslexic developer, for dyslexic users. Open source, free, no monetization. Success in 6–12 months: the builder uses ICantSpell daily and it materially reduces writing friction in their own life.

### Technical Success

- PTT → transcription → text injection latency under 2 seconds on Apple Silicon M-series
- Whisper.cpp transcription accuracy high enough that re-dictation is rare in normal use
- Text injection works reliably across 5 primary macOS apps (Safari, Mail, Notes, Slack, VS Code)
- Zero network calls in all usage paths — verifiable by network monitor
- App memory footprint under 300MB at idle (model loaded, no active transcription)

### Measurable Outcomes

- Builder uses ICantSpell as primary writing tool daily within 3 months of v1 launch
- v1 voice mode works in at least 5 target apps without manual configuration
- Whisper error rate low enough that users don't abandon voice mode for keyboard fallback

## User Journeys

### Journey 1: Anders — Writing Without Stopping (Voice Mode, Happy Path)

Anders is writing a work email. He holds the PTT hotkey, speaks the sentence, releases. The words appear in the email field — clean, correct, exactly what he said. He holds the hotkey again, speaks the next sentence. The email is done in two minutes. He sends it without re-reading for spelling.

For the first time in years, the thought made it out of his head and onto the screen without a fight.

**Emotional arc:** Anxiety → focus → relief → quiet pride
**Capabilities revealed:** Global PTT hotkey, mic capture, Whisper.cpp transcription, AXUIElement text injection, menu bar presence

---

### Journey 2: Anders — Whisper Gets It Wrong (Recovery Path)

Anders dictates "I'll forward the form to you." Whisper hears "from" instead of "forward." An amber underline appears under the word — subtle, not jarring. Anders taps it. A small overlay shows two alternatives and a re-dictate option. He taps re-dictate, speaks "forward," the word corrects in place. Three seconds. He moves on.

He never touched the keyboard. The thought didn't evaporate.

**Emotional arc:** Slight frustration → quick resolution → relief that it didn't spiral
**Capabilities revealed:** Whisper confidence scoring, amber underline indicator, correction overlay UI, re-dictate flow, alternative suggestions, one-tap correction

---

### Journey 3: New User — First Launch

Anders installs ICantSpell. A brief onboarding wizard opens. It walks him through two steps: grant Accessibility permission, set the PTT hotkey. The wizard closes, the app moves to the menu bar, a single notification confirms it's ready. He holds the hotkey and dictates his first sentence.

**Emotional arc:** Mild uncertainty → straightforward setup → immediate first use
**Capabilities revealed:** Onboarding wizard, Accessibility permission request, hotkey configuration, menu bar transition

---

### Journey 4: Open Source Contributor — Adding a Whisper Model Variant

A developer with dyslexia finds ICantSpell on GitHub. They want to try a smaller, faster Whisper model on their older MacBook Air. They clone the repo, find the STT integration layer, swap in a different `whisper-rs` configuration, rebuild. It works. They open a PR with the model config and a note about Intel Mac performance.

**Emotional arc:** Discovery → experimentation → contribution
**Capabilities revealed:** Clean STT abstraction boundary, documented build process, clear integration points

---

### Journey Requirements Summary

| Capability | Revealed By |
|---|---|
| Global PTT hotkey (configurable) | Journey 1, 3 |
| Microphone capture pipeline | Journey 1, 2 |
| Whisper.cpp transcription (whisper-rs) | Journey 1, 2 |
| Confidence scoring on transcription output | Journey 2 |
| AXUIElement text injection | Journey 1, 2 |
| Amber underline correction indicator | Journey 2 |
| Correction overlay (tap → re-dictate or alternatives) | Journey 2 |
| Menu bar presence, no Dock icon | Journey 1, 3 |
| First-launch onboarding wizard | Journey 3 |
| Accessibility permission request flow | Journey 3 |
| Clean STT abstraction layer | Journey 4 |

## Domain-Specific Requirements

### macOS Permissions Model

The app requires up to three sensitive permissions depending on version:
- **Microphone** — v1 voice mode
- **Accessibility** (AXUIElement) — v1 text injection
- **Input Monitoring** (CGEventTap) — v2 typing mode only

Each requires explicit user grant in System Settings. The onboarding flow must request these gracefully with clear explanations — users will be rightfully skeptical of "accessibility + microphone" permissions from an unknown app.

### Distribution — v1: Personal Use Only

No distribution goal for v1. Built for the developer's own use; potentially shared informally with friends. No Apple Developer account, no code signing, no notarization required — unsigned apps run on the developer's own machine with a one-time Gatekeeper override.

If the project gains organic traction through open source, distribution options will be evaluated then. Notarization and signing are only relevant if/when distributing to others.

### Privacy as Architectural Constraint

"No network calls" is enforced architecturally:
- No crash reporters (Sentry, Bugsnag, etc.)
- No analytics or telemetry
- No server-side update checks
- Any future self-update mechanism must be opt-in and user-triggered

### Non-Requirements

- App Store distribution — blocked by sandbox restrictions on AXUIElement/CGEventTap
- Sandboxing — incompatible with required OS-level APIs

## Innovation & Novel Patterns

### Detected Innovation Areas

**Freeze Prevention as Primary Design Goal**
Existing assistive writing tools treat dyslexia as a spelling accuracy problem — they correct words after they're wrong. ICantSpell reframes the problem: the harmful moment is the freeze that follows a broken word, not the misspelling itself. Every design decision optimizes against flow interruption.

**Local ML as a System-Wide Accessibility Layer**
Whisper.cpp and DeBERTa-v3-small run entirely on-device and inject into any macOS application via AXUIElement — no cloud, no app-specific plugin, no integration work per target app.

**Hesitation Rhythm as Input Signal (v2)**
The 400–600ms typing pause corresponds to the behavioral moment when a dyslexic user suspects something is wrong. The tool activates at the exact point of need without requiring any user action. The freeze itself is the signal.

**Dyslexia-Specific Contextual Error Correction (v2)**
Standard spell-checkers are blind to dyslexic substitution errors ("from/form," "there/their/they're") because both words are correctly spelled. The hybrid corrector — curated dyslexia error dictionary → DeBERTa context window — targets this specific error class: phonetically similar, semantically distinct, contextually resolvable.

### Market Context & Competitive Landscape

| Tool | Gap |
|---|---|
| Apple Dictation / macOS built-in | No dyslexia-specific tuning, no correction overlay, cloud-dependent |
| Dragon NaturallySpeaking | Expensive, Windows-primary, requires training, no dyslexia-specific design |
| Grammarly | Cloud-only, no voice mode, misses dyslexic substitution patterns, requires per-app integration |
| Voice Control (macOS Accessibility) | Full system control tool, not optimized for fluid writing, high cognitive overhead |

No existing tool combines: local-only processing + dyslexia-specific error patterns + freeze-prevention UX + system-wide injection without per-app integration.

### Validation Approach

- **v1:** Does the builder use voice mode daily within 3 months? Does it reduce freeze moments? Self-reported qualitative validation is sufficient.
- **v2:** Does the pause trigger fire at the right moment without false positives? Are DeBERTa suggestions accurate enough to feel helpful rather than intrusive?

## Desktop App Specific Requirements

### Platform Support

- **Primary:** macOS on Apple Silicon (M-series); development and testing on M3 Pro MacBook
- **Intel Mac:** Not a v1 goal — no optimization, testing, or bug-fixing effort directed at Intel hardware
- **Windows:** Not ruled out; Tauri preserves the path; no commitment for v1
- **Minimum macOS version:** Tauri default — no custom constraint

### System Integration

**v1 — Voice Mode:**
- `AXUIElement` — identifies active text field and injects transcribed text
- Microphone capture via system audio APIs — active only during PTT hold

**v2 — Typing Mode:**
- `CGEventTap` — system-wide keystroke monitoring for pause detection; requires Input Monitoring permission
- `AXUIElement` — reads surrounding text context for DeBERTa input

Neither integration works under App Store sandboxing.

### Implementation Considerations

- **Runtime:** Tauri (Rust backend + webview UI); menu bar via Tauri system tray API; no Dock icon
- **Build toolchain:** Rust + Cargo; standard Tauri build pipeline
- **Model bundling:** Whisper base (~150MB) bundled; additional models downloaded on demand to local directory
- **Updates:** Manual (pull from repo, rebuild) for v1; any future update mechanism must be opt-in and network-call-free

### Offline Architecture

Fully offline by design — not a mode, the only mode. All inference runs locally via ONNX Runtime. No feature is gated on connectivity.

## Project Scoping & Phased Development

### MVP Strategy

Problem-solving MVP — the minimum that meaningfully reduces flow paralysis for the builder. v1 is complete and useful on its own. v2 is a separate, harder engineering problem that begins only after v1 is in daily use.

**Resource:** Solo developer.

### v1 Feature Set — Voice Mode

**Journeys supported:** 1 (happy path), 2 (recovery), 3 (first launch)

**Must-have:**
- Global PTT hotkey (configurable)
- Microphone capture — active only during PTT hold
- Whisper.cpp transcription via `whisper-rs`
- AXUIElement text injection into active text field
- Whisper confidence scoring
- Amber underline for low-confidence output
- Correction overlay: re-dictate or pick alternatives (one-tap, no keyboard)
- Whisper model selection (tiny/base/small/medium) and local model download
- First-launch onboarding wizard (Accessibility + Microphone permissions, hotkey config)
- Menu bar only — no Dock icon
- Zero network calls

**Nice-to-have for v1 (defer if needed):**
- Hotkey customization UI beyond initial onboarding
- Compatibility testing beyond the 5 primary target apps

### v2 Feature Set — Typing Mode

Built after v1 is stable:
- CGEventTap pause detection (~500ms default, user-configurable)
- AXUIElement context reading for DeBERTa input
- Hybrid corrector: dyslexia error dictionary → DeBERTa-v3-small via ONNX Runtime
- Floating pill overlay — 2–3 suggestions, click or number key, Esc/continue to dismiss
- Input Monitoring permission added to onboarding

### Future

- Pluggable STT abstraction layer (swap in different STT engines)
- Optional cloud LLM via user-supplied API key (explicit privacy tradeoff; local-only remains the default)
- Windows support (Tauri cross-platform path)
- Custom Whisper fine-tuning for dyslexic speech patterns

### Risk Mitigation

| Risk | Mitigation |
|---|---|
| AXUIElement injection fails in some apps | Test v1 against 5 primary targets; document known gaps |
| Whisper.cpp latency too high | M3 Pro is primary test machine; start with base model; tune size if needed |
| `whisper-rs` integration complexity | Spike early — it's the core v1 dependency; fail fast |
| Correction overlay UX feels clunky | Iterate on builder's own daily use |
| Pause trigger (v2) fires too aggressively | Make threshold user-configurable; default 500ms |
| DeBERTa suggestions feel intrusive (v2) | Easy dismiss (Esc/continue); suggestions never block input |

**Resource risk:** Solo developer. If v1 stalls, v2 never starts — keep v1 ruthlessly small.

## Functional Requirements

### Voice Input

- **FR1:** User can activate microphone capture by holding a global PTT hotkey
- **FR2:** User can release the PTT hotkey to end capture and trigger transcription
- **FR3:** User can configure the PTT hotkey (initial setup + settings)
- **FR4:** System captures microphone audio only while the PTT hotkey is held — no background listening

### Speech Transcription

- **FR5:** System transcribes captured audio using a local Whisper.cpp model (via `whisper-rs`)
- **FR6:** System produces a confidence score for each transcription output
- **FR7:** System performs all transcription inference locally — no network calls at any point

### Text Injection

- **FR8:** System identifies the active text field in the frontmost macOS application
- **FR9:** System injects transcribed text into the active text field at the current cursor position
- **FR10:** System supports text injection across macOS applications without per-app configuration

### Transcription Error Handling

- **FR11:** System flags low-confidence transcription output with an amber underline indicator
- **FR12:** User can open a correction overlay by tapping an amber-underlined word
- **FR13:** User can re-dictate a flagged word from within the correction overlay
- **FR14:** User can select an alternative transcription suggestion from the correction overlay
- **FR15:** User can dismiss the correction overlay without making any change
- **FR16:** All correction interactions are completable without keyboard input

### App Presence & Controls

- **FR17:** System runs as a menu bar application with no Dock icon
- **FR18:** User can access app settings from the menu bar icon
- **FR19:** User can enable or disable voice mode from the menu bar

### Onboarding & Permissions

- **FR20:** System detects first launch and presents an onboarding wizard
- **FR21:** User can grant Accessibility permission through the onboarding flow
- **FR22:** User can grant Microphone permission through the onboarding flow
- **FR23:** User can set the PTT hotkey during onboarding
- **FR24:** System validates required permissions are granted before activating voice mode
- **FR25:** System notifies the user if a required permission is revoked after initial setup

### Privacy & Data

- **FR26:** System makes zero outbound network calls in all usage paths
- **FR27:** System performs all ML inference on-device with no cloud fallback
- **FR28:** System stores no user text, audio, or transcription history persistently

### Model Management

- **FR29:** User can select which Whisper model to use from app settings (tiny, base, small, medium)
- **FR30:** User can download additional Whisper model files from within the app (local download only — no account, no cloud service)
- **FR31:** System ships with one bundled default model (base) and loads additional models from a local directory

### v2 Capabilities — Typing Mode

- **FR32 [v2]:** System monitors keystrokes globally and detects typing pauses above a configurable threshold
- **FR33 [v2]:** System reads surrounding text context from the active text field
- **FR34 [v2]:** System matches typed input against a curated dyslexia error dictionary
- **FR35 [v2]:** System disambiguates contextually similar errors using DeBERTa-v3-small via ONNX Runtime
- **FR36 [v2]:** System presents 2–3 autocomplete suggestions in a floating pill overlay
- **FR37 [v2]:** User can accept a suggestion by clicking it or pressing its number key
- **FR38 [v2]:** User can dismiss the suggestion overlay by pressing Esc or continuing to type
- **FR39 [v2]:** User can configure the pause detection threshold
- **FR40 [v2]:** User can grant Input Monitoring permission through the onboarding flow

## Non-Functional Requirements

### Performance

- **NFR1:** PTT → transcription → text injection latency under 2 seconds on Apple Silicon M-series for utterances up to 15 seconds
- **NFR2:** Whisper model loads asynchronously — menu bar icon appears immediately on launch regardless of model load state
- **NFR3:** Correction overlay appears within 200ms of user tapping an amber-underlined word
- **NFR4 [v2]:** Autocomplete suggestions appear within 300ms of pause detection threshold being reached

### Reliability

- **NFR5:** Whisper transcription failure surfaces a non-intrusive error indicator and recovers gracefully — no crash, no frozen state
- **NFR6:** AXUIElement injection failure is reported to the user without crashing or corrupting the target app's content
- **NFR7:** App survives macOS sleep/wake cycles without requiring a restart
- **NFR8:** PTT hotkey registration survives app focus changes — active regardless of which app is in the foreground

### Resource Usage

- **NFR9:** Memory footprint under 300MB at idle (model loaded, no active transcription)
- **NFR10:** CPU usage below 2% at idle — no background polling between PTT activations
- **NFR11:** CPU/GPU spikes during active transcription are acceptable but must not cause fan spin-up for dictations under 30 seconds

### Privacy

- **NFR12:** Zero outbound network connections in any usage path — verifiable under Little Snitch or equivalent
- **NFR13:** No audio, transcription text, or user input written to disk outside of explicit model download actions
- **NFR14:** All OS-level permission grants are explicit, user-initiated, and documented in onboarding
