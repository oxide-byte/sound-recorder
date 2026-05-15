# Tasks: Continuous Sound-Activated Recording

**Input**: Design documents from `specs/003-continuous-recording/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅, quickstart.md ✅

**Tests**: Automated unit and integration tests included per user story. Tests live in `#[cfg(test)]` modules within source files (unit) and `tests/` directory (integration). Manual smoke test in Polish phase.

**Organization**: Tasks grouped by user story for independent implementation and delivery.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no unresolved dependencies)
- **[Story]**: Which user story this task belongs to (US1–US4)
- Exact file paths in all descriptions

---

## Phase 1: Setup

**Purpose**: Version bump before any code changes.

- [x] T001 Bump version from `1.0.0` to `1.1.0` in `Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add new types and module skeleton so the codebase compiles cleanly. No logic implemented here — stubs only.

**⚠️ CRITICAL**: All user story phases depend on this phase completing first.

- [x] T002 Add `MonitoringSubState` enum (`Listening`, `Capturing`) and `MonitorEvent` enum (`SubStateChanged`, `SegmentSaved`, `SegmentDiscarded`, `ContinuousTriggering`, `Failed`) to `src/model/mod.rs`
- [x] T003 Add `MonitoringHandle` struct (`stop_flag: Arc<AtomicBool>`, `event_rx: mpsc::Receiver<MonitorEvent>`, `thread: JoinHandle<()>`, `sub_state: MonitoringSubState`) and `AppState::Monitoring(MonitoringHandle)` variant to `src/model/mod.rs`
- [x] T004 [P] Add exhaustive `AppState::Monitoring(_) => {}` stub arms to all `match ctx.app_state` blocks in `src/tui/app.rs` so the project compiles with the new variant
- [x] T005 [P] Add `is_monitoring` and `is_capturing` booleans derived from `AppState::Monitoring` and expose a stub `[ Monitor ]` button in the button bar in `src/tui/view.rs`
- [x] T006 [P] Create `src/audio/monitor.rs` with `MonitorConfig` struct (fields: `threshold_fraction: f32`, `silence_timeout: Duration`, `pre_roll: Duration`, `min_clip_duration: Duration`) implementing `Default` with values `(0.02, 1500ms, 400ms, 500ms)`, plus a stub `pub fn start_monitoring_thread(…) -> JoinHandle<()>` that immediately exits
- [x] T007 Add `pub mod monitor;` to `src/audio/mod.rs`

**Checkpoint**: `cargo build` passes with no errors. No user-visible behaviour has changed.

---

## Phase 3: User Story 1 — Start Monitoring and Save Sound Segments (Priority: P1) 🎯 MVP

**Goal**: User presses `m`, the TUI shows Monitoring state, a sound above the threshold is detected, the pre-roll is prepended, samples are accumulated, after silence the segment is saved as a WAV and appears in the recordings list.

**Independent Test**: Provide synthetic above-threshold samples via the monitoring thread, wait for the silence timeout to elapse, and assert exactly one WAV file is created in the recordings directory. Also assert that silence-only input produces no files.

### Tests for User Story 1

- [x] T008 [P] [US1] Add `#[cfg(test)]` unit tests for `peak_amplitude()` (above/below threshold) and `PreRollBuffer::push_batch()` (bounded length, correct drain) in `src/audio/monitor.rs`
- [x] T009 [P] [US1] Add `#[cfg(test)]` integration test: feed synthetic above-threshold samples followed by silence > 1500ms → assert `MonitorEvent::SegmentSaved(path)` received and `path.exists()` is `true`, then assert silence-only input produces no `SegmentSaved` event in `src/audio/monitor.rs`

### Implementation for User Story 1

