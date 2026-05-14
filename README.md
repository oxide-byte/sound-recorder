# Sound Recorder

`sound-recorder` is a Rust TUI audio application.

Current user workflow is TUI-first:

- app startup opens the TUI directly,
- the UI provides `Record`, `Play`, and `Stop` actions,
- the UI shows a list of stored `WAV` files,
- each `Record` → `Stop` cycle creates one new `WAV` file.

Detailed target behavior is documented in
`specs/002-tui-recorder-ui/spec.md`.

## Build

```bash
cargo build
```