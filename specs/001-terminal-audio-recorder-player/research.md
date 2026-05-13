# Phase 0 Research: Terminal Audio Recorder/Player Alignment

## Decision 1: Treat `HEAD` behavior as the specification baseline

- **Decision**: Align planning/spec artifacts to current code and tests instead of
  aspirational roadmap behavior.
- **Rationale**: `src/audio/record.rs`, `src/audio/playback.rs`, and integration
  tests provide deterministic, test-backed behavior already in use.
- **Alternatives considered**:
  - Preserve older aspirational requirements in active acceptance criteria.
  - Document both current and target behavior equally as “implemented”.

## Decision 2: Keep fixed-duration recording window (~10 seconds)

- **Decision**: Document recording as a fixed default-device capture of roughly
  10 seconds (`DEFAULT_RECORDING_DURATION`).
- **Rationale**: Implementation explicitly sleeps for 10 seconds, and
  `tests/integration/record_cli.rs` verifies elapsed time and WAV duration.
- **Alternatives considered**:
  - User-configurable duration flag.
  - Indefinite capture until interrupted.

## Decision 3: Preserve temp-file write then rename for record output

- **Decision**: Describe record output as write-to-temp (`*.tmp.wav`) followed
  by rename to requested output path with cleanup on failure.
- **Rationale**: This avoids exposing partially written output files and is
  directly implemented in `record_to_wav`.
- **Alternatives considered**:
  - Direct writes to final output path.
  - Background recording with progressive file updates.

## Decision 4: Playback pipeline is format-aware with strict validation

- **Decision**: Document playback as:
  - source WAV decode supporting `Int/16-bit` and `Float/32-bit`,
  - output conversion based on runtime output sample format (`I16`, `U16`, `F32`),
  - volume scaling from `0..=100`.
- **Rationale**: This mirrors `read_samples_as_f32`, `write_output_data`, and
  command/unit test validation for invalid volume and missing files.
- **Alternatives considered**:
  - Accept every WAV encoding via generalized conversion.
  - Restrict output to one fixed sample format.

## Decision 5: Explicitly mark device-id routing as deferred capability

- **Decision**: Keep `--input-device` and `--output-device` in contracts/specs,
  but state that current audio pipelines use default host devices.
- **Rationale**: CLI parsing accepts these options while service functions ignore
  them (`_input_device_id`, `_output_device_id`), so claiming active routing
  support would be inaccurate.
- **Alternatives considered**:
  - Remove flags from documentation until implemented.
  - Describe full device-id routing as already available.