---
sessionId: session-260513-185210-rs7v
isActive: false
---

# Requirements

### Overview & Goals
Update the Speckit artifacts under `specs/001-terminal-audio-recorder-player/` so they reflect the behavior introduced in commit `0ed3069` (`Apply Fix Record/Play`) and restore planning/docs consistency.

### In Scope
- Align `spec.md` with current CLI/audio behavior implemented in:
  - `src/audio/record.rs` (real microphone capture, fixed ~10s recording window, temp-file rename cleanup)
  - `src/audio/playback.rs` (real WAV playback pipeline, per-format output conversion, volume scaling, unsupported-format errors)
  - `tests/integration/record_cli.rs` and `tests/integration/play_cli.rs` (observable contract expectations)
- Regenerate/fill planning artifacts expected by `/speckit.plan`:
  - `plan.md` (technical context + constitution check + concrete structure)
  - `research.md`, `data-model.md`, `quickstart.md`, `contracts/*`
- Update `tasks.md` statuses/scope so checkboxes match what is actually implemented vs still pending.
- Update `.junie/AGENTS.md` marker block to reference the current plan path.

### Out of Scope
- Any Rust source code changes in `src/` or runtime behavior changes.
- New feature implementation beyond documentation/spec alignment.

### Extension Hook Context
From `.specify/extensions.yml`, `hooks.before_plan` contains one executable optional hook:
- Extension: `git`
- Command: `/speckit.git.commit`
- Description: `Auto-commit before implementation planning`
- Prompt: `Commit outstanding changes before planning?`

# Technical Design

### Current Implementation (Observed)
- Project is a single Rust CLI crate (`Cargo.toml`, `src/main.rs`, `src/cli/*`, `src/audio/*`, `tests/*`).
- Command routing pattern: `main.rs` parses Clap args and dispatches into `cli::commands` functions, which call `audio::*` services and return `AppError`.
- Current behavior relevant to specs:
  - `record` records from default input device for a fixed duration (`DEFAULT_RECORDING_DURATION = 10s`) and writes captured samples to WAV (`src/audio/record.rs`).
  - `play` reads WAV samples, validates format/volume/file existence, streams via default output (`src/audio/playback.rs`).
  - `input_device` / `output_device` CLI args are currently accepted but not actively used by audio selection logic.
- Spec artifacts are incomplete/stale:
  - `plan.md` is still template content.
  - `research.md`, `data-model.md`, `quickstart.md`, `contracts/` are missing.
  - `tasks.md` has unchecked tasks that overlap with already-implemented behavior.

### Key Decisions
1. **Treat `HEAD` behavior as source of truth for this update**
   - Specs will reflect what the product currently does, not aspirational behavior that is still unchecked in tasks.
2. **Document unimplemented argument-driven device selection explicitly as deferred scope**
   - Keep CLI contract honest while preserving future roadmap entries in tasks.
3. **Use Speckit phase artifacts as the canonical design trail**
   - Fill `plan.md`, generate `research.md`, `data-model.md`, `quickstart.md`, and CLI contract docs to satisfy the expected planning workflow.

### Proposed Changes by File
- `specs/001-terminal-audio-recorder-player/plan.md`
  - Fill Technical Context with concrete values (Rust 2024, `clap/cpal/hound/thiserror`, file-based WAV output, `cargo test`, CLI app constraints).
  - Fill project structure section with real tree (`src/audio`, `src/cli`, `src/model`, `tests/integration`, `tests/unit`).
  - Re-check Constitution gates against the current implementation.
- `specs/001-terminal-audio-recorder-player/research.md` (new)
  - Record decisions/rationales/alternatives for capture duration strategy, playback sample-format handling, deterministic error messaging.
- `specs/001-terminal-audio-recorder-player/data-model.md` (new)
  - Formalize `AudioDevice`, `RecordingSession`, `WavAsset`, `PlaybackRequest` from `src/model/mod.rs` plus behavioral constraints observed in services.
