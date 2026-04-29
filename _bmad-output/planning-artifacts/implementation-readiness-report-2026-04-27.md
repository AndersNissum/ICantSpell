---
stepsCompleted: [step-01-document-discovery, step-02-prd-analysis, step-03-epic-coverage-validation, step-04-ux-alignment, step-05-epic-quality-review, step-06-final-assessment]
status: complete
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/epics.md
date: 2026-04-27
project: ICantSpell
---

# Implementation Readiness Assessment Report

**Date:** 2026-04-27
**Project:** ICantSpell

## PRD Analysis

### Functional Requirements

FR1: User can activate microphone capture by holding a global PTT hotkey
FR2: User can release the PTT hotkey to end capture and trigger transcription
FR3: User can configure the PTT hotkey (initial setup + settings)
FR4: System captures microphone audio only while the PTT hotkey is held — no background listening
FR5: System transcribes captured audio using a local Whisper.cpp model (via `whisper-rs`)
FR6: System produces a confidence score for each transcription output
FR7: System performs all transcription inference locally — no network calls at any point
FR8: System identifies the active text field in the frontmost macOS application
FR9: System injects transcribed text into the active text field at the current cursor position
FR10: System supports text injection across macOS applications without per-app configuration
FR11: System flags low-confidence transcription output with an amber underline indicator
FR12: User can open a correction overlay by tapping an amber-underlined word
FR13: User can re-dictate a flagged word from within the correction overlay
FR14: User can select an alternative transcription suggestion from the correction overlay
FR15: User can dismiss the correction overlay without making any change
FR16: All correction interactions are completable without keyboard input
FR17: System runs as a menu bar application with no Dock icon
FR18: User can access app settings from the menu bar icon
FR19: User can enable or disable voice mode from the menu bar [DROPPED from v1 — PTT-only activation makes toggle redundant]
FR20: System detects first launch and presents an onboarding wizard
FR21: User can grant Accessibility permission through the onboarding flow
FR22: User can grant Microphone permission through the onboarding flow
FR23: User can set the PTT hotkey during onboarding
FR24: System validates required permissions are granted before activating voice mode
FR25: System notifies the user if a required permission is revoked after initial setup
FR26: System makes zero outbound network calls in all usage paths
FR27: System performs all ML inference on-device with no cloud fallback
FR28: System stores no user text, audio, or transcription history persistently
FR29: User can select which Whisper model to use from app settings (tiny, base, small, medium)
FR30: User can download additional Whisper model files from within the app (local only)
FR31: System ships with one bundled default model (base) and loads additional models from a local directory

**Total v1 FRs: 31** (FR19 intentionally dropped; FR32–40 deferred to v2)

### Non-Functional Requirements

NFR1: PTT → transcription → text injection latency under 2 seconds on Apple Silicon M-series for utterances up to 15 seconds
NFR2: Whisper model loads asynchronously — menu bar icon appears immediately on launch regardless of model load state
NFR3: Correction overlay appears within 200ms of user tapping an amber-underlined word
NFR4 [v2]: Autocomplete suggestions appear within 300ms of pause detection threshold being reached
NFR5: Whisper transcription failure surfaces a non-intrusive error indicator and recovers gracefully — no crash, no frozen state
NFR6: AXUIElement injection failure is reported to the user without crashing or corrupting the target app's content
NFR7: App survives macOS sleep/wake cycles without requiring a restart
NFR8: PTT hotkey registration survives app focus changes — active regardless of which app is in the foreground
NFR9: Memory footprint under 300MB at idle (model loaded, no active transcription)
NFR10: CPU usage below 2% at idle — no background polling between PTT activations
NFR11: CPU/GPU spikes during active transcription are acceptable but must not cause fan spin-up for dictations under 30 seconds
NFR12: Zero outbound network connections in any usage path — verifiable under Little Snitch or equivalent
NFR13: No audio, transcription text, or user input written to disk outside of explicit model download actions
NFR14: All OS-level permission grants are explicit, user-initiated, and documented in onboarding

**Total NFRs: 14** (NFR4 deferred to v2)

### Additional Requirements