- [x] T010 [P] [US1] Implement `peak_amplitude(samples: &[i16]) -> f32` function (returns `max(|s|) / i16::MAX as f32`) in `src/audio/monitor.rs`
- [x] T011 [P] [US1] Implement `PreRollBuffer` internal struct: `VecDeque<i16>` with fixed capacity `(sample_rate × channels × pre_roll_secs) as usize`; implement `push_batch(&mut self, batch: &[i16])` (push-back + front-drain to cap) and `drain(&mut self) -> Vec<i16>` in `src/audio/monitor.rs`
- [x] T012 [US1] Implement `SoundSegment` internal struct (`samples: Vec<i16>`, `sample_rate: u32`, `channels: u16`, `start_timestamp: String`) and `append_batch(&mut self, batch: &[i16])` method in `src/audio/monitor.rs`
- [x] T013 [US1] Implement cpal stream setup in `start_monitoring_thread`: open default input device, build input stream for I16/U16/F32 formats (converting to `i16`), push batches into `Arc<Mutex<Vec<i16>>>` shared buffer; propagate device/stream errors as `MonitorEvent::Failed` via error callback in `src/audio/monitor.rs`
- [x] T014 [US1] Implement monitoring poll loop in `start_monitoring_thread`: drain shared buffer each iteration (50ms sleep), update `PreRollBuffer`, call `peak_amplitude`, detect threshold crossing → create `SoundSegment`, prepend pre-roll via `drain()`, send `SubStateChanged(Capturing)` in `src/audio/monitor.rs`
- [x] T015 [US1] Implement silence timeout tracking in monitoring poll loop: record `last_above_threshold: Option<Instant>`, update on each above-threshold batch; when `Capturing` and `elapsed >= silence_timeout` → call finalization helper in `src/audio/monitor.rs`
- [x] T016 [US1] Implement `finalize_segment(segment, recordings_dir, tx)` helper: generate timestamp filename via `generate_wav_filename()` (reuse pattern from `record.rs`), write samples to `{name}.tmp.wav` using `hound`, rename to `{name}.wav`; on success send `SegmentSaved(final_path)` + `SubStateChanged(Listening)` in `src/audio/monitor.rs`
- [x] T017 [US1] Implement `handle_monitor()` in `src/tui/app.rs`: guard against non-Idle state, call `audio::record::ensure_recordings_dir`, create `MonitorConfig::default()`, call `audio::monitor::start_monitoring_thread`, set `AppState::Monitoring(MonitoringHandle { … })`, set status message `"Monitoring — press 's' to stop"`
- [x] T018 [US1] Add `KeyCode::Char('m') => handle_monitor(ctx)?` branch to the key-event match in `run_event_loop()` in `src/tui/app.rs`
- [x] T019 [US1] Extend `check_audio_completion()` in `src/tui/app.rs`: add arm for `AppState::Monitoring` that calls `try_recv()` on `event_rx`; dispatch `SubStateChanged` (update `handle.sub_state`), `SegmentSaved` (refresh `ctx.wav_files`, update `selected_index`, show `"Saved: {name}"`); on `Disconnected` join thread and set `AppState::Idle`
- [x] T020 [US1] Update `[ Monitor ]` button style in `src/tui/view.rs`: Yellow+Bold when `is_monitoring`, Green when idle, DarkGray otherwise
- [x] T021 [US1] Add Monitoring/Capturing status label in `src/tui/view.rs`: when `is_monitoring && is_capturing` show `"Capturing — sound detected, recording…  's' to stop"`, when `is_monitoring` show `"Monitoring — listening for sound…  's' to stop"`

**Checkpoint**: Press `m`, speak into the microphone, wait ~2 s of silence, observe segment file in recordings list. User Story 1 is fully functional.

---

## Phase 4: User Story 2 — Stop Monitoring Safely (Priority: P2)

**Goal**: User presses `s` while monitoring in any sub-state; the app returns to idle without losing a valid in-progress segment.

**Independent Test**: Set stop flag during Listening state → thread exits, no file written, Idle within 1s. Set stop flag during Capturing with segment ≥ 500ms → `SegmentSaved` received, Idle.

### Tests for User Story 2

- [ ] T022 [P] [US2] Add `#[cfg(test)]` integration test: start monitoring thread, let it enter Listening, set stop_flag → assert no `SegmentSaved` event and thread joins in `src/audio/monitor.rs`
- [ ] T023 [P] [US2] Add `#[cfg(test)]` integration test: start monitoring thread, feed ≥ 500ms of above-threshold samples, set stop_flag before silence timeout → assert `SegmentSaved` received and WAV file exists in `src/audio/monitor.rs`

### Implementation for User Story 2

- [x] T024 [US2] Implement stop-flag check in the monitoring poll loop in `src/audio/monitor.rs`: at the top of each iteration, if `stop_flag` is set, call `finalize_segment` if a `SoundSegment` is active (delegating the min-clip check, implemented in US3/T029), then break
- [x] T025 [US2] Extend `handle_stop()` in `src/tui/app.rs` to add `AppState::Monitoring(h)` arm: set `h.stop_flag.store(true, Relaxed)`, set status message `"Stopping…"`
- [x] T026 [US2] Extend `check_audio_completion()` in `src/tui/app.rs` to handle `MonitorEvent::Failed(e)`: join the monitoring thread, set `AppState::Idle`, set status message `"Monitor error: {e}"`
- [x] T027 [US2] Verify `q`/`Esc` quit guard in `run_event_loop()` in `src/tui/app.rs` already blocks quit when `AppState::Monitoring` (existing `!matches!(Idle)` guard should cover it; add explicit test or confirm via code review)

