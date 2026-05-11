# Sound Recorder

`sound-recorder` is a Rust CLI application for recording microphone audio to WAV,
playing WAV files, listing audio devices, and launching a basic Ratatui mode.

## Usage

- `sound-recorder list-devices`
- `sound-recorder record --output ./sample.wav [--input-device <id>]`
- `sound-recorder play --file ./sample.wav [--output-device <id>] [--volume <0..100>]`
- `sound-recorder tui`