- No App Store distribution — blocked by AXUIElement/CGEventTap sandbox incompatibility
- No sandboxing — incompatible with required OS-level APIs
- Primary target: macOS Apple Silicon (M-series); Intel Mac not a v1 goal
- Unsigned binary — Gatekeeper one-time override on developer's own machine
- No crash reporters, analytics, telemetry, or server-side update checks

### PRD Completeness Assessment

The PRD is thorough and well-structured. Requirements are numbered, testable, and clearly scoped between v1 and v2. The v1/v2 boundary is explicit. Privacy constraints are architectural, not policy. No ambiguous or untestable requirements found.

## Epic Coverage Validation

### Coverage Matrix

| FR | PRD Requirement (summary) | Epic / Story | Status |
|---|---|---|---|
| FR1 | PTT hotkey activates mic capture | Epic 3 / Story 3.1 | ✅ Covered |
| FR2 | PTT release triggers transcription | Epic 3 / Story 3.1 | ✅ Covered |
| FR3 | User configures PTT hotkey | Epic 2 / Story 2.4 | ✅ Covered |
| FR4 | Mic capture only during PTT hold | Epic 3 / Story 3.2 | ✅ Covered |
| FR5 | Whisper.cpp transcription via whisper-rs | Epic 3 / Story 3.3 | ✅ Covered |
| FR6 | Confidence score on transcription output | Epic 3 / Story 3.3 | ✅ Covered |
| FR7 | Local-only inference, zero network | Epic 3 / Story 3.3 | ✅ Covered |
| FR8 | Identify active text field via AXUIElement | Epic 3 / Story 3.4 | ✅ Covered |
| FR9 | Inject transcribed text at cursor | Epic 3 / Story 3.4 | ✅ Covered |
| FR10 | Works across apps without configuration | Epic 3 / Story 3.4 | ✅ Covered |
| FR11 | Amber underline on low-confidence output | Epic 4 / Story 4.1 | ✅ Covered |
| FR12 | Tap underline to open correction overlay | Epic 4 / Story 4.2 | ✅ Covered |
| FR13 | Re-dictate from correction overlay | Epic 4 / Story 4.4 | ✅ Covered |
| FR14 | Select alternative suggestion from overlay | Epic 4 / Story 4.3 | ✅ Covered |
| FR15 | Dismiss overlay without change | Epic 4 / Story 4.2 | ✅ Covered |
| FR16 | All corrections completable without keyboard | Epic 4 / Stories 4.3, 4.4 | ✅ Covered |
| FR17 | Menu bar app, no Dock icon | Epic 1 / Story 1.1 | ✅ Covered |
| FR18 | Access settings from menu bar | Epic 5 / Story 5.1 | ✅ Covered |
| FR19 | Enable/disable voice mode | — | ✅ Intentionally dropped from v1 |
| FR20 | First launch triggers onboarding wizard | Epic 2 / Story 2.1 | ✅ Covered |
| FR21 | Grant Accessibility permission in onboarding | Epic 2 / Story 2.2 | ✅ Covered |
| FR22 | Grant Microphone permission in onboarding | Epic 2 / Story 2.3 | ✅ Covered |
| FR23 | Set PTT hotkey in onboarding | Epic 2 / Story 2.4 | ✅ Covered |
| FR24 | Validate permissions before activating voice | Epic 2 / Story 2.5 | ✅ Covered |
| FR25 | Notify if permission revoked post-setup | Epic 2 / Story 2.6 | ✅ Covered |
| FR26 | Zero outbound network calls | Epic 3 / Story 3.3 | ✅ Covered |
| FR27 | All ML inference on-device | Epic 3 / Story 3.3 | ✅ Covered |
| FR28 | No persistent storage of audio/text | Epic 3 / Story 3.2 | ✅ Covered |
| FR29 | Select Whisper model in settings | Epic 5 / Story 5.2 | ✅ Covered |
| FR30 | Download additional models locally | Epic 5 / Story 5.4 | ✅ Covered |
| FR31 | Bundled base model + local model directory | Epic 5 / Story 5.3 | ✅ Covered |

### Missing Requirements

None.

### Coverage Statistics

- Total PRD v1 FRs: 31
- FRs covered in stories: 30 (FR19 intentionally out-of-scope)
- FRs with no coverage: 0
- Coverage: **100%** of in-scope v1 requirements

## UX Alignment Assessment

### UX Document Status