**Checkpoint**: Press `m`, start speaking, press `s` mid-capture — verify the segment is saved or discarded with a visible message and the app returns to idle. Press `m`, stay silent, press `s` — verify idle with no file.

---

## Phase 5: User Story 3 — Avoid Noisy or Useless Recordings (Priority: P3)

**Goal**: Short noise bursts below the minimum clip duration are discarded with a visible message. Brief pauses within a sound event do not split it into two files. Continuous triggering produces a warning.

**Independent Test**: Feed < 500ms above-threshold data + silence timeout → assert `SegmentDiscarded` and no WAV file. Feed sound + pause < 1500ms + sound → assert exactly one `SegmentSaved` (not two).

### Tests for User Story 3

- [x] T028 [P] [US3] Add `#[cfg(test)]` unit test: segment with sample count below `min_clip_duration` threshold at finalization → `SegmentDiscarded` event, no WAV file created in `src/audio/monitor.rs`
- [ ] T029 [P] [US3] Add `#[cfg(test)]` integration test: above-threshold samples + pause < 1500ms + above-threshold samples → exactly one `SegmentSaved` event (not two); also test above-threshold < 500ms burst → `SegmentDiscarded` in `src/audio/monitor.rs`

### Implementation for User Story 3

- [x] T030 [US3] Add minimum clip duration check to `finalize_segment()` in `src/audio/monitor.rs`: compute `captured_duration = samples.len() / (sample_rate × channels)` in seconds; if below `min_clip_duration` send `MonitorEvent::SegmentDiscarded { reason: "segment too short ({dur}ms)".into() }` and `SubStateChanged(Listening)` without writing any file
- [x] T031 [US3] Handle `MonitorEvent::SegmentDiscarded` in `check_audio_completion()` in `src/tui/app.rs`: update `handle.sub_state` to `Listening`, set `ctx.status_message` to `"Discarded: {reason}"`
- [x] T032 [US3] Implement `ContinuousTriggering` detection in monitoring poll loop in `src/audio/monitor.rs`: track `capture_start: Option<Instant>`; if `Capturing` and `capture_start.elapsed() > 30s` send `MonitorEvent::ContinuousTriggering` once (reset timer to avoid repeated fires)
- [x] T033 [US3] Handle `MonitorEvent::ContinuousTriggering` in `check_audio_completion()` in `src/tui/app.rs`: set `ctx.status_message` to `"Warning: threshold may be too low — continuous triggering detected"`

**Checkpoint**: Make a brief tap sound — verify "Discarded: segment too short" message appears. Clap and pause briefly before continuing — verify one file, not two.

---

## Phase 6: User Story 4 — Maintain Playback and Recordings List Workflow (Priority: P4)

**Goal**: Sound-activated segments appear in the recordings list and integrate with the existing play flow. Monitoring and playback are mutually exclusive with visible rejection messages.

**Independent Test**: After a monitoring segment is saved, verify it appears in `ctx.wav_files` without manual refresh. Verify pressing `p` while monitoring produces a status message and no state change. Verify pressing `m` while playing produces a status message and no state change.

### Tests for User Story 4

- [x] T034 [P] [US4] Add `#[cfg(test)]` test: after `MonitorEvent::SegmentSaved` is processed by `check_audio_completion`, assert `ctx.wav_files` contains the new file and `ctx.selected_index` is set in `src/tui/app.rs`
- [x] T035 [P] [US4] Add `#[cfg(test)]` unit test: call `handle_monitor()` when `ctx.app_state` is `AppState::Playing(_)` → assert `app_state` is still `Playing`, assert `status_message` contains rejection text in `src/tui/app.rs`

### Implementation for User Story 4

- [x] T036 [US4] Reject `KeyCode::Char('p')` in `handle_play()` when `AppState::Monitoring`: set status message `"Stop monitoring before playback"` and return early in `src/tui/app.rs`
- [x] T037 [US4] Reject `KeyCode::Char('m')` in `handle_monitor()` when `AppState::Playing`: set status message `"Stop playback before monitoring"` and return early in `src/tui/app.rs`
- [x] T038 [US4] Update `play_style` in `src/tui/view.rs`: use `DarkGray` when `is_monitoring` (play is not available while monitoring)
- [x] T039 [US4] Update idle status bar help text in `src/tui/view.rs`: change `"Ready — press 'r' to record"` to `"Ready — press 'r' to record or 'm' to monitor"` and `"↑/↓ select  r record  p play  q quit"` to `"↑/↓ select  r record  m monitor  p play  q quit"`

