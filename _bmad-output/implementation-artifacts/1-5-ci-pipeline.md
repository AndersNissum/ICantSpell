# Story 1.5: CI Pipeline

Status: review

## Story

As a developer,
I want a GitHub Actions workflow that lints and tests the codebase on every push and PR,
so that regressions are caught automatically and code quality is enforced.

## Acceptance Criteria

1. **Given** `.github/workflows/ci.yml` exists, **When** a commit is pushed to `main` or a pull request is opened, **Then** the workflow triggers automatically.

2. **Given** the workflow runs, **When** `cargo clippy --all-targets -- -D warnings` executes, **Then** any clippy warning causes the build to fail.

3. **Given** the workflow runs, **When** `cargo test` executes, **Then** all unit and integration tests must pass for the workflow to succeed.

4. **And** the workflow completes successfully on the initial scaffold without requiring additional code changes.

## Tasks / Subtasks

- [x] Task 1: Create `.github/workflows/ci.yml` (AC: 1, 2, 3, 4)
  - [x] Create `.github/workflows/` directory at project root
  - [x] Write workflow with `push: branches: [main]` and `pull_request` triggers
  - [x] Use `macos-latest` runner (macOS-only app; `macos-private-api` Tauri feature requires macOS to compile)
  - [x] Set `defaults.run.working-directory: src-tauri` so all `run` steps execute in the Cargo workspace
  - [x] Add `actions/checkout@v4` step
  - [x] Add `dtolnay/rust-toolchain@stable` step with `components: clippy`
  - [x] Add `actions/cache@v4` step caching `~/.cargo/registry`, `~/.cargo/git`, and `src-tauri/target`
  - [x] Add clippy step: `cargo clippy --all-targets -- -D warnings`
  - [x] Add test step: `cargo test`

- [x] Task 2: Validate locally (AC: 2, 3, 4)
  - [x] Run `cargo clippy --all-targets -- -D warnings` from `src-tauri/` — must be zero warnings/errors
  - [x] Run `cargo test` from `src-tauri/` — all 7 existing tests must pass

## Dev Notes

### This Story Creates One New File

This story creates exactly one file: `.github/workflows/ci.yml` at the project root. No Rust source changes, no `Cargo.toml` changes.

### Complete `ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  check:
    name: Lint and Test
    runs-on: macos-latest
    defaults:
      run:
        working-directory: src-tauri

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            src-tauri/target
          key: ${{ runner.os }}-cargo-${{ hashFiles('src-tauri/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Test
        run: cargo test
```

### Why `macos-latest` (Critical)

**Do NOT use `ubuntu-latest`.** The project has `tauri = { features = ["macos-private-api", ...] }` in `Cargo.toml`. The `macos-private-api` feature gates macOS-specific code that won't compile on Linux. Future stories also add `AXUIElement`, `CGEventTap`, `NSWorkspace`, and `objc2` — all macOS-only. Using any non-macOS runner will cause compilation failures.

### Why `defaults.run.working-directory: src-tauri`

The `Cargo.toml` lives in `src-tauri/`, not the project root. Every `cargo` command must run from there. The `defaults.run.working-directory` applies to all `run` steps in the job, so you don't need `working-directory` on each individual step.

### Cache Key Strategy

The cache key uses `hashFiles('src-tauri/Cargo.lock')` — this invalidates the cache whenever dependencies change. The `restore-keys` fallback (`${{ runner.os }}-cargo-`) allows partial cache hits when only some packages changed.

Cache paths:
- `~/.cargo/registry` — downloaded crate source
- `~/.cargo/git` — git-sourced dependencies (none currently, but good practice)
- `src-tauri/target` — compiled artifacts (biggest win for CI speed)

### Current Codebase State

When the CI runs for the first time, it will compile and test exactly what exists now:
- `src-tauri/src/lib.rs` — Tauri builder, tray, config setup
- `src-tauri/src/config.rs` — Settings struct, load/save/ensure_defaults, 5 unit tests
- `src-tauri/src/error.rs` — AppError enum (thiserror)
- `src-tauri/src/stt/mod.rs` — SpeechToText trait + TranscriptionResult, 2 unit tests
- All other `*.rs` files are comment stubs — they compile cleanly

Expected test output (7 tests, all pass):
```
running 7 tests
test config::tests::test_default_confidence_threshold ... ok
test config::tests::test_default_ptt_hotkey_is_empty ... ok
test config::tests::test_default_selected_model ... ok
test config::tests::test_settings_json_roundtrip ... ok
test config::tests::test_settings_roundtrip_with_custom_values ... ok
test stt::tests::test_transcription_result_empty_alternatives ... ok
test stt::tests::test_transcription_result_fields ... ok
test result: ok. 7 passed; 0 failed
```

### What CI Does NOT Include (Deferred)

Per architecture, these are deferred/nice-to-have and NOT part of this story:
- `cargo deny` — HTTP crate enforcement (deferred per architecture Gap Analysis)
- Release/build pipeline — no `cargo tauri build` in CI (personal use, manual builds)
- Code signing / notarization — not for v1
- Frontend linting — no JS/TS toolchain needed for vanilla JS stubs

Do NOT add these to the workflow file.

### Project Structure Notes

File created:
- `.github/workflows/ci.yml` — NEW (creates `.github/` and `workflows/` directories at project root)

File NOT touched:
- Everything in `src-tauri/` — no changes needed
- Everything in `src/` — no changes needed

### References

- [Source: epics.md § Story 1.5] — Acceptance criteria and user story
- [Source: architecture.md § Infrastructure & Deployment] — "GitHub Actions: `cargo clippy --all-targets -- -D warnings` + `cargo test` on push to main and PRs"
- [Source: architecture.md § Privacy Boundary] — `cargo deny` for HTTP crate enforcement is deferred (nice-to-have per Gap Analysis)
- [Source: architecture.md § Enforcement Guidelines] — "Never add HTTP client crates" (CI doesn't enforce this yet; deferred to cargo deny)
- [Source: story 1-4 Completion Notes] — `macos-private-api` Tauri feature added in Story 1.4; dictates macOS runner requirement

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — implementation matched spec exactly, no issues encountered.

### Completion Notes List

- Created `.github/workflows/ci.yml` exactly as specified in Dev Notes
- `cargo clippy --all-targets -- -D warnings` passed with zero warnings (Finished dev profile)
- `cargo test` passed: 7 tests, 0 failed — matches expected output in Dev Notes
- No Rust source changes required; workflow file is the only deliverable

### File List

- `.github/workflows/ci.yml` — NEW

## Change Log

- 2026-04-29: Story 1.5 created — CI Pipeline ready for dev.
- 2026-04-29: Story 1.5 implemented — CI Pipeline complete, status → review.
