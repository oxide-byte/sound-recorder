# Data Model: Audio Format Compression

**Feature**: 004-audio-format-compression
**Date**: 2026-05-16

This document captures the entities, validation rules, and state transitions introduced by feature 004. All types live under `src/model/` (existing module) or `src/config/` (new module).

---

## Entities

### `SupportedFormat` (enum) — `src/model/mod.rs`

Identifies the container/format of the saved audio file.

| Variant | String id (config & errors) | Notes |
|---------|----------------------------|-------|
| `Wav`   | `wav`                      | Only variant in v1; emitted via `hound::WavWriter` |

**Validation**:
- Parsing is case-insensitive (`Wav` accepts `wav`, `WAV`, `Wav`).
- Unknown identifiers yield `AppError::Config(format!("unsupported format '{ident}'; supported: wav"))`.

**Extensibility**: New variants can be added without changing call sites — the compatibility matrix and the writer dispatch both `match` exhaustively, so the compiler enforces completeness.

---

### `CompressionProfile` (enum) — `src/model/mod.rs`

Identifies the bit-depth / sample-format profile applied when writing the file.

| Variant   | String id   | `hound::SampleFormat` | `bits_per_sample` |
|-----------|-------------|----------------------|-------------------|
| `Pcm8`    | `pcm8`      | `Int`                | 8                 |
| `Pcm16`   | `pcm16`     | `Int`                | 16                |
| `Pcm24`   | `pcm24`     | `Int`                | 24                |
| `Float32` | `float32`   | `Float`              | 32                |

**Validation**:
- Parsing is case-insensitive.
- Unknown identifiers yield `AppError::Config(format!("unsupported compression '{ident}'; supported: pcm8, pcm16, pcm24, float32"))`.

---

### `AudioOutputProfile` (struct) — `src/model/mod.rs`

The combined selection applied to a recording job.

```rust
pub struct AudioOutputProfile {
    pub format: SupportedFormat,
    pub compression: CompressionProfile,
}
```

**Invariants**:
- `(format, compression)` MUST appear in the compatibility matrix below.
- Constructed only via `AudioOutputProfile::validated(format, compression) -> Result<Self, AppError>`.

**Compatibility matrix (v1)**:

| Format \ Compression | `Pcm8` | `Pcm16` | `Pcm24` | `Float32` |
|----------------------|--------|---------|---------|-----------|
| `Wav`                | ✅      | ✅       | ✅       | ✅         |

All combinations are valid in v1 because there is only one format. The matrix is checked centrally so adding a future restrictive format requires only matrix edits and exhaustive-match maintenance.

**Default**:

```rust
AudioOutputProfile {
    format: SupportedFormat::Wav,
    compression: CompressionProfile::Pcm16,
}
```

`Pcm16` is the default because it matches feature 003's hardcoded writer output bit-for-bit.

---

### `ConfigSource` (enum) — `src/config/mod.rs`

Records where the active defaults came from. Used by the TUI to display a load-state notice.

| Variant     | Meaning                                                                      |
|-------------|------------------------------------------------------------------------------|
| `File`      | Loaded from `./config/audio.conf`                                            |
| `Fallback`  | File missing or unreadable; built-in defaults used                           |

---

### `AudioDefaultsConfig` (struct) — `src/config/mod.rs`

Resolved defaults plus provenance.

```rust
pub struct AudioDefaultsConfig {
    pub profile: AudioOutputProfile,
    pub source: ConfigSource,
}
```

**Invariants**:
- `profile` is always validated via `AudioOutputProfile::validated(..)` before the struct is constructed.

**Construction paths**:

```text
load_or_default(path)
├── file exists & parses & valid  → AudioDefaultsConfig { profile: parsed,  source: File }
├── file missing                  → AudioDefaultsConfig { profile: default, source: Fallback }
└── file exists but invalid       → Err(AppError::Config(...))
```

The third arm is **fail-fast**, matching FR-005/006: an *existing-but-broken* config is a user mistake worth surfacing, not silently masking.

---

### `ParsedConfigLine` (internal struct) — `src/config/mod.rs`

Intermediate representation used only inside the parser.

```rust
enum ParsedConfigLine {
    KeyValue { key: String, value: String, lineno: usize },
    Comment,
    Blank,
}
```

