# Story 3.2: Microphone Audio Capture Pipeline

Status: done

## Story

As a user,
I want audio to be captured from my microphone only while I hold the PTT hotkey,
so that there is no background listening and my privacy is protected.

## Acceptance Criteria

1. **Given** `audio.rs` owns a dedicated `std::thread::spawn` thread for the `cpal` stream,
   **When** the PTT keydown signal arrives via `std::sync::mpsc` channel (carrying a `CaptureCommand::Start` variant),
   **Then** mic capture starts and audio samples are written into an in-memory `Vec<f32>` buffer.

2. **Given** the PTT keyup signal arrives as a `CaptureCommand::Stop` on the same mpsc channel,
   **When** capture stops,
   **Then** the completed `Vec<f32>` buffer is sent to the STT pipeline via a separate output channel and no copy remains in `audio.rs`.

3. **Given** the PTT hotkey is not held,
   **When** the app is running at idle,
   **Then** no audio is captured and CPU usage attributable to audio is effectively zero (NFR10).

4. **Given** the audio buffer is passed to the STT pipeline,
   **When** transcription completes (success or failure),
   **Then** the buffer is dropped from memory — not written to disk at any point (NFR13/FR28).

5. **And** the mpsc channel supports repeated Start/Stop cycles across multiple PTT presses without requiring channel recreation.

## Tasks / Subtasks

- [x] Task 1: Add `cpal` dependency to Cargo.toml (AC: 1)
  - [x] 1.1 Add `cpal = "0.15"` to `[dependencies]` in `src-tauri/Cargo.toml`
  - [x] 1.2 Verify `cargo check` succeeds with the new dependency

- [x] Task 2: Define audio module types and command enum (AC: 1, 2, 5)
  - [x] 2.1 Define `CaptureCommand` enum: `Start`, `Stop`
  - [x] 2.2 Define `AudioPipeline` struct to hold command sender and audio output receiver
  - [x] 2.3 Define type aliases: `CaptureCommandSender`, `CaptureCommandReceiver`, `AudioBufferSender`, `AudioBufferReceiver`
  - [x] 2.4 Write unit tests for CaptureCommand enum construction

- [x] Task 3: Implement audio capture thread with cpal stream (AC: 1, 3)
  - [x] 3.1 Implement `start_audio_pipeline()` that spawns a named `"audio-capture"` thread
  - [x] 3.2 In the thread: get default input device via `cpal::default_host().default_input_device()`
  - [x] 3.3 Query device's default input config; determine if resampling to 16kHz is needed
  - [x] 3.4 Implement the capture loop: recv `CaptureCommand` on mpsc, on `Start` → build+play cpal input stream writing samples to `Vec<f32>`, on `Stop` → drop stream, send buffer out
  - [x] 3.5 Handle sample format conversion: cpal callback receives device-native format, convert to f32
  - [x] 3.6 Resample captured audio from device sample rate to 16000 Hz (Whisper requirement) using linear interpolation
  - [x] 3.7 Ensure the thread loops back to wait for next `Start` after sending buffer (AC: 5)

- [x] Task 4: Wire audio pipeline into app startup in lib.rs (AC: 1, 2)
  - [x] 4.1 Replace `_ptt_rx` discard with actual wiring: pass `PttReceiver` to a bridge that translates `PttEvent` → `CaptureCommand`
  - [x] 4.2 Start audio pipeline thread, store `AudioBufferReceiver` for future STT consumption (Story 3.3)
  - [x] 4.3 Spawn a bridge thread that receives `PttEvent::Pressed` → sends `CaptureCommand::Start`, `PttEvent::Released` → sends `CaptureCommand::Stop`

- [x] Task 5: Implement idle zero-CPU behavior (AC: 3)
  - [x] 5.1 Ensure no cpal stream exists when not capturing — stream is created on Start and dropped on Stop
  - [x] 5.2 Audio thread blocks on `mpsc::Receiver::recv()` when idle — zero CPU spin

- [x] Task 6: Ensure buffer is never written to disk (AC: 4)
  - [x] 6.1 Verify no `std::fs::write`, `File::create`, or any disk I/O in audio.rs
  - [x] 6.2 Buffer ownership transfers via channel `.send()` — `audio.rs` retains no reference after Stop

