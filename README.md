# Sound Recorder

`sound-recorder` is a Rust CLI application for:

- recording microphone audio to WAV,
- playing WAV files,
- listing audio devices,
- launching a basic Ratatui mode.

## Build

```bash
cargo build
```

## Usage

### List devices

```bash
cargo run -- list-devices
```

Output is tab-separated rows:

```text
<id>\t<input|output>\t<name>\t<available|unavailable>
```

### Record

```bash
cargo run -- record --output ./sample.wav [--input-device <id>]
```

- Records from the default input device.
- Current recording duration is approximately 10 seconds.
- Prints `recorded <path>` on success.

### Play

```bash
cargo run -- play --file ./sample.wav [--output-device <id>] [--volume <0..100>]
```

- Plays through the default output device.
- Supports WAV input formats: `Int/16-bit` and `Float/32-bit`.
- Prints `played <path>` on success.

### TUI

```bash
cargo run -- tui
```

Starts the Ratatui interface and exits when the UI closes.

## Notes

- `--input-device` and `--output-device` are currently accepted by the CLI but
  are not yet routed to explicit device selection.
- Volume must be in the range `0..=100`.