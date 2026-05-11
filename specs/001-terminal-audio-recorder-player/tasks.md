# Tasks: Terminal Audio Recorder & Player

**Input**: Design documents from `/specs/001-terminal-audio-recorder-player/`
**Prerequisites**: plan.md, spec.md

**Tests**: Include verification tasks for each user story. Automated tests are preferred; manual verification is acceptable only for trivial changes and MUST be documented.

## Phase 1: Setup (Shared Infrastructure)

- [X] T001 Create feature branch guard and prerequisite verification flow in project process docs (`specs/001-terminal-audio-recorder-player/tasks.md`)
- [X] T002 Add required Rust dependencies for CLI, TUI, and audio in `Cargo.toml`
- [X] T003 Create module scaffolding files for CLI/audio/TUI/model/error in `src/`

---

## Phase 2: Foundational (Blocking Prerequisites)

- [X] T004 Implement app-level error model and deterministic stdout/stderr mapping in `src/error.rs`
- [X] T005 Implement command routing and shared command contract types in `src/cli/mod.rs` and `src/cli/commands.rs`
- [X] T006 [P] Implement shared domain entities in `src/model/mod.rs`
- [X] T007 [P] Implement audio service interfaces and shared validation helpers in `src/audio/mod.rs`
- [X] T008 Define CLI input/output/error contract for core workflows in `src/main.rs`

---

## Phase 3: User Story 1 - Record and Save Audio (Priority: P1) 🎯 MVP

**Goal**: Record microphone audio and save a valid WAV file.

**Independent Test**: Start recording, stop recording, and verify a non-empty WAV file exists.

### Tests for User Story 1

- [X] T009 [P] [US1] Add unit tests for recording input/path validation in `tests/unit/record_validation.rs`
- [X] T010 [P] [US1] Add integration test for `record` command success/failure contract in `tests/integration/record_cli.rs`

### Implementation for User Story 1

- [ ] T011 [US1] Implement input device lookup and selection in `src/audio/devices.rs`
- [X] T012 [US1] Implement recording lifecycle and WAV persistence in `src/audio/record.rs`
- [X] T013 [US1] Add cleanup logic for interrupted/failed recordings in `src/audio/record.rs`
- [X] T014 [US1] Wire `record` command handler to audio service in `src/cli/commands.rs`

---

## Phase 4: User Story 2 - Replay Recorded Audio (Priority: P2)

**Goal**: Play a WAV file through selected/default output device.

**Independent Test**: Run playback with valid WAV and verify deterministic success and error behavior.

### Tests for User Story 2

- [ ] T015 [P] [US2] Add integration test for `play` command valid/missing file paths in `tests/integration/play_cli.rs`
- [ ] T016 [P] [US2] Add unit tests for playback request validation in `tests/unit/playback_validation.rs`

### Implementation for User Story 2

- [ ] T017 [US2] Implement WAV playback pipeline and output device routing in `src/audio/playback.rs`
- [ ] T018 [US2] Wire `play` command handler to playback service in `src/cli/commands.rs`

---

## Phase 5: User Story 3 - Manage Devices and Volume (Priority: P3)

**Goal**: List devices and apply validated playback volume.

**Independent Test**: List devices and run playback with in-range and out-of-range volume values.

### Tests for User Story 3

- [ ] T019 [P] [US3] Add integration test for `list-devices` output contract in `tests/integration/list_devices_cli.rs`
- [ ] T020 [P] [US3] Add unit tests for volume bounds and device-id validation in `tests/unit/device_volume_validation.rs`

### Implementation for User Story 3

- [ ] T021 [US3] Implement stable device enumeration for input/output in `src/audio/devices.rs`
- [ ] T022 [US3] Implement volume validation and application logic in `src/audio/playback.rs`
- [ ] T023 [US3] Wire `list-devices` command and volume-aware playback options in `src/cli/commands.rs`

---

## Phase 6: Ratatui Integration and Polish

- [ ] T024 Implement Ratatui app state/event loop in `src/tui/app.rs`
- [ ] T025 Implement Ratatui views for status and device interaction in `src/tui/view.rs`
- [ ] T026 Wire TUI entry mode in `src/tui/mod.rs` and `src/main.rs`
- [ ] T027 Update help text and usage docs in `README.md`
- [ ] T028 Run full test suite and capture verification evidence in `tests/`
