# Feature Specification: Continuous Sound-Activated Recording

**Feature Branch**: `003-continuous-recording`  
**Created**: 2026-05-15  
**Status**: Draft  
**Input**: User description: "Create a new 003-continous-recording spec based on instructions.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Start Monitoring and Save Sound Segments (Priority: P1)

As a user, I can start sound-activated monitoring from the TUI so that meaningful sounds are automatically saved as individual WAV files without storing silence.

**Why this priority**: This is the core feature value. Without the ability to start monitoring and produce saved clips, the feature delivers nothing. All other stories depend on this one working first.

**Independent Test**: Start monitoring, play a sound above the detection threshold for at least the minimum clip duration, wait for the silence timeout to expire, then verify exactly one WAV file was created in the recordings directory and that file contains audio rather than silence. Run monitoring for a test interval of pure silence and verify no new WAV files are created.

**Acceptance Scenarios**:

1. **Given** the app is idle, **When** the user activates the Monitor action in the TUI, **Then** the UI transitions to a "Monitoring" state within 1 second and begins listening for sound.
2. **Given** the app is monitoring and a sound above the threshold is detected, **When** the sound begins, **Then** the UI transitions to "Capturing" state and the app starts buffering audio.
3. **Given** the app is capturing a sound segment, **When** silence continues beyond the silence timeout, **Then** the app finalizes the WAV file, saves it to the recordings directory, the UI returns to "Monitoring" state, and the recordings list is refreshed to include the new file.
4. **Given** the app is monitoring, **When** no sound above the threshold occurs for an extended period, **Then** no WAV files are created and memory use stays bounded.

---

### User Story 2 - Stop Monitoring Safely (Priority: P2)

As a user, I can stop monitoring at any time so that the app returns to idle without losing a valid in-progress sound segment.

**Why this priority**: Users must be able to exit the monitoring mode reliably. Safe teardown — including proper finalization of any active segment — is required before any other TUI actions become available again.

**Independent Test**: Start monitoring, generate a sound event long enough to meet the minimum duration threshold, then stop monitoring while the segment is still being captured. Verify the segment is finalized and saved. Repeat stopping while the app is in silent monitoring (no active capture) and verify no file is written and the app returns to idle cleanly.

**Acceptance Scenarios**:

1. **Given** the app is monitoring with no active capture, **When** the user activates the Stop action, **Then** the app transitions to idle within 1 second and no file is written.
2. **Given** the app is capturing an active sound segment that meets the minimum duration, **When** the user activates the Stop action, **Then** the app finalizes and saves the segment, updates the recordings list, then transitions to idle.
3. **Given** the app is capturing a segment that does not yet meet the minimum duration, **When** the user activates the Stop action, **Then** the short segment is discarded, the UI shows a status message explaining the discard, and the app transitions to idle.

---

### User Story 3 - Avoid Noisy or Useless Recordings (Priority: P3)

As a user, I want the app to ignore silence and very short noise bursts so that the recordings list contains only meaningful sound events rather than accidental artifacts.

**Why this priority**: Without minimum-duration filtering and silence timeout behavior, the recordings list fills with useless files. This story is independently verifiable and adds meaningful product quality but is not required for the raw capture path to function.

**Independent Test**: Generate a noise burst shorter than the minimum clip duration and verify no WAV file is saved and the UI shows a "segment too short, discarded" status message. Generate a sound that starts, pauses briefly (within the silence timeout), then resumes — verify this produces exactly one WAV file rather than two.

**Acceptance Scenarios**:

1. **Given** the app is monitoring, **When** a brief noise spike below the minimum duration threshold is detected, **Then** the capture is started, the silence timeout passes, the segment is discarded because it is too short, and the UI shows a clear status message about the discard.
2. **Given** the app is capturing a sound segment, **When** a short pause occurs that is shorter than the silence timeout, **Then** the app continues capturing the same segment rather than finalizing it, producing one file that spans the pause.
3. **Given** the app is monitoring in a noisy environment where the threshold is continuously exceeded, **When** this is detected, **Then** the UI shows a clear warning that the threshold may be set too low.

