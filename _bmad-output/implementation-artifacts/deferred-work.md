# Deferred Work

## Deferred from: code review of 1-2-core-error-types-and-stt-trait-interface (2026-04-28)

- `unwrap()` on `default_window_icon()` panics if icon missing [lib.rs:22] — pre-existing from Story 1.1; convert to `?`-propagated error in a future lib.rs cleanup
- `tracing_subscriber::fmt::init()` silently discards error [lib.rs:10] — pre-existing from Story 1.1; switch to `try_init()` or handle the `Result`
- `on_menu_event` silently drops non-quit events [lib.rs:28] — pre-existing from Story 1.1; add logging or `unreachable!()` guard as menu grows
- `_tray` local variable may not keep tray icon alive [lib.rs:24] — pre-existing from Story 1.1; verify Tauri ownership semantics and store in managed state if needed
- `app.exit(0)` performs no orderly shutdown [lib.rs:30] — pre-existing from Story 1.1; add teardown hook when audio capture pipeline exists
- macOS-only `ActivationPolicy::Accessory` — no cross-platform equivalent [lib.rs:17] — pre-existing from Story 1.1; document or address in cross-platform story
- `confidence: f32` — no NaN/Inf/range guard [stt/mod.rs:6] — spec defines bare `f32`; consider newtype or validated constructor when confidence is used in comparisons
- `SttError` missing I/O, model-load, and timeout variants [error.rs] — spec defines exactly the required variants; expand as new failure modes emerge in Stories 3.x
- `AppError` variants use stringly-typed payloads [error.rs] — intentional per spec; revisit with structured inner types if programmatic error handling is needed
- `alternatives: Vec<String>` — no length bound or ordering contract [stt/mod.rs:7] — spec defines `Vec<String>`; document ordering and add cap when Whisper backend is implemented (Story 3.3)
- `EmptyBuffer` not enforced at trait boundary [stt/mod.rs:10] — trait is an interface; ensure concrete `impl SpeechToText` in Story 3.3 performs the check
- `&self` forces interior mutability on implementors with no guidance [stt/mod.rs:10] — intentional synchronous design; document expected `Arc<Mutex<...>>` usage pattern in Story 3.3 dev notes
- No test for `From<SttError>` → `AppError::Stt` conversion [stt/mod.rs] — story only required `TranscriptionResult` field test; add conversion test in Story 3.3 when `SttError` is first propagated

## Deferred from: code review of 1-3-configuration-persistence-foundation (2026-04-28)

- `ensure_defaults` unconditional save risks overwriting data on transient load failure [config.rs:57] — spec-defined behavior; data-loss path requires load-fail AND save-succeed simultaneously; revisit if store reliability issues surface
- `confidence_threshold: f32` — no range/NaN/Inf validation [config.rs:12] — spec defines bare `f32`; add bounds check (e.g., clamp to [0.0, 1.0]) when downstream comparisons are introduced in Story 3.5
- `selected_model` — no allowlist validation [config.rs:11] — Story 3.3 should validate the model name against discovered/bundled models before use
- `ptt_hotkey` empty-string default accepted as valid saved state [config.rs:17] — Story 3.1 should guard against empty string before attempting hotkey registration
- No `#[serde(deny_unknown_fields)]` or schema version field [config.rs:8] — consider adding a `version: u32` field and migration logic before any Settings schema change
- No `[profile.release]` hardening in `Cargo.toml` — add `panic = "abort"` and strip settings in a CI/release story
- `ptt_hotkey` absent from `ensure_defaults` structured log [config.rs:59] — minor observability gap; add `hotkey = %settings.ptt_hotkey` to the tracing::info! call in a future cleanup
- Concurrent `save` calls — no synchronization [config.rs:39] — no concurrent commands yet; ensure `save` is called only from one context or add a Mutex when Tauri commands are introduced
- `load` store-open failure indistinguishable from first-run [config.rs:25] — if UI needs to show "could not load settings" state, load must return a `Result` rather than silently defaulting
- No test for `confidence_threshold` boundary values (NaN, <0, >1) — add edge-value deserialization tests when confidence validation is introduced
- No test for malformed JSON store (wrong field types) — add a test that verifies the `unwrap_or_else` fallback path in `load` when deserialization fails

## Deferred from: code review of 1-4-three-window-tauri-architecture (2026-04-29)

