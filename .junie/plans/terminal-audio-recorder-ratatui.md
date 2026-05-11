---
sessionId: session-260511-174641-1voc
isActive: true
---

# Requirements

### Overview & Goals
Implement `specs/001-terminal-audio-recorder-player/spec.md` as a Rust terminal application with Ratatui-powered interactive UX while keeping deterministic CLI behavior required by `.specify/memory/constitution.md`.

### Scope
**In Scope**
- Record audio from microphone input and persist valid WAV files.
- Replay WAV files through selected/default output devices.
- List available input/output devices using stable identifiers.
- Apply validated playback volume controls.
- Keep command outcomes deterministic (stdout success, stderr failures).

**Out of Scope**
- Editing/effects (trim, mix, filters), multitrack workflows, cloud sharing.
- Output formats other than WAV.

### Functional Requirements
- Implement FR-001 through FR-010 from `specs/001-terminal-audio-recorder-player/spec.md`.
- Keep the documented environment assumption: OS-level microphone/speaker permissions are already granted.
- Preserve constitution gates: Rust-first, CLI-first, verification per story, integration safety, and version discipline.

### Implementation Preconditions
- `specs/001-terminal-audio-recorder-player/plan.md` is still template content and should be superseded by this approved Junie plan.
- `specs/001-terminal-audio-recorder-player/tasks.md` is not present yet; task breakdown must be generated before `/speckit.implement` execution.
- `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` currently fails on branch naming because the repo is on `master`; implementation must run from a feature branch like `001-terminal-audio-recorder-player`.
- `specs/001-terminal-audio-recorder-player/checklists/requirements.md` is complete, so checklist gating can pass once branch and tasks prerequisites are satisfied.

# Technical Design

### Current Implementation
- `src/main.rs` only contains a hello-world `main()`.
- `Cargo.toml` defines package metadata only; runtime dependencies are empty.
- Requirements and acceptance targets are fully defined in `specs/001-terminal-audio-recorder-player/spec.md` and `specs/001-terminal-audio-recorder-player/checklists/requirements.md`.
- `.specify/feature.json` pins the active feature directory to `specs/001-terminal-audio-recorder-player`.
- `specs/001-terminal-audio-recorder-player/plan.md` remains the unfilled template and no `tasks.md`, `research.md`, `data-model.md`, `quickstart.md`, or `contracts/` artifacts exist yet.
- Constitution constraints are codified in `.specify/memory/constitution.md`.
- There is no existing in-repository audio implementation to extend, so this is a greenfield feature slice.

### Key Decisions
1. **Single Rust binary with deterministic command interface and optional Ratatui flows**  
   Keep one executable exposing `list-devices`, `record`, and `play`; add Ratatui interaction as a presentation layer over the same services.
2. **Layered module split for testability**  
   Separate CLI routing, TUI rendering/state, audio I/O, domain models, and shared errors to keep behavior verifiable in isolation.
3. **Unified domain model across CLI/TUI**  
   Reuse `AudioDevice`, `RecordingSession`, `WavAsset`, and `PlaybackRequest` in both modes to avoid logic drift.
4. **Centralized validation and deterministic error mapping**  
   Normalize device/file/volume checks in shared validation paths so both CLI and TUI return consistent diagnostics.

### Proposed Changes
- Before coding, generate `specs/001-terminal-audio-recorder-player/tasks.md` from the approved plan and execute implementation from a compliant feature branch (`001-terminal-audio-recorder-player`) so prerequisite scripts pass.
- Replace `src/main.rs` with bootstrap code that initializes command parsing, dispatch, and optional TUI entry.
- Add `src/cli/` for argument definitions and command handlers.
- Add `src/audio/` for device enumeration, recording lifecycle, playback workflow, and WAV persistence/reading.
- Add `src/tui/` for Ratatui app state, event loop, and views.
- Add `src/model/` for shared feature entities.
- Add `src/error.rs` for domain/application error mapping to user-facing messages.
- Update `Cargo.toml` with minimal dependencies for CLI parsing, Ratatui UI, and audio capture/playback.
- Add/verify `.gitignore` Rust and universal patterns required by the implementation preflight checks (`target/`, `debug/`, `release/`, `*.log`, `.env*`, editor/system artifacts).

