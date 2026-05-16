# Implementation Plan: Audio Format Compression

**Branch**: `004-audio-format-compression` | **Date**: 2026-05-16 | **Spec**: specs/004-audio-format-compression/spec.md
**Input**: Feature specification from `specs/004-audio-format-compression/spec.md`

## Summary

Make the audio output of recording and monitoring configurable along two axes — **format** and **compression profile** — and read the defaults from a single, human-editable configuration file (`config/audio.conf`). The supported format set in this iteration is `{wav}`; the supported compression set is `{pcm8, pcm16, pcm24, float32}`, all driven through the existing `hound` writer. A new `AudioOutputProfile` value is loaded at TUI startup from the config file (falling back to built-in defaults if the file is missing), validated up front, and threaded through `record::start_recording_thread` and `monitor::start_monitoring_thread` so every saved WAV reflects the active profile. No new crate dependencies are introduced.

## Technical Context

**Language/Version**: Rust 1.85 (edition 2024)
**Primary Dependencies**: cpal 0.15.3, hound 3.5.1, ratatui 0.29.0, crossterm 0.28.1, thiserror 2.0.11 — all unchanged. **No new crates** (custom line-based parser keeps us off `toml`/`serde`).
**Storage**: Local filesystem. New: `./config/audio.conf` for defaults (version-controlled). Unchanged: `./recordings/` for output WAVs.
**Testing**: `cargo test` — unit tests for config parsing, profile validation, and bit-depth-aware WAV writing; integration test for the load-config-and-record path.
**Target Platform**: macOS/Linux terminal emulator
**Project Type**: Terminal application (TUI)
**Performance Goals**: Config load <5 ms (single small file); record/monitor latency unchanged from feature 003.
**Constraints**: Zero new dependencies. Config selection is the *only* user-facing override mechanism in this iteration (no new TUI keybindings). Existing WAV files are immutable and unaffected.
**Scale/Scope**: Single-user local use; one config file; finite, documented enum of supported profiles.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **CA-001 Rust-First** ✅: All new code is pure Rust. No new crates: config parsing is a ~40-line line-based `key=value` parser; bit-depth selection uses `hound`'s existing `WavSpec`/`SampleFormat` API. Justification for not adopting `toml`+`serde`: dependency-minimal principle; the config currently exposes 2 keys.
- **CA-002 CLI-First / TUI-First** ✅ (TUI-First inherited from 002): No new CLI flags, sub-commands, or TUI keybindings. The config file *is* the user-facing surface for selecting the profile. Status messages surface the active profile and any config-load warnings.
- **CA-003 Verification Gate** ✅: Config parsing and profile validation are unit-testable with string slices and synthetic input. Bit-depth-aware WAV writing is unit-testable by reading back the written file's `WavSpec` via `hound::WavReader`. Integration: a small test creates a config file, drives `handle_record` once, asserts the resulting WAV's spec.
- **CA-004 Integration Safety** ✅: `record::start_recording_thread` and `monitor::start_monitoring_thread` gain an `AudioOutputProfile` parameter — a contract change. Both callers in `tui/app.rs` are updated in the same patch. `MonitorConfig` gains an `output_profile` field. Existing tests for monitor segment lifecycle remain green (defaults preserved). New integration tests cover (a) missing config → fallback, (b) invalid config → fail-fast with clear error, (c) valid `pcm24` round-trip.
- **CA-005 Version Discipline** ✅: Behavior is additive (new defaults file, new profile-aware writers). Default profile (`wav + pcm16`) produces files byte-identical to the current writer, so existing recording behavior is preserved when no config file is present. Version bump: v1.1.0 → v1.2.0 (MINOR).

*Post-Phase 1 re-check*: Constitution Check remains satisfied. The compatibility matrix is intentionally permissive (all `(wav, *)` pairs valid), but the validation pipeline rejects unknown identifiers with deterministic messages, which is the spec's enforcement point.

## Project Structure

### Documentation (this feature)

```text
specs/004-audio-format-compression/
├── plan.md                          # This file
├── research.md                      # Phase 0 output
├── data-model.md                    # Phase 1 output
├── quickstart.md                    # Phase 1 output
├── contracts/
│   ├── config-file.md               # Phase 1 output — config file format & error messages
│   └── output-profile.md            # Phase 1 output — supported profiles & writer behavior
├── checklists/
│   └── requirements.md              # (existing)
└── tasks.md                         # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code

```text
src/
├── main.rs              # Unchanged
├── lib.rs               # Extended: pub mod config
├── config/
│   └── mod.rs           # NEW: AudioDefaultsConfig, parse, load_or_default
├── model/
│   └── mod.rs           # Extended: SupportedFormat, CompressionProfile, AudioOutputProfile
├── audio/
│   ├── mod.rs           # Unchanged
│   ├── record.rs        # Extended: signature accepts AudioOutputProfile; WAV spec derived from profile
│   ├── monitor.rs       # Extended: MonitorConfig gains output_profile; write_wav_file honors it
│   └── playback.rs      # Unchanged (already reads spec from file header)
├── error.rs             # Extended: AppError::Config(String) variant
└── tui/
    ├── mod.rs           # Unchanged
    ├── app.rs           # Extended: load config at startup, pass profile to record/monitor threads
    └── view.rs          # Possibly extended: show active profile in footer (minor cosmetic)

config/
└── audio.conf           # NEW: default config file checked into the repo

tests/
├── unit/
│   ├── config_parse.rs              # NEW: parsing valid/invalid lines, comments, missing keys
│   └── profile_validation.rs        # NEW: enum parsing, compatibility matrix, error strings
└── integration/
    └── config_record.rs             # NEW: load → record → assert WavSpec on saved file
```

**Structure Decision**: Single project, same shape as features 001–003. The new `src/config/` module is sibling to `audio/`, `model/`, `tui/`, mirroring how 003 added `audio/monitor.rs`. The default config file lives in `./config/` (root-relative), parallel to `./recordings/`, so users discover it next to runtime artifacts.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| Custom line-based config parser instead of `toml`+`serde` | Stays within the Rust-First dep-minimal principle; current config has 2 keys | `toml` + `serde` + `serde_derive` adds three transitive deps for a ~10-line config; the parser is ~40 lines with unit tests |
| Compatibility matrix returns "valid" for all `(wav, *)` pairs in v1 | Single format means the matrix is trivially permissive | Adding FLAC/MP3 now would introduce a non-trivial codec dependency (`flacenc`, `lame`, etc.) and is explicitly out of scope; the validation pipeline is shaped so incompatible pairs can be rejected the moment a second format is introduced |