---

### User Story 4 - Maintain Playback and Recordings List Workflow (Priority: P4)

As a user, I can see automatically saved sound-activated segments in the recordings list and play them back using the existing TUI workflow.

**Why this priority**: Integration with the existing recordings list and playback flow ensures the new monitoring mode is a first-class citizen of the existing UX. This is independently verifiable once P1 produces saved files.

**Independent Test**: After a monitoring session produces one or more sound-activated WAV files, open the recordings list in the TUI and verify the new files appear, sorted by newest-first. Select one file and start playback using the existing Play action. Verify that monitoring cannot be started while playback is active and playback cannot be started while monitoring is active.

**Acceptance Scenarios**:

1. **Given** monitoring has saved a new WAV segment, **When** the user views the recordings list in the TUI, **Then** the new file appears in the list sorted by newest-first alongside any manually recorded files.
2. **Given** a sound-activated segment appears in the recordings list, **When** the user selects it and activates Play, **Then** it plays back using the same flow as any manually recorded file.
3. **Given** monitoring is active, **When** the user attempts to start playback, **Then** the action is rejected and the UI displays a clear status message explaining the conflict.
4. **Given** playback is active, **When** the user attempts to start monitoring, **Then** the action is rejected and the UI displays a clear status message explaining the conflict.

---

### Edge Cases

- What happens when no default input device is available at the time monitoring is started?
- What happens when microphone permission is denied by the operating system?
- What happens when the audio stream opens successfully but produces no samples?
- What happens when the audio input provides samples in an unsupported format?
- What happens when the recordings directory cannot be created or written to?
- What happens when a WAV file fails to be created, written, finalized, or renamed from its temporary path?
- What happens when the audio device disconnects while monitoring is active?
- What happens when the recordings list fails to refresh after a segment is saved?
- What happens when the sound threshold is set so low that monitoring captures continuously without stopping?
- What happens when the sound threshold is set so high that expected sounds are never detected?
- What happens when monitoring is stopped while the app is in the silent-monitoring state (no active capture)?
- What happens when monitoring is stopped while a segment is being actively captured?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The TUI MUST expose a Monitor action that starts sound-activated monitoring when the app is idle.
- **FR-002**: Activating the Monitor action MUST be rejected with a visible status message when playback is currently active.
- **FR-003**: The TUI MUST display a distinct status label for each monitoring state: Idle, Monitoring (listening, no sound detected), Capturing (sound detected, recording in progress), and Stopping/Finalizing.
- **FR-004**: The app MUST use the system default audio input device for monitoring; no per-session device selection is required.
- **FR-005**: While monitoring is active, the app MUST continuously read audio samples from the input device without accumulating all samples in memory across the session.
- **FR-006**: Sound detection MUST be performed incrementally on each incoming batch of audio samples so that detection latency is bounded.
- **FR-007**: When a sound event above the detection threshold is detected, the app MUST begin buffering a pre-roll of audio captured just before the detection point so that the beginning of the sound is not clipped.
- **FR-008**: The app MUST maintain a rolling pre-roll buffer of approximately 300–500 ms so that audio immediately preceding a threshold crossing is included in the saved segment.
- **FR-009**: After a sound event begins, the app MUST continue appending audio to the current segment while sound above the threshold continues.
- **FR-010**: After sound falls below the threshold, the app MUST wait for a configurable silence timeout (default approximately 1–2 seconds) before finalizing the current segment, so that brief pauses within a single sound event do not cause it to be split into multiple files.
- **FR-011**: If sound resumes above the threshold before the silence timeout expires, the app MUST continue appending to the same segment rather than starting a new one.
- **FR-012**: When the silence timeout expires, the app MUST evaluate the captured segment's duration against the minimum clip duration (default approximately 300–500 ms).
- **FR-013**: If the captured segment meets or exceeds the minimum clip duration, the app MUST finalize and save it as a WAV file in the recordings directory.
- **FR-014**: If the captured segment is shorter than the minimum clip duration, the app MUST discard it without saving a file, and MUST display a clear status message in the TUI indicating that the segment was too short and was discarded.
- **FR-015**: Saved WAV files MUST use a timestamp-based filename sortable by newest-first, consistent with the naming convention used by manual recordings.
- **FR-016**: Each sound-activated segment MUST be saved as a separate WAV file; multiple segments from one monitoring session MUST NOT be merged into a single file.
- **FR-017**: WAV files MUST be written first as temporary/partial files that are renamed to their final name only upon successful completion, so that incomplete files do not appear in the recordings list.
- **FR-018**: After a segment is successfully saved, the app MUST refresh the recordings list in the TUI so the new file becomes visible without requiring a manual UI refresh.
- **FR-019**: Silence-only monitoring periods MUST NOT produce any WAV files.
- **FR-020**: The TUI MUST expose a Stop action that ends monitoring from any monitoring sub-state (Monitoring or Capturing).
- **FR-021**: When Stop is activated while the app is in the silent-monitoring state (no active capture), the app MUST transition to idle without writing any file.
- **FR-022**: When Stop is activated while a segment is being actively captured and the captured duration meets the minimum clip duration, the app MUST finalize and save the segment before transitioning to idle.
- **FR-023**: When Stop is activated while a segment is being actively captured but the captured duration is below the minimum clip duration, the app MUST discard the segment, display a clear status message, and transition to idle.
- **FR-024**: The Play action MUST be disabled or rejected while monitoring is active, with a visible status message explaining the conflict.
- **FR-025**: The Monitor action MUST be disabled or rejected while playback is active, with a visible status message explaining the conflict.
- **FR-026**: All audio detection and WAV file I/O MUST be performed on background threads so that the TUI event loop remains responsive during monitoring.
- **FR-027**: Any error that occurs during monitoring (device unavailable, permission denied, stream failure, file write failure, list refresh failure) MUST be displayed as a clear, user-visible message in the TUI and MUST NOT crash the application.
- **FR-028**: After any monitoring error, the app MUST return to idle state so the user can retry or continue using other TUI features.
- **FR-029**: If the monitoring threshold is so low that sound is detected continuously without pausing, the TUI MUST display a warning indicating that the threshold may be set too low.
- **FR-030**: Memory used for audio buffering MUST remain bounded during extended silent-monitoring periods; the app MUST NOT accumulate all incoming audio samples in memory.

