# Implementation Plan: TUI Recorder Workflow

**Branch**: `002-tui-recorder-ui` | **Date**: 2026-05-14 | **Spec**: specs/002-tui-recorder-ui/spec.md  
**Input**: Feature specification from `specs/002-tui-recorder-ui/spec.md`

## Summary

Replace the CLI command dispatch in `main.rs` with a direct TUI startup. The TUI presents Record, Play, and Stop controls alongside a scrollable list of WAV files stored in `./recordings/`. Each Record→Stop cycle produces a new auto-named WAV file using the existing `cpal`/`hound` audio stack run on background threads, communicating completion and errors back to the event loop via channels.

## Technical Context

**Language/Version**: Rust 1.85 (edition 2024)  
**Primary Dependencies**: ratatui 0.29.0, crossterm 0.28.1, cpal 0.15.3, hound 3.5.1, thiserror 2.0.11 (all unchanged); `clap` 4.5.8 removed — CLI argument parsing no longer required  
**Storage**: Local filesystem; `./recordings/` directory relative to CWD, auto-created on first run  
**Testing**: `cargo test` — unit + integration  
**Target Platform**: macOS/Linux terminal emulator  
**Project Type**: Terminal application (TUI)  
**Performance Goals**: UI frame render <16 ms; recording start latency <500 ms; Stop→file-saved latency <1 s  
**Constraints**: Single-threaded event loop on main thread; audio on dedicated background threads; no async runtime  
**Scale/Scope**: Single-user local use; no concurrent users or distributed state

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **CA-001 Rust-First** ✅: All implementation remains Rust. `clap` is removed (dependency reduction, not addition). All retained crates are already in use.
- **CA-002 TUI replaces CLI-First** ⚠️ BREAKING — JUSTIFIED: The spec explicitly mandates TUI-only interaction. All prior CLI sub-commands (`record`, `play`, `list-devices`, `tui`) are removed. Interaction is now event-driven key presses with deterministic state transitions. MAJOR version bump to v1.0.0 is required.
- **CA-003 Verification Gate** ✅: Each user story has an independently testable path. Integration tests for record→file-exists and play→stop are required. Manual TUI render verification acceptable for visual state indicators.
- **CA-004 Integration Safety** ✅: Removing `cli/` module and rewriting `tui/` are cross-module changes. Integration tests must cover audio engine wiring through TUI state transitions (Idle→Recording→Idle, Idle→Playing→Idle).
- **CA-005 Version Discipline** ✅: v0.1.0 → v1.0.0. Breaking change (CLI removal) documented above.

## Project Structure

### Documentation (this feature)

```text
specs/002-tui-recorder-ui/
├── plan.md                        # This file
├── research.md                    # Phase 0 output
├── data-model.md                  # Phase 1 output
├── quickstart.md                  # Phase 1 output
├── contracts/
│   ├── tui-key-bindings.md        # Phase 1 output
│   └── audio-thread-protocol.md   # Phase 1 output
└── tasks.md                       # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code

```text
src/
├── main.rs              # Init terminal → run_tui() → restore terminal; no CLI dispatch
├── tui/
│   ├── mod.rs           # pub fn run_tui() — entry point
│   ├── app.rs           # AppState machine, event loop, audio-thread coordination
│   └── view.rs          # ratatui render function (buttons + file list + status bar)
├── audio/
│   ├── mod.rs           # re-exports
│   ├── record.rs        # Modified: stop-signal-controlled stream instead of sleep-based
│   └── playback.rs      # Modified: stop-signal-controlled stream instead of polling loop
├── model/
│   └── mod.rs           # Updated entities: WavFileEntry, RecordingHandle, PlaybackHandle
├── error.rs             # AppError — unchanged
└── lib.rs               # Remove cli re-export; expose audio, model, tui, error

recordings/              # Created at runtime; WAV output files stored here
```

**Structure Decision**: Single project. `cli/` module is removed entirely. `tui/` is rewritten from stub to full implementation. `audio/` is modified to support stop-signal-controlled streams. `model/` entities are updated to reflect TUI-centric state.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| Background audio threads | Recording and playback must not block the TUI event loop | A blocking single-thread approach locks the UI; Stop can never be activated while audio is running |