Not exposed publicly.

---

### `MonitorConfig` extension — `src/audio/monitor.rs`

Existing struct (from feature 003) gains one field:

```rust
pub struct MonitorConfig {
    pub threshold_fraction: f32,
    pub silence_timeout: Duration,
    pub pre_roll: Duration,
    pub min_clip_duration: Duration,
    pub output_profile: AudioOutputProfile,   // ← NEW
}
```

`Default for MonitorConfig` populates `output_profile` with `AudioOutputProfile::default()` so existing tests that construct `MonitorConfig::default()` still compile.

---

### `AppError::Config` (enum variant) — `src/error.rs`

Existing enum gains:

```rust
#[error("config error: {0}")]
Config(String),
```

Used for:
- Parse failures (unknown key, malformed line, unknown identifier).
- Validation failures (unsupported format/compression, incompatible pair).
- Optionally: file-read errors that are not `NotFound` (e.g., permission denied) — file `NotFound` is *not* an error and triggers the `Fallback` source instead.

---

## Relationships

```text
AudioDefaultsConfig
 ├─ profile: AudioOutputProfile
 │           ├─ format: SupportedFormat
 │           └─ compression: CompressionProfile
 └─ source: ConfigSource

MonitorConfig
 └─ output_profile: AudioOutputProfile  (same type as above)

start_recording_thread(stop_flag, tx, dir, profile: AudioOutputProfile)
start_monitoring_thread(stop_flag, tx, dir, monitor_config: MonitorConfig)
```

The TUI holds a single `AudioDefaultsConfig`, loaded once at startup, and clones the inner `AudioOutputProfile` into each thread it spawns.

---

## State Transitions

### Config load (TUI startup)

```text
[start]
   │
   ▼
read ./config/audio.conf
   │
   ├── not found ─────────────────────────────► AudioDefaultsConfig {default, Fallback}
   │                                            status: "Using built-in defaults — config/audio.conf not found."
   │
   ├── read error (perm denied, etc.) ────────► AppError::Config(...) — TUI shows error, profile remains None
   │
   └── read OK
        │
        ▼
      parse lines
        │
        ├── unknown key / malformed ──────────► AppError::Config(...) — TUI shows error
        │
        └── parsed OK
             │
             ▼
           validate (AudioOutputProfile::validated)
             │
             ├── invalid pair / unknown id ────► AppError::Config(...) — TUI shows error
             │
             └── valid ────────────────────────► AudioDefaultsConfig {profile, File}
                                                  status: optional one-shot "Loaded audio defaults: wav/pcm16"
```

### Recording / monitoring start

```text
press 'r' or 'm'
   │
   ▼
ctx.defaults.is_some()? ── no ──► status: "Audio defaults invalid — fix config/audio.conf"; no thread spawned
   │
   yes
   │
   ▼
AudioOutputProfile::validated(profile.format, profile.compression)
   │
   ├── err ──► status: error message; no thread spawned
   │
   └── ok ───► start_recording_thread(..., profile)   or   start_monitoring_thread(..., MonitorConfig { ..., output_profile: profile })
```

### File write (inside record/monitor thread)

```text
captured samples (Vec<i16>)
   │
   ▼
match profile.compression:
   ├─ Pcm8     → WavSpec { Int,   bits: 8  }, write_sample((s as i32 / 256 + 128) as u8) channel-wise
   ├─ Pcm16    → WavSpec { Int,   bits: 16 }, write_sample(s as i16)   (existing path)
   ├─ Pcm24    → WavSpec { Int,   bits: 24 }, write_sample((s as i32) << 8)
   └─ Float32  → WavSpec { Float, bits: 32 }, write_sample(s as f32 / i16::MAX as f32)
   │
   ▼
finalize WAV
```

Bit-depth conversion notes:

- `Pcm8` per WAV spec is **unsigned** 8-bit (offset-binary). Converting `i16` → `u8` requires the `+128` offset.
- `Pcm24` per WAV spec is **signed** 24-bit, little-endian, sign-extended to 32-bit when fed to `hound::WavWriter::write_sample::<i32>`.
- `Float32` uses `[-1.0, 1.0]` range per WAV-EXT spec.

---

## Open Questions

None. All resolved in research.md.