### Constitution Alignment *(mandatory)*

- **CA-001 (Rust-First)**: All new audio processing, detection, and file I/O code MUST be implemented in Rust using the existing `cpal` and `hound` crates. No additional language runtimes or external processes are permitted unless explicitly justified in the planning artifacts.
- **CA-002 (TUI-First, not CLI-First)**: All user-facing controls — starting monitoring, stopping monitoring, viewing status, and seeing saved segments — MUST be accessible exclusively through the TUI. No CLI flags or sub-commands may be introduced for this feature.
- **CA-003 (Verification Gate)**: Each user story MUST have an independently testable verification path. Automated tests are preferred for detection logic, file-writing behavior, and state-machine transitions. Manual TUI render verification is acceptable for status label display.
- **CA-004 (Integration Safety)**: The new monitoring state machine MUST be integrated into the existing TUI `AppState` machine. Integration tests MUST cover the transitions Idle→Monitoring→Capturing→Idle (with save) and Idle→Monitoring→Idle (stop during silence) to verify audio engine wiring.
- **CA-005 (Version Discipline)**: Adding sound-activated monitoring alongside the existing manual record flow is additive. If no existing behavior is removed or changed in an incompatible way, this is a MINOR version increment. If any existing keybinding or state transition is changed, a MAJOR bump is required and must be called out in planning.

### Key Entities *(include if feature involves data)*

