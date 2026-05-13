# CLI Contract

Executable name: `sound-recorder`

## `list-devices`

- **Usage**: `sound-recorder list-devices`
- **Current behavior**:
  - exits with status `0` on success,
  - prints one tab-separated row per device:
    - `<id>\t<input|output>\t<name>\t<available|unavailable>`
- **Current baseline rows**:
  - `default-input\tinput\tDefault Input\tavailable`
  - `default-output\toutput\tDefault Output\tavailable`

## `record`

- **Usage**: `sound-recorder record --output <path> [--input-device <id>]`
- **Arguments**:
  - `--output` (required): output WAV path.
  - `--input-device` (optional): accepted by CLI, currently not routed to
    device selection.
- **Success contract**:
  - exits with status `0`,
  - records from default input for approximately 10 seconds,
  - writes valid WAV file to `<path>`,
  - prints `recorded <path>` to stdout.
- **Validation errors** (stderr, non-zero exit):
  - `output path cannot be empty`
  - `output directory does not exist: <dir>`
  - `no default input device available`
  - `no samples were captured from the input device`
  - `unsupported sample format: <format>`

## `play`

- **Usage**:
  `sound-recorder play --file <path> [--output-device <id>] [--volume <0..100>]`
- **Arguments**:
  - `--file` (required): input WAV path.
  - `--output-device` (optional): accepted by CLI, currently not routed to
    device selection.
  - `--volume` (optional, default `100`): playback scale percent.
- **Success contract**:
  - exits with status `0`,
  - plays WAV through default output device,
  - prints `played <path>` to stdout.
- **Validation errors** (stderr, non-zero exit):
  - `file does not exist: <path>`
  - `volume must be between 0 and 100, got <value>`
  - `unsupported wav format: <sample_format> <bits>-bit`
  - `no default output device available`
  - `wav file contains no samples`

## `tui`

- **Usage**: `sound-recorder tui`
- **Current behavior**:
  - starts Ratatui mode (`tui::app::run`).
  - exits with status `0` when UI closes successfully.

## Error envelope

- Main command handler returns `AppError` and prints:
  - `error: <message>` to stderr,
  - process exits with code `1` on command failure.