# Feature Specification: TUI Recorder Workflow

**Feature Branch**: `[002-tui-recorder-ui]`  
**Created**: 2026-05-14  
**Status**: Draft  
**Input**: User description: "Transform the application to an TUI only application. Main only starts the UI. In the UI we have a Record Play and Stop Button. The UI contains also the list of stored WAF files. Each Record/Stop generates a new file."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Record and Save from TUI (Priority: P1)

As a user, I can start recording and then stop recording from a TUI so I can
quickly capture audio without command-line subcommands.

**Why this priority**: Recording is the core value and the primary workflow users
expect from the application.

**Independent Test**: Start the app, select `Record`, then `Stop`, and confirm a
new WAV file appears in the TUI file list and on disk.

**Acceptance Scenarios**:

1. **Given** the app is open in the TUI, **When** the user selects `Record`,
   **Then** recording starts and the UI clearly indicates active recording state.
2. **Given** recording is active, **When** the user selects `Stop`, **Then**
   recording ends and a new WAV file is saved.
3. **Given** recording ends successfully, **When** the file list refreshes,
   **Then** the new WAV file is shown in the stored files list.

---

### User Story 2 - Play Stored WAV Files and Toggle Playback Modes (Priority: P2)

As a user, I can select and play a stored WAV file from the TUI, and cycle through playback modes (Single, Continuous, Loop), so I can listen to recordings individually or as a sequence.

**Why this priority**: Playback is the second essential workflow. Adding playback modes enhances usability for reviewing multiple recordings.

**Independent Test**: Start the app with at least one WAV file present, select a file, choose `Play`. While playing, press 'p' again and verify the mode indicator changes to `[C]`. Press 'p' again and verify it changes to `[L]`. Verify that in `[C]` and `[L]` modes, the next track starts automatically when the current one finishes.

**Acceptance Scenarios**:

1. **Given** at least one WAV file exists, **When** the user selects it and chooses `Play`, **Then** the file playback begins in Single mode (`[ Play [ ]]`).
2. **Given** playback is running, **When** the user presses 'p' again, **Then** the playback mode cycles to Continuous (`[ Play [C]]`).
3. **Given** playback is in Continuous mode, **When** the user presses 'p' again, **Then** the playback mode cycles to Loop (`[ Play [L]]`).
4. **Given** playback is in Loop mode, **When** the user presses 'p' again, **Then** the playback mode cycles back to Single (`[ Play [ ]]`).
5. **Given** playback is in Continuous mode, **When** the current track finishes, **Then** the system automatically starts playing the next track in the list.
6. **Given** playback is in Loop mode, **When** the last track in the list finishes, **Then** the system automatically starts playing the first track in the list.
7. **Given** the selected file cannot be played, **When** the user chooses `Play`, **Then** the UI shows a clear error message and remains usable.

---

### User Story 3 - Launch Directly into TUI (Priority: P3)

As a user, I can run the application and immediately see the TUI so startup is
simple and consistent.

**Why this priority**: This ensures the app follows the requested TUI-only
experience, while still being lower priority than core record/play actions.

**Independent Test**: Run the application entry point and verify it launches the
TUI directly with action controls and stored-file list visible.

**Acceptance Scenarios**:

1. **Given** the application starts, **When** `main` executes, **Then** it
   launches the TUI as the primary interface.
2. **Given** the TUI is shown, **When** the app is idle, **Then** `Record`,
   `Play`, and `Stop` actions are available in the interface.

---

### User Story 4 - Delete Recording (Priority: P2)

As a user, I can delete a recording from the TUI so I can manage my storage and remove unwanted audio.

**Why this priority**: Managing recordings is a fundamental part of the recording workflow.

**Independent Test**: Select a file in the TUI, press 'd', and confirm the file is removed from the list and from disk.

**Acceptance Scenarios**:

1. **Given** at least one WAV file exists, **When** the user selects it and presses 'd', **Then** the file is deleted.
2. **Given** a file is deleted, **When** the file list refreshes, **Then** the file is no longer visible in the list.

---

### User Story 5 - Amplify Recording (Priority: P3)

As a user, I can amplify a recording's volume so I can hear quiet recordings more clearly.

**Why this priority**: Provides basic post-processing functionality to improve the utility of recordings.

**Independent Test**: Select a file, press 'a', and confirm that subsequent playback is louder (or check the file modified date).

**Acceptance Scenarios**:

1. **Given** a WAV file is selected, **When** the user presses 'a', **Then** the file is processed and its volume is increased.
2. **Given** amplification completes, **When** the user plays the file, **Then** the volume is noticeably louder.

---

### Edge Cases

- User selects `Stop` when no recording is active.
- User selects `Play` when no WAV file is selected.
- No stored WAV files exist on first launch.
- A recording is interrupted by device or permission failure.
- File list contains non-WAV files in the same directory.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST run as a TUI-only application for normal user
  interaction.
- **FR-002**: `main` MUST only initialize and start the TUI workflow.
- **FR-003**: TUI MUST provide visible `Record`, `Play` (with mode indicator), and `Stop` controls.
- **FR-004**: TUI MUST display a list of stored WAV files available for
  playback.
- **FR-005**: System MUST create one new WAV file for each completed
  Record-to-Stop cycle.
- **FR-006**: Newly created WAV files MUST appear in the stored-files list
  without requiring application restart.
- **FR-007**: TUI MUST prevent invalid actions (for example, `Stop` without an
  active recording or playback) and show clear user-facing feedback.
- **FR-011**: System MUST support cycling playback modes (Single, Continuous, Loop) by pressing 'p' during active playback.
- **FR-012**: System MUST automatically advance to the next recording in Continuous and Loop modes upon track completion.
- **FR-013**: System MUST restart from the first recording when the last one ends if Loop mode is active.
- **FR-008**: When playback or recording fails, TUI MUST present a clear error
  state and allow the user to continue using the interface.
- **FR-009**: TUI MUST provide a way to delete selected recordings.
- **FR-010**: TUI MUST provide a way to amplify the volume of selected recordings.

### Constitution Alignment *(mandatory)*

- **CA-001 (Rust-First)**: Any non-Rust implementation or new dependency MUST be
  explicitly justified and approved in planning artifacts.
- **CA-002 (CLI-First)**: User-facing behavior MUST define deterministic command
  input/output and explicit error semantics.
- **CA-003 (Verification Gate)**: Each user story MUST define independent
  verification (automated tests preferred; manual-only allowed for trivial scope).
- **CA-004 (Integration Safety)**: Contract or cross-module changes MUST identify
  required integration validation.
- **CA-005 (Version Discipline)**: Behavior-breaking changes MUST be called out
  with semantic-versioning impact.

### Key Entities *(include if feature involves data)*

- **Recording Session**: Represents one active or completed recording operation,
  including start time, stop time, and output WAV path.
- **Stored WAV File**: Represents a persisted playable WAV asset shown in the
  TUI list (name, path, creation order, availability).
- **Playback Selection**: Represents the currently selected file and its playback
  state (idle, playing, failed, completed).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of application launches open directly into the TUI without
  requiring additional user commands.
- **SC-002**: In user validation, at least 95% of Record→Stop operations result
  in exactly one newly created WAV file.
- **SC-003**: In user validation, at least 95% of attempted play actions on valid
  stored WAV files start playback successfully.
- **SC-004**: At least 90% of users can complete record, stop, and replay of a
  file in under 2 minutes on first attempt.

## Assumptions

- WAV remains the required and only persisted recording format for this feature.
- The stored-files list is sourced from a single application-managed recordings
  location.
- Single-user local desktop usage is the target context for this release.