- [x] Task 7: Write unit and integration tests (AC: 1-5)
  - [x] 7.1 Unit test: CaptureCommand enum variants exist and are constructible
  - [x] 7.2 Unit test: `resample_to_16khz` correctly resamples from 48kHz to 16kHz (verify output length = input_length * 16000 / 48000)
  - [x] 7.3 Unit test: `resample_to_16khz` is identity when source is already 16kHz
  - [x] 7.4 Unit test: `convert_samples_to_f32` handles i16 and f32 sample formats
  - [x] 7.5 Integration test: `AudioPipeline` can be created, Start/Stop commands sent, buffer received (mock device or skip if no mic)

## Dev Notes

### Architecture Requirements

- **Module:** `src-tauri/src/audio.rs` — replace the 4-line placeholder stub
- **Thread model:** Dedicated `std::thread::spawn` named `"audio-capture"` — do NOT use `tokio::spawn` or `tokio::task::spawn_blocking`. cpal's audio callback runs on its own internal thread; the capture thread manages stream lifecycle.
- **Channel architecture:**
  - Input: `std::sync::mpsc::Receiver<CaptureCommand>` — receives Start/Stop from PTT bridge
  - Output: `std::sync::mpsc::Sender<Vec<f32>>` — sends completed audio buffer to STT pipeline
- **Buffer type:** `Vec<f32>` — linear buffer, NOT a ring buffer. Pre-allocate with `Vec::with_capacity(16000 * 30)` for up to 30 seconds at 16kHz.
- **Privacy:** Audio buffer exists only in memory. Ownership transfers via channel. No disk writes. No logging of audio content.

### Sample Rate Resampling — CRITICAL

**Previous project lesson:** A prior app failed to capture usable audio on macOS because the device sample rate (typically 48kHz on macOS CoreAudio) was passed directly to Whisper, which expects 16kHz. This caused garbled/silent transcription.

**Required approach:**
1. Query device default config: `device.default_input_config()` → get `sample_rate`
2. Capture at device-native rate (do NOT request 16kHz from cpal — CoreAudio may reject non-native rates)
3. After capture stops, resample the buffer from device rate → 16000 Hz before sending to STT
4. Use linear interpolation for resampling — sufficient quality for speech, zero external dependencies
5. Implement as `fn resample_to_16khz(samples: &[f32], source_rate: u32) -> Vec<f32>`
6. If device is already 16kHz, return samples unchanged (no-op path)

### cpal Usage Pattern (v0.15)

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

let host = cpal::default_host();
let device = host.default_input_device().ok_or(AppError::Audio("No input device".into()))?;
let config = device.default_input_config()?;
let sample_rate = config.sample_rate().0;  // e.g., 48000
let sample_format = config.sample_format(); // e.g., SampleFormat::F32