- **MonitoringSession**: Represents one continuous listening period initiated by the user. Has sub-states: Listening (silent), Capturing, Finalizing/Stopping. A session ends when the user stops monitoring or an unrecoverable error occurs.
- **SoundSegment**: A contiguous block of audio samples captured during a single detected sound event within a session. Has a start timestamp, a rolling pre-roll buffer, accumulated audio data, and a duration. Either saved as a WAV file or discarded depending on minimum duration.
- **PreRollBuffer**: A bounded circular buffer of recent audio samples (approximately 300–500 ms) maintained continuously while monitoring. Prepended to a new SoundSegment when a threshold crossing is detected.
- **WavFileEntry**: An existing entity representing a saved WAV file visible in the recordings list. Sound-activated segments produce WavFileEntry records identical in shape to manually recorded files.
- **DetectionThreshold**: A configurable value (with a documented default) against which incoming audio sample amplitude is compared to classify a sample window as "sound" or "silence."

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Starting monitoring from idle changes the TUI status label to "Monitoring" within 1 second of the user activating the action.
- **SC-002**: A sound event above the detection threshold that lasts at least the minimum clip duration produces exactly one completed WAV file in the recordings directory after the silence timeout expires.
- **SC-003**: Silence-only monitoring for at least 30 seconds produces zero new WAV files.
- **SC-004**: A noise burst shorter than the minimum clip duration produces zero saved WAV files and causes a visible "too short, discarded" status message to appear in the TUI.
- **SC-005**: A sound event with one internal pause shorter than the silence timeout produces exactly one WAV file (not two), with the pause bridged within a single segment.
- **SC-006**: Stopping monitoring while a valid segment is in progress (duration ≥ minimum) saves the segment and returns the app to idle within 2 seconds.
- **SC-007**: Stopping monitoring while no capture is in progress returns the app to idle within 1 second without writing any file.
- **SC-008**: The TUI remains responsive to key input during continuous monitoring; a keypress produces a visible response within 100 ms even while audio capture is active.
- **SC-009**: Memory consumed by audio buffering during a 5-minute silent monitoring period does not grow unboundedly; it remains within the size of the pre-roll buffer plus any fixed per-session overhead.
- **SC-010**: A newly saved sound-activated segment appears in the recordings list in the TUI without requiring any user-initiated refresh action.
- **SC-011**: Attempting to start monitoring while playback is active, or start playback while monitoring is active, results in a visible rejection message within 100 ms.
- **SC-012**: Any monitoring error (device unavailable, stream failure, file write failure) results in a user-visible message in the TUI and the app returning to idle without crashing.

## Assumptions

- The system default audio input device is used for monitoring; no device selector UI is added in this feature iteration.
- The detection threshold, silence timeout, pre-roll duration, and minimum clip duration are either fixed at documented default values or exposed as startup configuration. Runtime adjustment via the TUI is not in scope unless explicitly added in planning.
- Assumed default values: detection threshold — a documented amplitude level to be refined in planning; silence timeout — 1.5 seconds; pre-roll — 400 ms; minimum clip duration — 500 ms.
- WAV files produced by monitoring use the same format (sample rate, bit depth, channel count) as those produced by manual recording.
- Timestamp-based filenames for monitoring-produced files follow the same convention used by the existing recording workflow, ensuring consistent sort order.
- The recordings directory is the same `./recordings/` path used by manual recordings; no separate monitoring output directory is created.
- Monitoring is a standalone action alongside the existing manual record action; the manual Record flow is preserved unchanged.
- The TUI event loop remains single-threaded; all audio detection and file I/O happens on dedicated background threads and communicates results back to the event loop via channels, consistent with the existing audio architecture.
- This feature targets macOS and Linux terminal emulators and uses the existing `cpal` and `hound` dependencies. No new mandatory crates are assumed.
- Speech recognition, semantic classification, and any form of machine learning are out of scope.

## Non-Goals

- Speech-to-text transcription or any form of audio content analysis.
- Semantic sound classification (e.g., distinguishing music from speech from noise).
- Cloud upload, remote sharing, or streaming of recordings.
- Editing, trimming, or merging saved clips after recording.
- Multi-track or multi-source recording.
- Manual audio waveform visualization in the TUI.
- Advanced noise reduction or noise-gate DSP beyond a simple amplitude threshold.
- Per-session audio device selection beyond the system default (unless already planned elsewhere in the project).
- Adjusting detection parameters (threshold, timeouts, durations) at runtime via the TUI in this feature iteration.