**Checkpoint**: After a monitoring session, new segments appear in the list. Attempting `p` while monitoring shows rejection message. Attempting `m` while playing shows rejection message. Select a monitoring-produced segment and play it back — it works identically to a manually recorded file.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [x] T040 [P] Ensure `SegmentDiscarded` during stop (stop-flag path) also goes through the min-clip check in `finalize_segment()` and sends the correct event in `src/audio/monitor.rs` (verify T024 and T030 interact correctly; no separate new code if already correct)
- [x] T041 [P] Review all `eprintln!` calls introduced during implementation in `src/audio/monitor.rs`; route any stream error messages through `MonitorEvent::Failed` instead, consistent with FR-027
- [x] T042 Run `cargo test` — assert all tests pass including pre-existing recording and playback tests; fix any regressions
- [ ] T043 Manual smoke test per `specs/003-continuous-recording/quickstart.md`: start monitoring, speak, observe Capturing label, observe Saved + list refresh, select new file, play it back, verify no crash or regression in existing Record/Play flows

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — **BLOCKS** all user stories
- **Phase 3 (US1 P1)**: Depends on Phase 2 — delivers MVP
- **Phase 4 (US2 P2)**: Depends on Phase 2; shares `src/audio/monitor.rs` with US1 — start after US1 `T016` (finalize_segment) is merged
- **Phase 5 (US3 P3)**: Depends on Phase 2; extends T016 (finalize_segment) and Phase 4 stop logic — start after T024 (stop-flag handling) is merged
- **Phase 6 (US4 P4)**: Depends on Phase 2; all US4 tasks touch `app.rs`/`view.rs` independently of US1–US3 audio work — can proceed after Phase 2 is complete
- **Phase 7 (Polish)**: Depends on US1–US4 complete

### User Story Dependencies

- **US1 (P1)**: Starts immediately after Phase 2. No dependencies on US2–US4.
- **US2 (P2)**: Requires `finalize_segment()` from US1 T016. Can proceed in parallel otherwise.
- **US3 (P3)**: Requires `finalize_segment()` from US1 T016 and stop-path from US2 T024. Sequential after US2.
- **US4 (P4)**: Requires `check_audio_completion()` monitoring arm from US1 T019. Can proceed in parallel with US2/US3 for view and rejection tasks.

### Within Each User Story

- Tests (T008/T009, T022/T023, T028/T029, T034/T035) can be written before or alongside implementation
- For US1: data structures (T010, T011, T012) before loop logic (T013, T014, T015) before finalization (T016) before TUI wiring (T017–T021)
- For US2: stop-flag in loop (T024) before TUI stop handler (T025)
- For US3: min-clip check (T030) before TUI dispatch (T031); continuous-trigger (T032) before TUI dispatch (T033)

### Parallel Opportunities

| Phase | Tasks that can run concurrently |
|-------|---------------------------------|
| Phase 2 | T004, T005, T006 (different files; all depend on T003) |
| Phase 3 US1 | T008, T009 (tests + peak_amplitude); T010, T011 (different structs in same file — sequential write, parallel review) |
| Phase 6 US4 | T036, T037 (same file, sequential); T038, T039 (same file, sequential); T034, T035 (independent tests) |
| Phase 7 | T040, T041 in parallel |

---

## Parallel Example: User Story 1

```
# Parallel: data structures + tests
Task T008: unit tests for peak_amplitude + PreRollBuffer
Task T010: implement peak_amplitude()
Task T011: implement PreRollBuffer

# Then sequential: SoundSegment → cpal stream → loop → finalize → TUI wiring
Task T012 → T013 → T014 → T015 → T016 → T017 → T018 → T019 → T020 → T021
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002–T007) — `cargo build` must pass
3. Complete Phase 3: User Story 1 (T008–T021) — core monitoring loop + TUI wiring
4. **STOP and VALIDATE**: Press `m`, make sound, observe saved segment in list
5. Demo / handoff if needed

### Incremental Delivery

1. Phase 1 + Phase 2 → codebase compiles, no behaviour change
2. Phase 3 (US1) → monitoring starts, segments detected and saved, list refreshed
3. Phase 4 (US2) → safe stop from any monitoring sub-state
4. Phase 5 (US3) → noisy/short recordings filtered; continuous-trigger warning
5. Phase 6 (US4) → mutual exclusion with playback; help text updated
6. Phase 7 → all tests green, smoke test complete

---

## Notes

- `[P]` tasks operate on different files or have no shared state — safe to implement in one pass without ordering constraints relative to each other
- Every user story ends with a **Checkpoint** that verifies independent testability
- Temp `.tmp.wav` files must never appear in the recordings list; the `scan_wav_files` filter on `.wav` extension already excludes them
- No new crate dependencies — `Cargo.toml` change is the version bump only
- All existing `Recording` and `Playing` paths must remain untouched; verify with `cargo test` after each phase