// Build stream — callback pushes samples into shared buffer
let stream = device.build_input_stream(
    &config.into(),
    move |data: &[f32], _: &cpal::InputCallbackInfo| {
        // Push samples to buffer (behind Arc<Mutex<Vec<f32>>>)
    },
    move |err| { tracing::error!("Audio stream error: {}", err); },
    None, // no timeout
)?;
stream.play()?;
// ... on Stop: drop(stream), take buffer, resample, send
```

**Sample format handling:** Default config may return `I16` or `F32`. Use `cpal::SampleFormat` match to build the correct typed stream. Convert to f32 in the callback using `cpal::Sample::to_float_sample()`.

### Existing Code Integration Points

**lib.rs current state (lines 89-97):**
```rust
let settings = config::load(app.handle());
let (ptt_tx, _ptt_rx) = std::sync::mpsc::channel();
if !settings.ptt_hotkey.is_empty() {
    hotkey::start_hotkey_listener(&settings.ptt_hotkey, ptt_tx)
        .unwrap_or_else(|e| tracing::error!("..."));
} else {
    tracing::info!("No PTT hotkey configured, skipping listener");
}
```

**Required changes:**
1. Remove `_ptt_rx` — replace with `ptt_rx` (used)
2. Call `audio::start_audio_pipeline()` which returns `(CaptureCommandSender, AudioBufferReceiver)`
3. Spawn bridge thread: `PttEvent::Pressed` → `CaptureCommand::Start`, `PttEvent::Released` → `CaptureCommand::Stop`
4. Store `AudioBufferReceiver` as `_audio_rx` for now — Story 3.3 will consume it

**hotkey.rs types to use:**
- `PttEvent::Pressed`, `PttEvent::Released` — from `hotkey.rs` (Story 3.1)
- `PttSender` = `std::sync::mpsc::Sender<PttEvent>`
- `PttReceiver` = `std::sync::mpsc::Receiver<PttEvent>`

**error.rs already has:** `AppError::Audio(String)` variant — use this for all audio errors.

### What NOT To Do

- Do NOT use `tokio::sync::mpsc` — the epics mention it but Story 3.1 established `std::sync::mpsc` as the pattern. Stay consistent.
- Do NOT use `tokio::spawn` or `tokio::task::spawn_blocking` for the audio thread — use `std::thread::spawn`
- Do NOT request a specific sample rate from cpal — capture at device-native rate and resample after
- Do NOT add `whisper-rs` or implement transcription — that's Story 3.3
- Do NOT write audio data to disk, temp files, or logs
- Do NOT add any Tauri commands (`#[tauri::command]`) in audio.rs
- Do NOT modify `hotkey.rs` — use its existing `PttEvent`/`PttReceiver` types as-is
- Do NOT add ring buffer or circular buffer — use simple `Vec<f32>`
- Do NOT add external resampling crates (rubato, dasp) — linear interpolation is sufficient for speech

### Naming Conventions

- Functions: `snake_case` — `start_audio_pipeline`, `resample_to_16khz`
- Types: `PascalCase` — `CaptureCommand`, `AudioPipeline`
- Constants: `SCREAMING_SNAKE_CASE` — `WHISPER_SAMPLE_RATE`
- Module: `audio.rs` (already exists as placeholder)

### Testing Strategy

- **Unit tests:** Inline `#[cfg(test)] mod tests` in `audio.rs`
  - Test resampling math (deterministic, no hardware needed)
  - Test sample format conversion
  - Test CaptureCommand construction
- **Integration tests:** Hardware-dependent tests should be `#[ignore]` by default
  - Use `#[test] #[ignore]` for tests requiring a real microphone
  - CI runs `cargo test` (skips ignored), local dev runs `cargo test -- --include-ignored`
- **Regression:** All 20 existing tests must continue to pass

### Project Structure Notes

- `audio.rs` is already declared as `pub mod audio;` in `lib.rs` (line ~4)
- Placement follows architecture: `src-tauri/src/audio.rs`
- No new module files needed — everything goes in `audio.rs`

### References

- [Source: _bmad-output/planning-artifacts/architecture.md § Audio Pipeline Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md § cpal Audio Stream Lifecycle]
- [Source: _bmad-output/planning-artifacts/epics.md § Epic 3, Story 3.2]
- [Source: _bmad-output/planning-artifacts/prd.md § FR1, FR2, FR4, FR28, NFR10, NFR13]
- [Source: _bmad-output/implementation-artifacts/3-1-global-ptt-hotkey-registration.md § Dev Notes, Thread Model]
- [Source: MEMORY — Audio sample rate mismatch risk from previous project]

### Review Findings

- [x] [Review][Decision] **Mono downmix for multi-channel input devices** — Applied: extract channel 0 from interleaved frames in callback using `data.chunks(channels).map(|frame| frame[0])`. [audio.rs] (blind+edge+auditor)

- [x] [Review][Decision] **`_audio_rx` dropped at end of setup closure** — Applied: store in Tauri managed state via `app.manage(Mutex::new(Some(audio_rx)))` for Story 3.3 retrieval. [lib.rs] (auditor+edge)

- [x] [Review][Patch] **`convert_i16_to_f32` asymmetric scaling** — Applied: divide by `32768.0f32` for symmetric [-1.0, 1.0] range. [audio.rs:68, audio.rs:170] (blind+edge)

