# Research: Audio Format Compression

**Feature**: 004-audio-format-compression
**Date**: 2026-05-16

This document resolves the technical unknowns called out by the feature spec and locks in the decisions that drive Phase 1 design.

---

## R-001 — How will "compression" be expressed without new codec dependencies?

**Decision**: Express compression as a **WAV PCM bit-depth/sample-format profile** using only the existing `hound` crate. Supported profiles:

| Profile id  | `hound::SampleFormat` | `bits_per_sample` | Bytes/sample (mono) |
|-------------|----------------------|-------------------|---------------------|
| `pcm8`      | `Int`                | 8                 | 1                   |
| `pcm16`     | `Int`                | 16                | 2                   |
| `pcm24`     | `Int`                | 24                | 3                   |
| `float32`   | `Float`              | 32                | 4                   |

**Rationale**: Achieves the spec goal of "selectable compression profile" by changing file size and fidelity, with **zero new dependencies**. Existing capture pipeline already produces `i16` samples; profile-aware writing happens in the WAV writer step and is straightforward:

- `pcm16` → write `i16` directly (current behavior; bitwise identical to today's output).
- `pcm8` → scale `i16` → `u8` and store as 8-bit unsigned PCM (per WAV convention).
- `pcm24` → sign-extend `i16` → `i32` and use `bits_per_sample = 24`.
- `float32` → `i16` / `i16::MAX as f32`, `SampleFormat::Float`, `bits_per_sample = 32`.

**Alternatives considered**:

- **FLAC** (e.g., `flacenc`): true lossless compression with measurable size savings. Rejected — adds a non-trivial dependency, requires either FFI bindings or a pure-Rust encoder, and pulls codec complexity into the audio path. Will be revisited when a second format is genuinely required.
- **MP3 / AAC / Opus** (lossy): rejected — license complications (`lame` LGPL, AAC patents), bigger dependency surface, requires resampling pipeline. Far outside the simplicity envelope.
- **Per-sample dithering / channel reduction**: rejected as the v1 compression knob — orthogonal to format and adds tuning surface without clearly mapping to a single profile id.

---

## R-002 — Which audio formats does the v1 "supported set" include?

**Decision**: One format: `wav`. Modeled as a `SupportedFormat` enum with a single variant in v1 and a documented extensibility point.

**Rationale**: The spec demands a "defined supported set". A set of cardinality 1 is still a defined set, and the surrounding scaffolding (enum, parser, validator, compatibility matrix) is shaped so adding a second format later is a localized change. Honest scoping — promising FLAC/MP3 now would force a dependency we don't yet need.

**Alternatives considered**:

- **WAV + FLAC** simultaneously: rejected for the same reason as R-001 — new dep + codec integration.
- **Pure marketing list** (declare `mp3`, `flac` "supported" but reject them at runtime): rejected — violates the spec's CA-002 deterministic-behavior gate and would surface confusing errors to users.

---

## R-003 — Config file format and location

**Decision**: A line-based `key=value` text file at `./config/audio.conf`, with `#` line comments and blank lines ignored.

```text
# Audio defaults — edit and restart the app to apply.
format=wav
compression=pcm16
```

- **Path**: `./config/audio.conf` relative to the working directory, mirroring `./recordings/`.
- **Parser**: ~40-line custom parser, single pass over `String::lines()`, trimming whitespace and rejecting unknown keys.
- **Charset**: UTF-8, ASCII-safe identifiers.
- **Default copy**: a `config/audio.conf` is checked into the repo so users can see the canonical form and the documented values.

**Rationale**:

- **Zero new deps** — no `toml`, `serde`, or `serde_derive`. Stays in the Rust-First dep-minimal envelope.
- The config has 2 keys; a structured format is overkill.
- Plain text is friendlier than JSON for hand-editing and survives line-ending differences without ceremony.

**Alternatives considered**:

- **TOML via `toml` + `serde` + `serde_derive`**: rejected for v1 — adds three dependencies for two keys. Will be reconsidered once the config grows past ~5 keys or nests.
- **JSON via `serde_json`**: rejected — less ergonomic for hand-editing; still a new dependency.
- **Environment variables only**: rejected — not "stored in a configuration file" per FR-003, and harder to discover.
- **Embedded constants only**: rejected — violates FR-003 explicitly.

---

## R-004 — When does validation happen?

**Decision**: Two validation points:

1. **At TUI startup**, immediately after parsing `config/audio.conf` (or selecting the fallback defaults). Invalid values surface as a non-fatal status message *and* prevent recording/monitoring start; the user can keep navigating recordings and editing the file.
2. **At thread spawn**, inside `handle_record` / `handle_monitor`, the active `AudioOutputProfile` is re-validated against the compatibility matrix before any audio thread is created.

**Rationale**:

- Matches FR-005/FR-006 ("validate before recording begins" / "deterministic, actionable error messages").
- Two-layer validation tolerates future hot-reload or in-TUI override paths without restructuring the recording path.

**Alternatives considered**:

- **Validate only at write time**: rejected — creates a half-recorded file before discovering the problem, contradicting the spec's fail-fast intent.
- **Validate only at startup**: rejected — would break the moment runtime override surfaces are introduced.

---

## R-005 — Fallback when the config file is missing

**Decision**: When `./config/audio.conf` is missing or unreadable, fall back to **built-in defaults** `format = wav`, `compression = pcm16`, and surface the load source as a non-fatal status note:

> `Using built-in defaults — config/audio.conf not found.`

**Rationale**: Direct from the spec's "Edge Cases" section: *"What happens when the configuration file is missing? The system uses built-in fallback defaults and indicates that file-based defaults were unavailable."*

- `pcm16` is the existing pre-feature behavior, so an upgrading user with no config file sees no behavior change.
- Surface the source (`File` vs `Fallback`) on the loaded `AudioDefaultsConfig` so the TUI can render the notice deterministically.

**Alternatives considered**:

- **Hard-fail on missing config**: rejected — friction for upgrading users; spec edge case forbids it.
- **Auto-write a default file**: rejected for v1 — silent disk writes from a TUI startup path are surprising; the repo ships a default copy under `config/`, which is the explicit-by-design alternative.

---

## R-006 — How is the profile threaded through the existing audio pipeline?

**Decision**:

- `record::start_recording_thread(stop_flag, tx, recordings_dir, profile)` — new `profile: AudioOutputProfile` parameter.
- `MonitorConfig` gains a `output_profile: AudioOutputProfile` field; defaults to `wav + pcm16` to preserve current monitor test behavior.
- The single capture loop continues to accumulate `Vec<i16>`. The **only** profile-aware step is the final `write_wav_file` (and its counterpart in `record.rs`), which builds a `hound::WavSpec` from the profile and converts samples as needed.

**Rationale**: Keeps the hot path (capture) untouched; isolates the profile concern to the finalization step. Minimizes surface area and risk to feature 003's monitor lifecycle.

**Alternatives considered**:

- **Convert samples during capture**: rejected — capture runs under audio callback latency; pushing format-conversion there risks underruns. Doing it at finalize is fine because finalize is already a synchronous flush.
- **Per-segment profile overrides**: rejected — out of scope; spec only requires defaults + future override, no per-segment selection.

---

## R-007 — Output naming and existing-file safety

**Decision**: Filenames keep the current `recording_YYYYMMDD_HHMMSS_mmm.wav` scheme regardless of profile. No `.wav` extension change.

**Rationale**:

- All v1 profiles produce WAV containers — the extension stays accurate.
- FR-008 ("preserve existing recordings unchanged when defaults or explicit settings are modified") is satisfied because the writer only emits to a fresh timestamped path; existing files on disk are never touched.

**Alternatives considered**:

- **Suffix the filename with the profile** (e.g., `recording_..._pcm24.wav`): considered nice for forensics but rejected as scope creep — playback discovers the spec from the WAV header anyway.

---

## Summary of Decisions

| Topic              | Decision                                                          |
|--------------------|-------------------------------------------------------------------|
| Compression knob   | WAV bit-depth profiles: `pcm8`, `pcm16`, `pcm24`, `float32`       |
| Supported formats  | `wav` (single, with enum extensibility)                           |
| Config format      | `key=value` plain text, `#` comments, custom parser, no new deps  |
| Config path        | `./config/audio.conf` (default copy checked into repo)            |
| Validation         | At startup AND at thread spawn                                    |
| Missing config     | Built-in defaults `wav + pcm16`, surface source on `AudioDefaultsConfig` |
| Pipeline integration | Profile threaded through to the WAV writer step only            |
| File naming        | Unchanged                                                         |

All spec-level NEEDS CLARIFICATION are resolved.