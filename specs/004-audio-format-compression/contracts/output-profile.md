# Contract: Audio Output Profile

**Feature**: 004-audio-format-compression
**Owner module**: `src/model/` (types) + `src/audio/record.rs` & `src/audio/monitor.rs` (writers)
**Consumer**: `src/tui/app.rs`, tests under `tests/unit/profile_validation.rs`

This contract defines the validation surface and the writer-side guarantees for `AudioOutputProfile`. It complements `config-file.md`, which defines the file format that *produces* a profile.

---

## Types

```rust
pub enum SupportedFormat { Wav }

pub enum CompressionProfile { Pcm8, Pcm16, Pcm24, Float32 }

pub struct AudioOutputProfile {
    pub format: SupportedFormat,
    pub compression: CompressionProfile,
}
```

---

## Identifier Parsing

```rust
impl SupportedFormat {
    pub fn from_id(s: &str) -> Result<Self, AppError>;
    pub fn as_id(&self) -> &'static str;
}

impl CompressionProfile {
    pub fn from_id(s: &str) -> Result<Self, AppError>;
    pub fn as_id(&self) -> &'static str;
}
```

| Method     | Behavior                                                                            |
|------------|-------------------------------------------------------------------------------------|
| `from_id`  | Case-insensitive lookup against the documented identifier set. Returns `AppError::Config` on unknown input (exact wording in `config-file.md` error catalog). |
| `as_id`    | Returns the canonical lowercase id used in config files and error messages.         |

Identifier round-trip: `T::from_id(T::as_id(&t)).unwrap() == t` for every variant.

---

## Validated Construction

```rust
impl AudioOutputProfile {
    pub fn validated(format: SupportedFormat, compression: CompressionProfile)
        -> Result<Self, AppError>;
}
```

Performs the compatibility-matrix check. In v1 the matrix is fully permissive (all `(Wav, *)` valid), so this function always returns `Ok` in v1. The function MUST be called anyway — both at config load and at thread spawn — so adding a restrictive variant later is a single-site change.

The future error wording is:

> `config error: format '{f}' is not compatible with compression '{c}'`

---

## Default

```rust
impl Default for AudioOutputProfile {
    fn default() -> Self {
        AudioOutputProfile {
            format: SupportedFormat::Wav,
            compression: CompressionProfile::Pcm16,
        }
    }
}
```

The default is byte-identical to feature 003's hardcoded writer (`Int, 16-bit, channels & sample_rate inherited from the input device`).

---

## Writer Behavior

### `record::start_recording_thread`

```rust
pub fn start_recording_thread(
    stop_flag: Arc<AtomicBool>,
    tx: mpsc::Sender<Result<PathBuf, AppError>>,
    recordings_dir: PathBuf,
    profile: AudioOutputProfile,            // ← NEW
) -> JoinHandle<()>;
```

Inside the thread, after capture completes and `Vec<i16>` is in hand:

| `profile.compression` | `WavSpec`                                              | Per-sample conversion                              |
|-----------------------|--------------------------------------------------------|----------------------------------------------------|
| `Pcm8`                | `{ Int, bits_per_sample: 8,  channels, sample_rate }`  | `u8 = ((s as i32 + 32_768) >> 8) as u8`            |
| `Pcm16`               | `{ Int, bits_per_sample: 16, channels, sample_rate }`  | identity (`write_sample::<i16>(s)`)                |
| `Pcm24`               | `{ Int, bits_per_sample: 24, channels, sample_rate }`  | `write_sample::<i32>((s as i32) << 8)`             |
| `Float32`             | `{ Float, bits_per_sample: 32, channels, sample_rate }`| `write_sample::<f32>(s as f32 / i16::MAX as f32)`  |

`format` is always `SupportedFormat::Wav` in v1; the writer dispatches `match` exhaustively so a future variant fails to compile.

### `monitor::start_monitoring_thread`

`MonitorConfig` gains:

```rust
pub struct MonitorConfig {
    pub threshold_fraction: f32,
    pub silence_timeout: Duration,
    pub pre_roll: Duration,
    pub min_clip_duration: Duration,
    pub output_profile: AudioOutputProfile,   // ← NEW
}
```

`Default for MonitorConfig` populates `output_profile` with `AudioOutputProfile::default()`, preserving feature 003's monitor tests.

`monitor::write_wav_file` follows the same compression-dispatch table as the recording thread above.

---

## Round-Trip Guarantees

For each `(profile)` the saved WAV file MUST satisfy, when re-read via `hound::WavReader::open(path)`:

| Property             | Expected                                              |
|----------------------|-------------------------------------------------------|
| `spec.sample_format` | `Int` for `Pcm8`/`Pcm16`/`Pcm24`, `Float` for `Float32` |
| `spec.bits_per_sample` | 8 / 16 / 24 / 32 respectively                       |
| `spec.channels`      | unchanged from input device                           |
| `spec.sample_rate`   | unchanged from input device                           |
| Sample count         | identical to the count produced by feature 003's writer for the same capture buffer (no resampling) |

---

## Validation Touchpoints

| Site                                  | What it checks                                          |
|---------------------------------------|---------------------------------------------------------|
| `config::load_or_default`             | Identifier parse + matrix                               |
| `tui::app::handle_record`             | Matrix re-check before thread spawn                     |
| `tui::app::handle_monitor`            | Matrix re-check before thread spawn                     |

The thread itself does **not** revalidate — by construction, `AudioOutputProfile` only exists in valid states (`validated()` is the only public constructor).

---

## Test Contracts

Unit tests in `tests/unit/profile_validation.rs` MUST cover:

- `SupportedFormat::from_id`: every valid id (case-insensitive) and at least three unknown-id cases assert the documented error message.
- `CompressionProfile::from_id`: same coverage.
- `AudioOutputProfile::validated` returns `Ok` for every v1 pair.
- Round-trip `as_id` → `from_id` for every variant.

Unit tests in `src/audio/record.rs` (`#[cfg(test)]`) MUST cover:

- For each compression profile, the WAV writer produces a file whose `WavSpec` matches the expected row in the dispatch table.
- The default profile produces a file bitwise-identical to the pre-feature writer for an identical sample buffer.