Not present — no UX design document exists for this project.

### Assessment

No UX document is expected or needed here. ICantSpell's UI surface is minimal by design:
- A menu bar tray icon (no window)
- A small settings popover (~3 controls)
- A correction overlay (~3 tappable items)
- A 3-step onboarding wizard

The Architecture document fully specifies the UI approach (vanilla HTML/CSS/JS, pre-created hidden windows, Tauri IPC) and the PRD contains sufficient UX intent in its user journeys and FR descriptions. The correction overlay positioning, amber underline, and one-tap interaction patterns are all addressed in epics stories without requiring a separate UX spec.

### Warnings

None. A formal UX document would be warranted if this app had complex navigation, forms, dashboards, or responsive layouts — none of which apply here.

## Epic Quality Review

### Epic Structure Validation

| Epic | User Value? | Standalone? | Verdict |
|---|---|---|---|
| Epic 1: App Shell & Foundation | ✅ App visible in menu bar (FR17) | ✅ Stands alone | ✅ Pass — greenfield foundation epic |
| Epic 2: First Launch & Onboarding | ✅ User set up and ready to use | ✅ Uses Epic 1 only | ✅ Pass |
| Epic 3: Voice Dictation Pipeline | ✅ Core dictation experience | ✅ Uses Epics 1 & 2 | ✅ Pass |
| Epic 4: Transcription Error Recovery | ✅ One-tap error correction | ✅ Uses Epics 1 & 3 | ✅ Pass |
| Epic 5: Settings & Model Management | ✅ Model customisation | ✅ Uses Epics 1 & 3; parallel with 4 | ✅ Pass |

### Story Dependency Analysis

All within-epic story dependencies flow forward only. No story references a future story as a prerequisite. Greenfield foundation stories (1.2 STT trait, 1.3 config, 1.4 windows, 1.5 CI) are developer-infrastructure stories with no direct end-user value individually, but are explicitly accommodated by the greenfield pattern and the Architecture's mandated implementation sequence.

### Best Practices Compliance

**Epic 1** — Starter template story (1.1) correctly anchors Epic 1 Story 1. No upfront entity creation. ✅
**Epic 2** — Permission and onboarding stories are appropriately sized. ✅
**Epic 3** — Pipeline stories follow the Architecture implementation order. ✅
**Epic 4** — Overlay stories are independent and additive on top of Epic 3. ✅
**Epic 5** — Settings stories are independent and additive on top of Epics 1 & 3. ✅

---

### 🟠 Major Issue 1 — Story 4.1: Contradictory Acceptance Criteria

**Story:** 4.1 Amber Underline Indicator

**Problem:** Two ACs directly contradict each other:
- AC1: `"Then the backend emits a show_correction_overlay IPC event **instead of injecting the text silently**"`
- AC2: `"Given the text is injected into the target app, When confidence is below threshold, Then an amber underline is rendered beneath the flagged word"`

AC1 says text is NOT injected on low confidence. AC2 assumes text IS already injected. These cannot both be true.

**Correct behavior per PRD (Journey 2):** Text IS injected. Then an amber underline appears. The user then taps it to open the correction overlay (FR12 — Story 4.2). The overlay is triggered by user tap, not automatically.

**Remediation required:** Rewrite Story 4.1 ACs to: (1) inject the low-confidence text, (2) show the amber underline indicator over the injected word, (3) the overlay only opens when user taps (handled in Story 4.2). Remove "instead of injecting" from AC1.

---

### 🟠 Major Issue 2 — Story 5.4 / Architecture: Model Download vs. No-HTTP-Client Rule

**Stories affected:** 5.4 (Local Model Download)
**Documents in conflict:** `architecture.md` and `epics.md` Story 5.4

**Problem:** The Architecture enforces "No HTTP client crates (`reqwest`, `hyper`, `ureq`, `surf`, etc.) in `Cargo.toml`" and CI is meant to enforce this. Yet FR30 requires in-app model file downloads, which necessarily involves an HTTP request. Story 5.4 cites both `FR30` and `NFR12` as if they are compatible, but does not resolve the technical contradiction.

NFR12 ("zero outbound network connections in any usage path") was designed for the dictation/transcription usage paths — not for explicit, user-initiated model management. However, the Architecture's blanket "no HTTP crates" rule does not carve out an exception for downloads.