- `specs/001-terminal-audio-recorder-player/contracts/cli.md` (new)
  - Define command contracts for `list-devices`, `record`, `play`, `tui` with expected stdout/stderr semantics and validation errors.
- `specs/001-terminal-audio-recorder-player/quickstart.md` (new)
  - Add runnable command examples and expected success/error outputs matching current command handlers.
- `specs/001-terminal-audio-recorder-player/spec.md`
  - Update user stories/acceptance details where needed to match fixed-duration recording and default-device behavior.
  - Keep deferred capabilities clearly marked as future tasks rather than current guarantees.
- `specs/001-terminal-audio-recorder-player/tasks.md`
  - Reconcile checkbox state with implemented commit changes (`record`/`play` pipeline and integration wiring) and leave true gaps unchecked.
- `.junie/AGENTS.md`
  - Replace marker content to reference the active plan path per Speckit instruction.

### Risks & Mitigations
- **Risk:** Overstating support for selected device IDs when code currently ignores device arguments.
  - **Mitigation:** Explicitly label device-id routing as pending in `spec.md` and `tasks.md`.
- **Risk:** Tasks checklist drift after partial implementation.
  - **Mitigation:** Trace each task against concrete files/tests before toggling status.

# Testing

### Validation Approach
- Perform a spec-to-code consistency review for each updated requirement and acceptance scenario.
- Validate command contracts against existing tests and handlers:
  - `tests/integration/record_cli.rs`
  - `tests/integration/play_cli.rs`
  - `tests/integration/list_devices_cli.rs`
  - `src/cli/commands.rs`

### Key Scenarios to Verify in Docs
- Recording behavior documents fixed ~10s duration and successful WAV creation.
- Playback behavior documents supported WAV formats, missing-file errors, and volume bounds.
- Device-selection capability is documented as current default-device behavior with explicit future work.
- Plan and Constitution sections contain no unresolved `NEEDS CLARIFICATION` markers.

### Completion Checks
- Required Speckit artifacts exist in `specs/001-terminal-audio-recorder-player/` (`plan.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`).
- `tasks.md` statuses match current implementation state and test coverage expectations.

# Delivery Steps

### ✓ Step 1: Rebuild planning baseline from current branch state
Implementation planning context is reconstructed from `001-terminal-audio-recorder-player` and commit `0ed3069`.
- Capture planning inputs from `.specify/scripts/bash/setup-plan.sh --json` (`FEATURE_SPEC`, `IMPL_PLAN`, `SPECS_DIR`, `BRANCH`).
- Audit current implementation diffs and existing specs to create a precise mismatch list.
- Carry forward extension-hook context (`before_plan` optional `/speckit.git.commit`) for operator visibility.

### ✓ Step 2: Generate and fill Speckit Phase 0/1 artifacts
`plan.md`, `research.md`, `data-model.md`, `contracts/*`, and `quickstart.md` describe the implemented CLI/audio architecture without unresolved gaps.
- Populate `plan.md` Technical Context, concrete project structure, and Constitution Check using repository facts.
- Create `research.md` with explicit decisions/rationale/alternatives for recording/playback behavior and constraints.
- Create `data-model.md` and `contracts/cli.md` from `src/model/mod.rs` and CLI command contracts.
- Create `quickstart.md` with validated usage paths and expected outputs.
- Update `.junie/AGENTS.md` marker block to point to the active implementation plan path.

### ✓ Step 3: Align feature spec and task tracking with implemented behavior
`spec.md` and `tasks.md` accurately represent what is implemented now versus what remains pending.
- Revise `spec.md` stories/acceptance criteria/edge cases to match fixed-duration microphone recording and current playback pipeline semantics.
- Explicitly document deferred device-selection routing as future work, avoiding false “implemented” claims.
- Reconcile `tasks.md` checkbox states against actual code and tests so progress tracking is trustworthy.
- Perform a final consistency pass across all spec artifacts and include after-plan hook visibility (`after_plan` optional `/speckit.git.commit`).