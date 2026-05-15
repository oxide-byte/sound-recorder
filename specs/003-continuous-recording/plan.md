# Implementation Plan: Continuous Sound-Activated Recording

**Branch**: `003-continuous-recording` | **Date**: 2026-05-15 | **Spec**: specs/003-continuous-recording/spec.md  
**Input**: Feature specification from `specs/003-continuous-recording/spec.md`

## Summary

Add a sound-activated monitoring mode to the existing TUI. When the user presses `m`, a background thread starts reading the default microphone continuously, maintains a rolling pre-roll buffer, detects sound by amplitude threshold, captures each detected sound event to a temp WAV file (prepending the pre-roll), applies a silence timeout to bridge short pauses, enforces a minimum clip duration to discard accidental noise bursts, and sends status events back to the TUI event loop via a channel. The TUI displays four distinct monitoring sub-states (Monitoring, Capturing, Finalizing, Stopping) and refreshes the recordings list on each successfully saved segment. No new crate dependencies are required.

## Technical Context

**Language/Version**: Rust 1.85 (edition 2024)  
**Primary Dependencies**: ratatui 0.29.0, crossterm 0.28.1, cpal 0.15.3, hound 3.5.1, thiserror 2.0.11 — all unchanged; no new crates  
**Storage**: Local filesystem; `./recordings/` directory (existing, auto-created)  
**Testing**: `cargo test` — unit + integration  
**Target Platform**: macOS/Linux terminal emulator  
**Project Type**: Terminal application (TUI)  
**Performance Goals**: UI frame render <16 ms; monitor activation <1 s; Stop→idle ≤2 s; keypress response <100 ms during monitoring  
**Constraints**: Single-threaded TUI event loop; audio detection and file I/O on dedicated background thread; no async runtime; pre-roll buffer bounded to ≤500 ms of samples; individual sound segments buffered in memory (bounded by expected event duration of seconds to a few minutes)  
**Scale/Scope**: Single-user local use; no concurrent users or distributed state

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **CA-001 Rust-First** ✅: No new crate dependencies. All new code (monitoring thread, detection logic, pre-roll buffer) is pure Rust using existing `cpal` and `hound` crates. `MonitorConfig` and `MonitoringHandle` are new Rust structs, not wrappers around foreign code.
- **CA-002 TUI-First** ✅ (CLI-First superseded by 002-tui-recorder-ui BREAKING change): All new user controls — start monitoring (`m`), stop monitoring (`s`) — are TUI key bindings. No CLI flags, sub-commands, or stdin/stdout surfaces are added. Status labels and rejection messages are rendered via ratatui.
- **CA-003 Verification Gate** ✅: Detection logic (amplitude threshold, silence timeout, minimum clip duration, pre-roll) is unit-testable with synthetic `i16` sample slices without requiring a real audio device. State machine transitions (Idle→Monitoring→Capturing→Idle) are integration-testable via the existing `AppState` pattern. Manual TUI verification is acceptable for visual state labels only.
- **CA-004 Integration Safety** ✅: `AppState` gains new variants (`Monitoring`). `TuiContext` gains no new fields — monitoring sub-state is conveyed through the channel protocol. Integration tests must cover Idle→Monitoring→Capturing→Idle (save path) and Idle→Monitoring→Idle (stop-during-silence path). Existing `Recording` and `Playing` paths are unchanged; `handle_stop` is extended but the existing arms are not modified.
- **CA-005 Version Discipline** ✅: Feature is purely additive (new `m` keybinding, new `AppState::Monitoring` variant, new `audio/monitor.rs` module). No existing keybinding or state transition is removed or changed incompatibly. Version bump: v1.0.0 → v1.1.0 (MINOR).

*Post-Phase 1 re-check*: Constitution Check remains fully satisfied after design. No violations introduced.

## Project Structure

### Documentation (this feature)

```text
specs/003-continuous-recording/
├── plan.md                              # This file
├── research.md                          # Phase 0 output
├── data-model.md                        # Phase 1 output
├── quickstart.md                        # Phase 1 output
├── contracts/
│   ├── monitor-thread-protocol.md       # Phase 1 output
│   └── monitor-keybindings.md           # Phase 1 output
└── tasks.md                             # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code

```text
src/
├── main.rs              # Unchanged
├── tui/
│   ├── mod.rs           # Unchanged
│   ├── app.rs           # Extended: poll MonitoringHandle, handle_monitor(), handle_stop() extended
│   └── view.rs          # Extended: Monitor button, Monitoring/Capturing/Stopping state styles
├── audio/
│   ├── mod.rs           # Extended: pub mod monitor
│   ├── record.rs        # Unchanged
│   ├── playback.rs      # Unchanged
│   └── monitor.rs       # NEW: MonitorConfig, pre-roll buffer, detection, segment lifecycle, thread entry
├── model/
│   └── mod.rs           # Extended: MonitoringHandle, MonitorEvent, MonitoringSubState, AppState::Monitoring
├── error.rs             # Unchanged
└── lib.rs               # Unchanged

recordings/              # Runtime; monitoring-produced WAV files stored alongside manual recordings
```

**Structure Decision**: Single project, consistent with 001 and 002. `audio/monitor.rs` is a new file rather than extending `record.rs` because the monitoring loop has a fundamentally different internal structure (pre-roll buffer, threshold detection, silence timeout, multi-segment lifecycle). All other modules are extended in-place with minimal surface additions.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| Segment samples buffered in `Vec<i16>` | Simplest approach matching existing record.rs pattern | Streaming incremental WAV writes require keeping a `hound::WavWriter` open across the silence timeout window with partial state; adds complexity and risks partial writes on stop. Buffering is safe for expected event durations (seconds to minutes). |
