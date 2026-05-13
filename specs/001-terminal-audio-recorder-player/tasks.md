# Tasks: Terminal Audio Recorder & Player

**Input**: Design documents from `/specs/001-terminal-audio-recorder-player/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli.md, quickstart.md

**Tests**: Automated verification tasks are required by the specification success criteria.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Ensure crate metadata and module layout are ready for feature work.

- [ ] T001 Confirm CLI/audio/TUI dependencies and feature flags in `Cargo.toml`
- [ ] T002 Confirm crate entrypoints and module exports in `src/main.rs` and `src/lib.rs`
- [ ] T003 [P] Confirm feature documentation entrypoints in `README.md` and `specs/001-terminal-audio-recorder-player/quickstart.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared contracts and error handling that block all user stories.

- [ ] T004 Define deterministic application error variants/messages in `src/error.rs`
- [ ] T005 [P] Define CLI command argument contracts in `src/cli/mod.rs`
- [ ] T006 [P] Define command dispatch and success-output helpers in `src/cli/commands.rs`
- [ ] T007 [P] Define shared entities (`AudioDevice`, `RecordingSession`, `WavAsset`, `PlaybackRequest`) in `src/model/mod.rs`
- [ ] T008 Define audio module service interfaces and exports in `src/audio/mod.rs`

**Checkpoint**: Foundational interfaces are complete; user stories can be implemented.

---

## Phase 3: User Story 1 - Record and Save Audio (Priority: P1) 🎯 MVP

**Goal**: Record from microphone and persist a valid WAV file via CLI.

**Independent Test**: Run `record --output <path>` and confirm a valid ~10s WAV is created with deterministic success/error behavior.

### Tests for User Story 1

- [ ] T009 [P] [US1] Add/validate unit tests for record request validation in `tests/unit/record_validation.rs`
- [ ] T010 [P] [US1] Add/validate integration tests for `record` CLI contract in `tests/integration/record_cli.rs`

### Implementation for User Story 1

- [ ] T011 [US1] Implement record request validation and temp-path planning in `src/audio/record.rs`
- [ ] T012 [US1] Implement default-input capture flow with fixed ~10s duration in `src/audio/record.rs`
- [ ] T013 [US1] Implement WAV writer finalization and temp-file cleanup/rename behavior in `src/audio/record.rs`
- [ ] T014 [US1] Wire `record` command arguments/outputs to audio service in `src/cli/commands.rs`

**Checkpoint**: User Story 1 is independently functional and testable.

---

## Phase 4: User Story 2 - Replay Recorded Audio (Priority: P2)

**Goal**: Play supported WAV files with deterministic validation and volume scaling.

**Independent Test**: Run `play` against valid/invalid inputs and verify deterministic success, missing-file, unsupported-format, and volume-range behavior.

### Tests for User Story 2

- [ ] T015 [P] [US2] Add/validate integration tests for `play` CLI contract in `tests/integration/play_cli.rs`
- [ ] T016 [P] [US2] Add/validate unit tests for playback validation rules in `tests/unit/playback_validation.rs`

### Implementation for User Story 2

- [ ] T017 [US2] Implement playback request validation (`file` existence and `volume` bounds) in `src/audio/playback.rs`
- [ ] T018 [US2] Implement WAV decode constraints and output-format conversion pipeline in `src/audio/playback.rs`
- [ ] T019 [US2] Implement volume scaling and playback completion signaling in `src/audio/playback.rs`
- [ ] T020 [US2] Wire `play` command arguments/outputs to playback service in `src/cli/commands.rs`

**Checkpoint**: User Story 2 is independently functional and testable.

---

## Phase 5: User Story 3 - Manage Devices and Volume (Priority: P3)

**Goal**: Provide device listing and maintain documented device-flag behavior with validated volume usage.

**Independent Test**: Run `list-devices`, verify output row shape/directions, and verify explicit device flags are accepted while default devices are still used.

### Tests for User Story 3

- [ ] T021 [P] [US3] Add/validate integration tests for `list-devices` output contract in `tests/integration/list_devices_cli.rs`
- [ ] T022 [P] [US3] Add/validate integration assertions for accepted-but-deferred device flags in `tests/integration/record_cli.rs` and `tests/integration/play_cli.rs`

### Implementation for User Story 3

- [ ] T023 [US3] Implement/confirm deterministic device rows and availability mapping in `src/audio/devices.rs`
- [ ] T024 [US3] Ensure CLI exposes `--input-device` and `--output-device` while routing remains default-device behavior in `src/cli/mod.rs` and `src/audio/record.rs` and `src/audio/playback.rs`
- [ ] T025 [US3] Wire list-devices rendering and volume-related CLI options in `src/cli/commands.rs`

**Checkpoint**: User Story 3 is independently functional and testable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and docs consistency across stories.

- [ ] T026 [P] Align CLI examples and known limitations in `README.md` and `specs/001-terminal-audio-recorder-player/contracts/cli.md`
- [ ] T027 [P] Align quickstart verification commands and expected outputs in `specs/001-terminal-audio-recorder-player/quickstart.md`
- [ ] T028 Run regression suites for documented contracts in `tests/integration.rs`, `tests/unit/record_validation.rs`, and `tests/unit/playback_validation.rs`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies.
- **Phase 2 (Foundational)**: Depends on Phase 1; blocks all user stories.
- **Phase 3+ (User Stories)**: Depend on Phase 2 completion.
  - Priority order for incremental delivery: US1 → US2 → US3.
  - Stories may be parallelized after Phase 2 when team capacity allows.
- **Phase 6 (Polish)**: Depends on completion of selected user stories.

### User Story Dependencies

- **US1 (P1)**: Depends only on Foundational phase.
- **US2 (P2)**: Depends only on Foundational phase; can be validated independently with fixture WAV files.
- **US3 (P3)**: Depends only on Foundational phase; uses list-devices and CLI flag behavior contracts.

### Within-Story Ordering Rules

- Write/validate tests before implementation.
- Implement validation before stream/control flow.
- Wire command handlers after service behavior exists.

---

## Parallel Opportunities

- **Setup**: `T003` can run in parallel with `T001`/`T002`.
- **Foundational**: `T005`, `T006`, and `T007` can run in parallel after `T004` framing.
- **US1**: `T009` and `T010` can run in parallel.
- **US2**: `T015` and `T016` can run in parallel.
- **US3**: `T021` and `T022` can run in parallel.
- **Polish**: `T026` and `T027` can run in parallel before `T028`.

---

## Parallel Example: User Story 2

```bash
# Run parallel test authoring for User Story 2
Task: "T015 [US2] tests/integration/play_cli.rs"
Task: "T016 [US2] tests/unit/playback_validation.rs"

# Then implement playback pipeline tasks sequentially
Task: "T017 [US2] src/audio/playback.rs"
Task: "T018 [US2] src/audio/playback.rs"
Task: "T019 [US2] src/audio/playback.rs"
Task: "T020 [US2] src/cli/commands.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 and Phase 2.
2. Complete US1 tasks (`T009`-`T014`).
3. Validate US1 independently.

### Incremental Delivery

1. Deliver MVP (US1).
2. Add US2 playback behavior (`T015`-`T020`) and validate independently.
3. Add US3 device-management behavior (`T021`-`T025`) and validate independently.
4. Execute polish tasks (`T026`-`T028`) before final release.
