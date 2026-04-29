---
stepsCompleted: [1, 2]
inputDocuments: []
session_topic: 'Local voice-to-text (Whisper-style) + system-wide AI autocomplete for macOS — built specifically for people with dyslexia'
session_goals: 'Architecture decisions, local model selection, OS-level input integration, cross-app compatibility — all optimized for dyslexic users'
selected_approach: 'ai-recommended'
techniques_used: ['First Principles Thinking', 'Morphological Analysis', 'Constraint Mapping']
ideas_generated: []
context_file: ''
---

# Brainstorming Session Results

**Project:** ICantSpell
**Facilitator:** Anders
**Date:** 2026-04-27

## Session Overview

**Topic:** Local voice-to-text (Whisper-style) + system-wide AI autocomplete for macOS
**Primary User:** People with dyslexia
**Core Purpose:** Make writing easier and less frustrating for dyslexic users — voice-to-text and autocomplete as accessibility tools, not productivity tools

**Goals:**
- Architecture decisions optimized for low-friction dyslexic UX
- Local model selection (performance, privacy, accuracy for non-standard speech patterns)
- OS-level input integration strategy (macOS Accessibility API, cross-app compatibility)
- UX decisions that reduce cognitive load and frustration for dyslexic users

## Technique Selection

**Approach:** AI-Recommended
**Sequence:** First Principles Thinking → Morphological Analysis → Constraint Mapping

---

## Phase 1: First Principles

**[FP #1]: Flow Paralysis is the Core Problem**
*Concept:* The root harm isn't spelling difficulty — it's the freeze that happens when a user can see a word is wrong but lacks the spelling knowledge to fix it. The thought evaporates mid-sentence.
*Novelty:* Reframes the app from "spelling helper" to "freeze preventer."

**[FP #2]: Voice Eliminates the Entire Problem Category**
*Concept:* Speech-to-text bypasses spelling entirely. The user never produces a broken word — they speak correctly and the transcript is clean. Voice-first isn't a convenience feature; it's the core accessibility intervention.
*Novelty:* Voice is the primary therapeutic intervention, not a power-user feature.

**[FP #3]: Autocomplete is a Voice-Unavailable Safety Net**
*Concept:* When voice isn't possible (meetings, public, quiet spaces), autocomplete bridges the gap — completing approximate phonetic input before the user freezes on a broken word.
*Novelty:* Autocomplete should be phonetically-driven AND context-aware, not prefix-matching.

**[FP #4]: Whisper Errors Need Flagging Without Friction**
*Concept:* When Whisper mishears and produces a plausible-wrong word, the user must know — but correction must be low-friction (tap → re-dictate or alternatives), or it recreates the original freeze.
*Novelty:* Error correction as a one-tap interaction, never keyboard-driven.

**[FP #5]: The Pause is the Trigger**
*Concept:* Autocomplete activates on a short typing pause (~400–600ms). The pause is the natural moment the user already suspects something is off — the app meets them there.
*Novelty:* Uses the user's hesitation rhythm as the input signal, not a hotkey or explicit request.

**[FP #6]: Correction is Context-Aware, Not Just Phonetic**
*Concept:* Dyslexic users frequently produce contextually wrong words — "from/form," "there/their/they're" — that are spelled correctly but semantically wrong. The model must read surrounding context.
*Novelty:* Pushes the requirement from spell-checker to lightweight language model with context window.

**[FP #7]: Voice Mode and Typing Mode are Architecturally Separate**
*Concept:* Two distinct, non-overlapping pipelines. Voice mode: mic → Whisper.cpp → flagged transcript. Typing mode: keystrokes → pause detection → hybrid model → floating pill. They share a text output target but nothing else.
*Novelty:* Simplifies architecture dramatically — each pipeline designed and tested independently.

**[FP #8]: Privacy is a Core Trust Promise, Not a Setting**
*Concept:* "Your text never leaves your device" is a founding guarantee. Zero network calls, zero telemetry. For self-conscious dyslexic users, on-device processing is psychological safety.
*Novelty:* Rules out any architecture path that requires network access, including crash reporting and model updates via cloud.

---

## Phase 2: Morphological Analysis — Locked Decisions

| Dimension | Decision |
|-----------|----------|
| STT Model | Whisper.cpp via `whisper-rs` |
| Rule Layer | Curated dyslexia error dictionary (there/their, from/form, etc.) |
| LLM Layer | DeBERTa-v3-small — 180MB, ONNX-native, context-aware |
| Inference Runtime | ONNX Runtime (cross-platform model compatibility) |
| Voice Trigger | Push-to-talk hotkey (default) |
| Privacy | 100% local, hard requirement — no network calls, no telemetry |
| Platform/Runtime | Tauri (Rust + webview) — macOS first, Windows path preserved |
| OS Integration | AXUIElement (primary) + CGEventTap (pause detection) |
| Whisper Error UX | Soft amber underline + tap → re-dictate or pick alternatives |
| Autocomplete UX | Floating pill overlay, 2–3 options, click or number key, Esc/continue to dismiss |
| App Presence | Menu bar only, no Dock icon, settings window hidden, onboarding wizard on first launch |

### Emerging Architecture

```
ICantSpell — Core Architecture

┌─────────────────────────────────────────────────┐
│  Tauri App (Rust backend + webview UI)          │
│                                                 │
│  ┌─────────────┐    ┌──────────────────────┐   │
│  │ Voice Mode  │    │    Typing Mode       │   │
│  │             │    │                      │   │
│  │ PTT hotkey  │    │ CGEventTap           │   │
│  │     ↓       │    │ (pause detection)    │   │
│  │ whisper-rs  │    │     ↓                │   │
│  │ (Whisper.cpp│    │ AXUIElement          │   │
│  │  via ONNX)  │    │ (read text context)  │   │
│  │     ↓       │    │     ↓                │   │
│  │ Transcript  │    │ Hybrid corrector     │   │
│  │ + confidence│    │ (dict → DeBERTa)     │   │
│  │     ↓       │    │     ↓                │   │
│  │ Amber flags │    │ Floating pill        │   │
│  │ + overlay   │    │ overlay window       │   │
│  │   window    │    │ (always-on-top)      │   │
│  └─────────────┘    └──────────────────────┘   │
│                                                 │
│  Shared: ONNX Runtime · 100% local · Menu bar  │
└─────────────────────────────────────────────────┘
```