- [x] [Review][Patch] **Unbounded buffer growth on long PTT holds** — Applied: added `MAX_CAPTURE_SECONDS = 120` constant, callback checks `buf.len() >= max` and stops extending. [audio.rs] (blind+edge)

- [x] [Review][Patch] **Mutex poisoning silently drops all audio** — Applied: `Err` branch now logs at `warn!` level. [audio.rs] (blind+edge)

- [x] [Review][Patch] **`panic!` on audio pipeline startup failure** — Applied: returns `Ok(())` early with error log instead of panicking. Bridge thread failure also no longer panics. [lib.rs] (blind)

- [x] [Review][Patch] **`command_tx` dropped when no hotkey configured** — Applied: stored in Tauri managed state via `app.manage(Mutex::new(Some(capture_tx)))` so pipeline thread stays alive. [lib.rs] (edge+auditor)

- [x] [Review][Defer] **Linear interpolation resampler has no anti-alias filter** — 3:1 downsampling without low-pass pre-filter introduces aliasing. Real impact on Whisper accuracy is uncertain. Acceptable for v1; consider `rubato` crate if transcription quality suffers. [audio.rs:37-60] — deferred, acceptable for v1 per story spec

- [x] [Review][Defer] **No device-change handling for hot-unplugged USB mics** — If default device is disconnected mid-capture, stream error callback logs but capture loop blocks forever. No timeout on recv. [audio.rs:199] — deferred, edge case for v1

- [x] [Review][Defer] **Additional sample format support (U16, I32, F64)** — Only F32 and I16 handled. Some USB devices report other formats. [audio.rs:153-190] — deferred, uncommon on macOS CoreAudio

## Dev Agent Record

### Agent Model Used

claude-opus-4-6

### Debug Log References

No issues — all tasks implemented cleanly without debug iterations.

### Completion Notes List

- Added `cpal = "0.15"` dependency to Cargo.toml. cpal v0.15.3 resolved with CoreAudio backend for macOS.
- Replaced audio.rs 4-line placeholder with full module: `CaptureCommand` enum, `AudioPipeline` struct, type aliases for all channel types, `WHISPER_SAMPLE_RATE` constant.
- Implemented `start_audio_pipeline()` — spawns named `"audio-capture"` thread with `audio_capture_loop` that blocks on mpsc recv when idle (zero CPU).
- `capture_until_stop()` — opens default input device, builds typed cpal input stream (handles F32 and I16 sample formats), captures to `Arc<Mutex<Vec<f32>>>`, drops stream on Stop.
- `resample_to_16khz()` — linear interpolation resampler from device-native rate (typically 48kHz) to 16kHz for Whisper. Identity pass-through when source is already 16kHz. Addresses previous project sample rate mismatch issue.
- `convert_i16_to_f32()` — converts i16 samples to f32 in [-1.0, 1.0] range.
- Wired audio pipeline into lib.rs: `start_audio_pipeline()` called in setup, PTT-audio bridge thread spawned translating `PttEvent::Pressed/Released` → `CaptureCommand::Start/Stop`. `AudioBufferReceiver` stored as `_audio_rx` for Story 3.3 consumption.
- 10 new unit tests added (CaptureCommand variants, resampling math at 48kHz/44.1kHz/16kHz/empty, i16→f32 conversion, channel round-trips, value range preservation). 30/30 tests pass total. Zero clippy warnings.

### Change Log

- 2026-05-01: Implemented Story 3.2 — Microphone audio capture pipeline via cpal. CaptureCommand/AudioPipeline types, start_audio_pipeline() with dedicated thread, resample_to_16khz() for Whisper compatibility, PTT-audio bridge wiring. 30/30 tests pass, zero clippy warnings.

### File List

- src-tauri/Cargo.toml — added `cpal = "0.15"` dependency
- src-tauri/src/audio.rs — replaced 4-line stub with full module (CaptureCommand, AudioPipeline, start_audio_pipeline, capture_until_stop, resample_to_16khz, convert_i16_to_f32, 10 unit tests)
- src-tauri/src/lib.rs — added `pub mod audio;` declaration + audio pipeline startup + PTT-audio bridge thread wiring
