# Data Model

## Entity: `AudioDevice`

- **Source**: `src/model/mod.rs`
- **Fields**:
  - `id: String`
  - `name: String`
  - `direction: DeviceDirection` (`Input` | `Output`)
  - `is_available: bool`
- **Current production constraints**:
  - `audio::devices::list_devices()` currently returns deterministic placeholder
    rows (`default-input`, `default-output`).
  - CLI rendering prints tab-separated rows in order:
    `id`, `direction`, `name`, availability token.

## Entity: `RecordingSession`

- **Source**: `src/model/mod.rs`
- **Fields**:
  - `input_device_id: Option<String>`
  - `output_path: String`
  - `status: String`
- **Current production constraints**:
  - `record_to_wav()` creates an in-memory session object on success with:
    - `input_device_id: None`
    - `status: "completed"`
  - Session is not persisted; it models completed operation metadata.

## Entity: `WavAsset`

- **Source**: `src/model/mod.rs`
- **Fields**:
  - `path: String`
  - `duration_ms: Option<u64>`
- **Current production constraints**:
  - Recording writes PCM 16-bit integer WAV output (`hound::WavSpec`).
  - Playback accepts only:
    - integer 16-bit WAV,
    - float 32-bit WAV.
  - Missing, empty, or unsupported-format files produce deterministic errors.

## Entity: `PlaybackRequest`

- **Source**: `src/model/mod.rs`
- **Fields**:
  - `wav_path: String`
  - `output_device_id: Option<String>`
  - `volume: u8`
- **Current production constraints**:
  - `volume` must be within `0..=100`; values over 100 are rejected.
  - Current runtime ignores `output_device_id` and uses host default output
    device.

## Behavioral State Transitions

### Recording flow

1. Validate output path (`validate_record_request`).
2. Resolve temp path (`<output>.tmp.wav`).
3. Capture microphone samples from default input for ~10 seconds.
4. Write WAV samples to temp file and finalize writer.
5. Rename temp file to final output path.
6. On any failure before rename, remove temp file.

### Playback flow

1. Validate file exists and `volume <= 100`.
2. Open and decode supported WAV into normalized `f32` samples.
3. Build output stream for default output device and runtime sample format.
4. Convert/scaled sample frames into output buffer until finished.
5. Exit successfully after playback state reaches `finished = true`.