**Remediation required:** The Architecture document and Story 5.4 must be updated with one of these resolutions:
- **Option A (Recommended):** Approve a download-only dependency (e.g., macOS `URLSession` via `objc2` FFI — no new Rust HTTP crate) and document the carve-out explicitly. Update NFR12 to read "zero outbound network connections in all dictation/transcription usage paths; model downloads are the sole exception and are always user-initiated."
- **Option B:** Approve a minimal HTTP crate (e.g., `ureq`) scoped exclusively to `models.rs`, with CI updated to allow only that crate in that module. Enforce via `cargo deny` allow-list.
- **Option C:** Remove in-app download from v1 scope — user downloads model files manually and places them in the `~/Library/Application Support/icantspell/models/` directory. Story 5.4 becomes a "manual model discovery" story only.

---

### 🟡 Minor Concern — Story 3.5: Low-Confidence Behavior Before Epic 4

**Story:** 3.5 End-to-End Voice Pipeline Integration

**Concern:** Story 3.5 specifies the high-confidence path (inject directly when confidence ≥ 0.85). It does not specify what happens when confidence < 0.85 in the context of Epic 3 alone (before Epic 4 adds the overlay). A developer implementing Story 3.5 without Epic 4 context has no specified behaviour for the low-confidence case.

**Remediation:** Add one AC to Story 3.5: "Given transcription confidence is below threshold, When the pipeline runs, Then the text is injected and a `show_correction_overlay` event is emitted — the overlay handling is implemented in Epic 4." This makes the boundary explicit and prevents a developer from guessing.

---

### Quality Summary

| Severity | Count | Items |
|---|---|---|
| 🔴 Critical | 0 | — |
| 🟠 Major | 2 | Story 4.1 AC contradiction; Architecture/FR30 HTTP client conflict |
| 🟡 Minor | 1 | Story 3.5 low-confidence boundary unspecified |

## Summary and Recommendations

### Overall Readiness Status

**NEEDS WORK** — 2 major issues must be resolved before implementation begins. No critical blockers. The foundation is strong.

### Issues Requiring Action Before Implementation

**🟠 Issue 1 — Story 4.1: Contradictory ACs (fix in epics.md)**

The amber underline story has two ACs that contradict each other on whether text is injected before flagging. Based on PRD Journey 2, the correct behaviour is: inject text → show amber underline → user taps to open overlay.

Action: Rewrite Story 4.1 ACs. Remove "instead of injecting" from AC1. Replace with: text is injected AND the amber indicator is shown. Make clear the overlay opens only on user tap (Story 4.2), not automatically.

**🟠 Issue 2 — Architecture / Story 5.4: Model download HTTP conflict (decision required)**

The Architecture bans all HTTP client crates; FR30 requires in-app model downloads. Story 5.4 cites both without resolving the contradiction. A decision is needed before Epic 5 implementation.

Recommended resolution (Option A): Use macOS `URLSession` via `objc2` FFI (already a dependency for sleep/wake) — no new Rust HTTP crate required. Update NFR12 to explicitly carve out user-initiated model downloads. Update Architecture with this decision.

### Recommended Next Steps

1. **Fix Story 4.1** — rewrite the two contradictory ACs in `epics.md` so the amber underline behaviour is unambiguous (inject first, then flag).
2. **Resolve the model download architecture** — pick Option A/B/C from Issue 2 above, record the decision in `architecture.md`, and update Story 5.4's ACs accordingly.
3. **Add one AC to Story 3.5** — specify that low-confidence transcriptions in Epic 3 emit `show_correction_overlay` but that handling is deferred to Epic 4, so the implementation boundary is explicit.
4. **Proceed to Sprint Planning** (`bmad-sprint-planning`) once issues 1 and 2 are resolved.

### Final Note

This assessment identified **3 issues** across **2 categories** (2 major, 1 minor). No critical blockers were found. FR coverage is 100% of in-scope v1 requirements. Epic structure is sound, story dependencies are clean, and the Architecture is well-specified. Resolve the two major issues — both are small, targeted fixes — and this project is ready for implementation.

**Report saved to:** `_bmad-output/planning-artifacts/implementation-readiness-report-2026-04-27.md`
