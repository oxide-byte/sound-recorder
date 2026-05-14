# Quickstart: TUI Recorder Workflow

**Feature**: 002-tui-recorder-ui  
**Date**: 2026-05-14

## Prerequisites

- Rust toolchain (edition 2024, stable ≥ 1.85): `rustup update stable`
- Microphone and speaker accessible at the OS level (check System Preferences → Privacy → Microphone on macOS)
- Terminal emulator with UTF-8 support

## Build

```sh
cargo build --release
```

The binary is placed at `target/release/sound-recorder`.

## Run

```sh
cargo run --release
# or
./target/release/sound-recorder
```

The TUI launches immediately. A `./recordings/` directory is created automatically if it does not exist.

## Using the TUI

```
┌─────────────────────────────────────────────────────────┐
│  sound-recorder                                          │
├─────────────────────────────────────────────────────────┤
│  [ Record ]   [ Play ]   [ Stop ]                        │
├─────────────────────────────────────────────────────────┤
│  Stored Recordings                                       │
│  ▸ recording_20260514_120000_000.wav                     │
│    recording_20260514_115500_123.wav                     │
├─────────────────────────────────────────────────────────┤
│  Ready — ↑/↓ to select, 'r' record, 'p' play            │
└─────────────────────────────────────────────────────────┘
```

| Key | Action |
|-----|--------|
| `r` | Start recording (from idle) |
| `s` | Stop recording or playback |
| `p` | Play selected file (from idle) |
| `↑` / `↓` | Navigate file list |
| `q` / `Esc` | Quit (from idle only) |

## Record a New File

1. Press `r` — status bar shows `Recording… press 's' to stop`.
2. Speak into the microphone.
3. Press `s` — a new file (e.g., `recording_20260514_120000_000.wav`) appears in the list.

## Play a Recording

1. Use `↑`/`↓` to highlight a file.
2. Press `p` — status bar shows `Playing <filename> — press 's' to stop`.
3. Press `s` to stop early, or wait for playback to finish.

## Quit

Press `q` or `Esc` while in idle state.

## Run Tests

```sh
cargo test
```

Integration tests require a connected microphone and speaker. Tests that depend on real audio hardware are marked `#[ignore]` and can be run explicitly:

```sh
cargo test -- --ignored
```

## Recordings Directory

WAV files are stored in `./recordings/` relative to the directory from which you run the binary. Files are named `recording_YYYYMMDD_HHMMSS_mmm.wav` (UTC timestamp). To browse recordings outside the TUI:

```sh
ls -lt recordings/
```