# Feature Specification: Terminal Audio Recorder & Player

**Feature Branch**: `[001-terminal-audio-recorder-player]`  
**Created**: 2026-05-11  
**Status**: Updated  
**Input**: User description: "Terminal Audio Recorder & Player. Description: Build a CLI application that allows users to record audio from their microphone, save it as a WAV file, and replay it through the system speakers. The app should include volume controls and a listing of available audio devices."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Record and Save Audio (Priority: P1)

As a CLI user, I can record audio from my microphone and save it to a WAV file so
I can capture voice/audio notes from the terminal.

**Why this priority**: Recording and saving audio is the core value of the feature;
without this flow, playback and controls are not useful.

**Independent Test**: Can be fully tested by running `record --output <path>`,
waiting for command completion, and confirming a valid ~10-second WAV file is
created.

**Acceptance Scenarios**:

1. **Given** at least one microphone is available, **When** the user runs the
   record command with an output filename, **Then** the system records from the
   default input device for ~10 seconds and creates the target WAV file.
2. **Given** recording completes successfully, **When** the command exits,
   **Then** the system prints `recorded <output>` and leaves a valid non-empty
   WAV file at the requested path.
3. **Given** an invalid output path or capture failure occurs, **When**
   recording is started, **Then** the system fails with a clear error and does
   not leave partial output artifacts.

---

### User Story 2 - Replay Recorded Audio (Priority: P2)

As a CLI user, I can play a WAV file through my system output so I can verify or
listen to recordings without leaving the terminal workflow.

**Why this priority**: Playback is the second essential workflow and directly
complements recording by enabling immediate verification.

**Independent Test**: Can be tested by verifying deterministic command behavior:
success path for supported WAV files and clear non-zero failure for missing
files or invalid volume.

**Acceptance Scenarios**:

1. **Given** a valid supported WAV file exists, **When** the user runs the play
   command, **Then** the system plays audio through the default output device
   and prints `played <file>`.
2. **Given** a missing or unreadable WAV file, **When** the user runs the play
   command, **Then** the system returns a clear error without crashing.
3. **Given** an out-of-range volume value, **When** the user runs the play
   command, **Then** the system rejects the request with a deterministic error.

---

### User Story 3 - Manage Devices and Volume (Priority: P3)

As a CLI user, I can list available input/output audio devices and control
playback volume so I can inspect available endpoints and choose listening level.

**Why this priority**: Device visibility and volume controls improve usability
but are secondary to basic record/play capability.

**Independent Test**: Can be tested by running `list-devices`, checking for
input/output rows, and validating playback acceptance/rejection across volume
boundaries.

**Acceptance Scenarios**:

1. **Given** at least one audio endpoint is available, **When** the user runs
   the list-devices command, **Then** the system outputs tabular rows containing
   input/output direction and availability.
2. **Given** a playback command and explicit volume value, **When** the volume
   is within the accepted range, **Then** playback uses that volume level.
3. **Given** explicit `--input-device` or `--output-device` arguments are
   provided, **When** commands execute today, **Then** playback/recording still
   use default host devices (device-id routing is deferred).

---

### Edge Cases

- No default input device available when starting recording.
- No default output device available when starting playback.
- Output path parent directory does not exist.
- Microphone stream starts but captures zero samples.
- Unsupported capture sample format from input stream.
- Missing input WAV file for playback.
- Unsupported WAV source format (anything other than int16/float32).
- Output device exposes unsupported runtime sample format.
- Volume value over `100`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide `list-devices` and print tab-separated rows
  containing device id, direction (`input`/`output`), name, and availability.
- **FR-002**: System MUST validate `record --output` path input (non-empty path,
  existing parent directory when specified).
- **FR-003**: System MUST record from the default input device for a fixed
  duration of approximately 10 seconds.
- **FR-004**: System MUST write captured samples to a temporary WAV and rename to
  final output path on success.
- **FR-005**: System MUST remove temporary recording output when recording fails
  before completion.
- **FR-006**: System MUST validate `play --file` existence before attempting
  playback.
- **FR-007**: System MUST validate `play --volume` and reject values above `100`
  with deterministic error text.
- **FR-008**: System MUST decode and play WAV sources only for supported source
  formats (`Int/16-bit`, `Float/32-bit`), rejecting unsupported formats.
- **FR-009**: System MUST scale playback amplitude by `volume / 100` and convert
  to the runtime output sample format (`I16`, `U16`, `F32`).
- **FR-010**: System MUST currently use default host devices for recording and
  playback; `--input-device`/`--output-device` options are accepted but not yet
  routed to device selection.
- **FR-011**: System MUST emit deterministic stdout success messages
  (`recorded <path>`, `played <path>`) and stderr errors (`error: <message>`).
- **FR-012**: System MUST display help/usage guidance for all user-facing
  commands through clap-generated CLI help.

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

- **Audio Device**: Represents an input or output hardware endpoint with fields
  such as stable device identifier, display name, direction (input/output), and
  availability status.
- **Recording Session**: Represents one capture operation with selected input
  device reference (currently default), output path, and final status.
- **WAV Asset**: Represents a saved audio file with path, duration,
  and compatibility-relevant format metadata.
- **Playback Request**: Represents one replay operation with source WAV path,
  output device reference (currently default), requested volume, and completion
  status.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Integration test for `record` confirms command runtime is at least
  9.5 seconds and generated WAV duration is within ±0.1 seconds of 10 seconds.
- **SC-002**: Playback command rejects missing files with non-zero exit and
  deterministic error text containing `file does not exist`.
- **SC-003**: Playback validation rejects out-of-range volume (>100) with
  deterministic error text containing `volume must be between 0 and 100`.
- **SC-004**: List-devices integration test confirms output contains at least one
  `input` or `output` direction row.

## Assumptions

- Users run the tool in an environment with microphone and speaker access
  permitted at OS level.
- Real-time audio effects, editing, trimming, and multi-track mixing are out of
  scope for this feature.
- The feature targets local, single-user CLI usage and does not require cloud
  upload, sharing, or remote streaming.
- WAV is the only required persisted output format for this version.
- Device-selection flags remain visible in CLI and documentation while routing
  implementation is tracked as future work.
