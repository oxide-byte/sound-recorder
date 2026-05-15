# Sound Recorder

`sound-recorder` is a Rust TUI audio application.

Current user workflow is TUI-first:

- app startup opens the TUI directly,
- the UI provides `Record`, `Play`, and `Stop` actions,
- the UI shows a list of stored `WAV` files,
- each `Record` → `Stop` cycle creates one new `WAV` file.

Detailed target behavior is documented in specs:
- [001-terminal-audio-recorder-player](specs/001-terminal-audio-recorder-player)
- [002-tui-recorder-ui](specs/002-tui-recorder-ui)`specs/002-tui-recorder-ui/spec.md`
- [003-continuous-recording](specs/003-continuous-recording)`specs/specs/003-continuous-recording)003-tui-recorder-ui/spec.md`

## Features

- Launches directly into the TUI on startup.
- Supports end-to-end recording with `Record` and `Stop` actions.
- Supports playback of stored `WAV` files via the `Play` action.
- Displays and refreshes the recordings list in the UI.

## Development

This project is based on SSD(Spec Driven Development), see: https://github.com/github/spec-kit

The development workflow follows these phases using Speckit commands:

### 1. Specification Phase
Create or update a feature specification from a natural language description.
```bash
/speckit.specify "description of the feature"
```

### 2. Planning Phase
Generate technical design artifacts (research, data model, contracts) based on the specification.
```bash
/speckit.plan
```

### 3. Task Phase
Break down the implementation plan into actionable tasks.
```bash
/speckit.tasks
```

### 4. Implementation Phase
Execute the tasks and implement the feature.
```bash
/speckit.implement
```

## Build

```bash
cargo build
```

## Look

![screenshot.png](doc/screenshot.png)