### Data Models / Contracts
- `AudioDevice { id, name, direction, is_available }`
- `RecordingSession { input_device_id, output_path, started_at, stopped_at, status }`
- `WavAsset { path, duration, format_metadata, created_at }`
- `PlaybackRequest { wav_path, output_device_id, volume, status }`

Command targets:
- `list-devices` -> prints categorized input/output devices with stable IDs.
- `record --output <path> [--input-device <id>]` -> starts/stops recording and finalizes WAV output.
- `play --file <path> [--output-device <id>] [--volume <0..100>]` -> plays WAV with validated volume and explicit completion/error result.

### File Structure
- `src/main.rs`
- `src/cli/mod.rs`, `src/cli/commands.rs`
- `src/audio/mod.rs`, `src/audio/devices.rs`, `src/audio/record.rs`, `src/audio/playback.rs`
- `src/tui/mod.rs`, `src/tui/app.rs`, `src/tui/view.rs`
- `src/model/mod.rs`
- `src/error.rs`
- `tests/unit/`
- `tests/integration/`

### Risks
- OS audio backend variance (device naming/availability differences).
- Interrupted recording leaving partial outputs without explicit cleanup.
- Divergence between CLI and TUI behavior if validation/audio logic is duplicated.

# Testing

### Validation Approach
- Use automated unit and integration tests for deterministic command/validation logic.
- Use manual host-hardware checks only for end-to-end microphone/speaker behavior that cannot be reliably simulated.

### Key Scenarios
- Implementation preflight passes: feature branch naming, tasks artifact presence, and checklist status checks succeed.
- Device enumeration returns stable IDs and direction classification.
- Recording produces non-empty WAV output that is externally playable.
- Playback succeeds for valid WAV inputs and fails clearly for invalid/missing files.
- Volume accepts valid values and rejects out-of-range values consistently.

### Edge Cases
- No available input or output devices.
- Permission denied / device busy during record or play.
- Immediate stop after recording begins.
- Invalid output path or overwrite conflicts.
- WAV format readable but unsupported by current output hardware path.

### Test Changes
- Add unit tests for validation logic (device lookup, volume bounds, path handling).
- Add integration tests for CLI contracts of `list-devices`, `record`, and `play`.
- Add a concise manual verification checklist for hardware-backed record/play runs.

# Delivery Steps

### ✓ Step 1: Satisfy implementation preflight and establish the runtime skeleton
Implementation prerequisites pass and a runnable Rust binary skeleton exists with deterministic command routing.
- Generate `specs/001-terminal-audio-recorder-player/tasks.md` aligned to this approved plan.
- Ensure implementation is executed on a valid feature branch (`001-terminal-audio-recorder-player`) so prerequisite checks pass.
- Create/verify `.gitignore` for Rust and universal ignore patterns expected by setup verification.
- Replace hello-world `src/main.rs` with bootstrap/dispatch and create baseline `cli`, `audio`, `tui`, `model`, and `error` modules.
- Add required dependencies in `Cargo.toml`.

### * Step 2: Implement microphone recording and WAV persistence
Users can record from selected/default input devices and save finalized WAV files safely.
- Implement input-device selection and validation.
- Implement recording lifecycle (start, stop, finalize WAV).
- Add failure-path cleanup to prevent corrupted artifacts.
- Add focused unit/integration tests for recording validation and output behavior.

###   Step 3: Implement device listing, playback, and volume handling
Users can list devices and replay WAV files through selected/default outputs with validated volume.
- Implement unified input/output device enumeration with stable IDs.
- Implement playback flow with output device routing.
- Implement volume validation/application and deterministic error responses.
- Add integration coverage for list/play/volume command paths.

###   Step 4: Integrate Ratatui workflows and finalize quality gates
Interactive Ratatui views support the same feature set without diverging from CLI behavior.
- Implement Ratatui app loop and views for device/status interaction.
- Reuse shared validation/audio services between TUI and CLI paths.
- Validate constitution gates and scenario-level verification evidence.
- Update user guidance/help text and hardware verification notes.