- CSP null disables all Content Security Policy protections [src-tauri/tauri.conf.json] — pre-existing; set a restrictive CSP (e.g., `default-src 'self'`) before any user content is rendered (Epic 4+)
- `withGlobalTauri: true` exposes full Tauri API to all windows [src-tauri/tauri.conf.json] — pre-existing; evaluate removing or scoping when IPC commands are formalized
- Redundant `defer` attribute on `type="module"` script in index.html [src/index.html:8] — pre-existing; remove `defer` for consistency with overlay.html and onboarding.html
- Overlay transparency defeated by shared styles.css opaque background [src/styles.css] — explicitly deferred to Story 4.2; overlay body/html must set `background: transparent`
- No meta CSP fallback in any HTML file — pre-existing; add `<meta http-equiv="Content-Security-Policy">` as defense-in-depth when CSP config is addressed
- `macOSPrivateApi: true` present in tauri.conf.json but absent from Story 1.4 spec — documentation discrepancy from prior story; update spec if accuracy matters for future reference

## Deferred from: code review of 2-6-permission-revocation-monitoring (2026-04-30)

- Monitor thread (`start_permission_monitor`) has no cancellation/shutdown mechanism — spawns infinite loop with no exit condition or `JoinHandle`; `std::process::exit()` terminates threads today but graceful shutdown should be added when a teardown/cleanup story is created [src-tauri/src/permissions.rs]

## Deferred from: code review of 2-5-permission-validation-and-onboarding-completion (2026-04-30)

- Permission monitor thread (`start_permission_monitor`) has no shutdown/cancellation mechanism — spawns detached `std::thread` with infinite loop, no `JoinHandle` stored, no way to stop on app quit [src-tauri/src/permissions.rs] — Story 2.6 scope
- `PermissionChangedPayload` struct and `test_permission_changed_payload_fields` test are Story 2.6 code present in working tree — verify test count alignment when 2.6 is reviewed
- Event names `permission_revoked`/`permission_restored` use snake_case — confirm JS listener convention matches when Story 2.6 frontend wiring is implemented

## Deferred from: code review of 3-1-global-ptt-hotkey-registration (2026-04-30)

- No shutdown/re-registration mechanism for hotkey listener — changing hotkey in settings requires app restart; `rdev::listen` thread has no cancellation. Add live-reload in a future settings story. [src-tauri/src/hotkey.rs]
- `rdev::listen` failure has no recovery path — if CGEventTap is invalidated (e.g., permission revoked at runtime), thread exits with no retry or user notification. Story 3.5 integration should handle graceful degradation. [src-tauri/src/hotkey.rs:193]

## Deferred from: code review of 2-2-accessibility-permission-request (2026-04-29)

- FFI `bool` vs `u8` for `AXIsProcessTrusted` return type [src-tauri/src/permissions.rs:8] — macOS `Boolean` is `unsigned char`, not C99 `_Bool`; declaring as `-> bool` in extern "C" relies on Apple always returning 0/1. Spec prescribes this declaration. Revisit if UB concerns arise.
- `is_first_launch` and onboarding show logic is Story 2.1 scope [src-tauri/src/lib.rs] — first-launch detection and onboarding window show were implemented alongside 2.2 but belong to Story 2.1.
- Pre-existing `.expect()` in `run()` [src-tauri/src/lib.rs] — `expect("error while running tauri application")` is technically unwrap outside tests. Pre-existing from Story 1.1.
- `is_first_launch` checks `settings.json` vs tauri-plugin-store actual on-disk path [src-tauri/src/lib.rs] — verify that `tauri-plugin-store` writes exactly `settings.json` (not a variant like `.settings.json.dat`). Story 2.1 scope.

## Deferred from: code review of 3-2-microphone-audio-capture-pipeline (2026-05-01)

- Linear interpolation resampler has no anti-alias filter — 3:1 downsampling without low-pass pre-filter introduces aliasing artifacts. Acceptable for v1; consider `rubato` crate if transcription quality suffers. [src-tauri/src/audio.rs:37-60]
- No device-change handling for hot-unplugged USB mics — if default device is disconnected mid-capture, stream error callback logs but capture loop blocks forever waiting for Stop command. No timeout on recv. [src-tauri/src/audio.rs:199]
- Additional sample format support (U16, I32, F64) — only F32 and I16 handled; some USB devices report other formats. Uncommon on macOS CoreAudio. [src-tauri/src/audio.rs:153-190]
