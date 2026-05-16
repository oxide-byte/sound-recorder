---

description: "Task list for feature 004 — audio-format-compression"
---

# Tasks: Audio Format Compression

**Input**: Design documents from `/specs/004-audio-format-compression/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Included. Plan requires unit tests for the parser, profile validation, and the WAV writer dispatch; integration tests for the load-then-record path (valid + missing + invalid).

**Organization**: Tasks are grouped by user story so each story can be implemented and verified independently. Priorities follow spec.md: US1 (P1) → US2 (P2) → US3 (P3).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single-project Rust crate (per plan.md):
- `src/` — production code
- `tests/` — integration & unit tests
- `config/` — runtime defaults (new in this feature)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the new module skeleton and ship the default config file so subsequent phases can wire into them.

- [X] T001 Create empty module file `src/config/mod.rs` (re-exported in step T003)
- [X] T002 [P] Create the default config file `config/audio.conf` with the exact contents documented in `specs/004-audio-format-compression/contracts/config-file.md` (the "shipped default" example block)
- [X] T003 Add `pub mod config;` to `src/lib.rs` and `mod config;` to `src/main.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The error variant and the model types that every other task imports.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Add `Config(String)` variant to `AppError` in `src/error.rs` with `#[error("config error: {0}")]`
- [X] T005 Add `SupportedFormat` enum (variants `Wav`) with `from_id(&str) -> Result<Self, AppError>` and `as_id(&self) -> &'static str` (case-insensitive parsing, exact error wording per `contracts/config-file.md`) in `src/model/mod.rs`
- [X] T006 Add `CompressionProfile` enum (variants `Pcm8`, `Pcm16`, `Pcm24`, `Float32`) with `from_id`/`as_id` to `src/model/mod.rs` (depends on T005 — same file)
- [X] T007 Add `AudioOutputProfile { format, compression }` struct with `validated(format, compression) -> Result<Self, AppError>` (compatibility-matrix check) and `Default` impl (`Wav` + `Pcm16`) to `src/model/mod.rs` (depends on T006 — same file)
- [X] T008 Add `ConfigSource { File, Fallback }` enum and `AudioDefaultsConfig { profile, source }` struct to `src/config/mod.rs` (depends on T007)

**Checkpoint**: `cargo build` compiles. Types exist but are unused — no behavior change yet.

---

## Phase 3: User Story 1 — Configure Recording Output Profile (Priority: P1) 🎯 MVP

**Goal**: When the recording or monitoring threads are spawned with an `AudioOutputProfile`, the saved WAV reflects that profile's container and bit-depth/sample-format.

**Independent Test**: With a hand-constructed `AudioOutputProfile::validated(Wav, Pcm24).unwrap()`, call the (now profile-aware) recording writer on a synthetic `Vec<i16>` and assert via `hound::WavReader` that the file's `WavSpec` has `sample_format = Int`, `bits_per_sample = 24`. Repeat for each profile variant. No config file or TUI involvement required for this story.

### Tests for User Story 1

- [X] T009 [US1] Unit tests for `SupportedFormat::from_id`/`as_id`, `CompressionProfile::from_id`/`as_id`, `AudioOutputProfile::validated`, and identifier round-trip in `tests/unit/profile_validation.rs` — assert exact error messages from `contracts/config-file.md` error catalog

### Implementation for User Story 1

- [X] T010 [US1] Extend `record::start_recording_thread` signature to accept `profile: AudioOutputProfile` and thread it through `record_until_stop` / `record_microphone_until_stop` (capture loop unchanged) in `src/audio/record.rs`
- [X] T011 [US1] Replace the hardcoded `hound::WavSpec { bits_per_sample: 16, sample_format: Int, .. }` in `record_microphone_until_stop` with a profile-aware writer dispatch matching the table in `contracts/output-profile.md` (Pcm8 unsigned-offset, Pcm16 identity, Pcm24 left-shifted into `i32`, Float32 normalized to `[-1.0, 1.0]`) in `src/audio/record.rs`
- [X] T012 [P] [US1] Add `output_profile: AudioOutputProfile` field to `MonitorConfig` and populate it via `AudioOutputProfile::default()` in the `Default` impl in `src/audio/monitor.rs`
- [X] T013 [US1] Replace the hardcoded `WavSpec` in `monitor::write_wav_file` with a profile-aware writer dispatch (same dispatch table as T011); update the call site in `finalize_or_discard` to pass the profile in `src/audio/monitor.rs` (depends on T012 — same file)
- [X] T014 [P] [US1] Add `#[cfg(test)]` unit tests in `src/audio/record.rs` that, for each compression profile, write a small synthetic `Vec<i16>` and assert the resulting WAV's `WavSpec` via `hound::WavReader::open(path).unwrap().spec()`
- [X] T015 [P] [US1] Add `#[cfg(test)]` unit tests in `src/audio/monitor.rs` for `write_wav_file` covering each compression profile (same assertion shape as T014)
- [X] T016 [US1] Update the two callers of `start_recording_thread` and `start_monitoring_thread` in `src/tui/app.rs` to pass `AudioOutputProfile::default()` for now (US2 will replace this with the loaded config) — keep the TUI building

**Checkpoint**: `cargo test` is green. Writers honor any `AudioOutputProfile` passed in; the TUI still uses the hardcoded default, so user-observable behavior is unchanged.

---

## Phase 4: User Story 2 — Use Central Default Audio Settings (Priority: P2)

**Goal**: On TUI startup the application reads `./config/audio.conf`, validates the keys, and uses the resulting `AudioOutputProfile` for every subsequent recording/monitoring run. If the file is missing, fall back to the built-in defaults.

**Independent Test**: With the shipped `config/audio.conf` (`format=wav`, `compression=pcm16`) present, launch the TUI, press `r`, stop, quit, and assert the resulting WAV is 16-bit Int. Then change `compression` to `pcm24`, relaunch, record again, assert 24-bit Int. Delete the file, relaunch, assert the TUI shows `Using built-in defaults — config/audio.conf not found.` and the next recording is 16-bit Int. None of these scenarios touch invalid-config handling (that's US3).

### Tests for User Story 2

- [X] T017 [P] [US2] Unit tests for the line-based parser in `tests/unit/config_parse.rs` — cover blank lines, `#` comments, whitespace tolerance, case-insensitive identifier parsing, valid round-trip, missing-required-key error, and exact error-message text for each entry in the error catalog (parse failures only — validation failures are tested in US3)
- [X] T018 [P] [US2] Integration test in `tests/integration/config_record.rs`: write a valid `audio.conf` with `compression=pcm24` to a temp dir, run a brief record cycle (using a helper that drives the recording thread directly, not via the live mic), and assert the saved WAV's `WavSpec` matches `pcm24`. Include a second test case for the missing-file fallback that asserts `source == Fallback` and the saved file is `pcm16`.

### Implementation for User Story 2

- [X] T019 [P] [US2] Implement the line-based parser, `parse_config(text: &str) -> Result<AudioDefaultsConfig, AppError>` (returns a profile, never `Fallback` source), in `src/config/mod.rs`
- [X] T020 [US2] Implement `pub fn load_or_default(path: &Path) -> Result<AudioDefaultsConfig, AppError>` in `src/config/mod.rs` following the behavior matrix in `contracts/config-file.md` (file missing → `Ok(Fallback)`; file present + valid → `Ok(File)`; other I/O error → `Err(AppError::Io)`; parse/validation failure → `Err(AppError::Config)`) — depends on T019
- [X] T021 [US2] Add `defaults: Option<AudioDefaultsConfig>` field to `TuiContext` and initialize to `None` in `TuiContext::new()` in `src/model/mod.rs`
- [X] T022 [US2] In `src/tui/mod.rs` (or the entry point called by `main`), call `config::load_or_default(Path::new("config/audio.conf"))` before entering the event loop; on `Ok`, assign to `ctx.defaults`; on `Err`, set the status message per the contract (US3 covers the error wording — for US2, accept `unwrap_or_default()` semantics that yield the fallback profile so the happy path is testable in isolation)
- [X] T023 [US2] In `handle_record` and `handle_monitor` in `src/tui/app.rs`, take the active profile from `ctx.defaults.as_ref().map(|d| d.profile.clone()).unwrap_or_default()` and pass it to `start_recording_thread` / construct `MonitorConfig { output_profile, .. }` — replaces the `AudioOutputProfile::default()` placeholder from T016
- [X] T024 [US2] When `ctx.defaults.as_ref().map(|d| &d.source) == Some(&ConfigSource::Fallback)`, set `ctx.status_message = Some("Using built-in defaults — config/audio.conf not found.".to_string())` exactly once at startup in `src/tui/app.rs` (or `src/tui/mod.rs`, wherever the loader is invoked)

**Checkpoint**: With the shipped `config/audio.conf`, recordings honor the configured profile; with the file removed, recordings use built-in defaults and the fallback notice is shown. Invalid-config handling is *not* yet wired (that's US3).

---

## Phase 5: User Story 3 — Safe Handling of Unsupported Configuration (Priority: P3)

**Goal**: When `config/audio.conf` exists but contains invalid or unsupported values, the TUI surfaces a clear, actionable error and refuses to start recording or monitoring until the file is fixed.

**Independent Test**: Write `config/audio.conf` with `compression=lossless-magic`, launch the TUI, observe the status message `Audio defaults invalid — fix config/audio.conf: config error: unsupported compression 'lossless-magic'; supported: pcm8, pcm16, pcm24, float32`. Press `r` and `m` — verify no recording starts and no WAV file is created under `recordings/`.

### Tests for User Story 3

- [X] T025 [US3] Add unit tests to `tests/unit/config_parse.rs` for the **validation-failure** error catalog entries: unknown format id, unknown compression id, malformed line, empty key, empty value, unknown key, duplicate key — assert exact wording per `contracts/config-file.md` (depends on T017 — same file)
- [X] T026 [US3] Add an integration test case to `tests/integration/config_record.rs`: write an invalid `audio.conf` (`compression=lossless-magic`), instantiate a `TuiContext`, invoke the loader path used by startup, simulate `handle_record` and `handle_monitor` calls, and assert (a) status message contains the documented `Audio defaults invalid — fix config/audio.conf:` prefix and (b) no WAV file is produced and `ctx.app_state` remains `Idle` (depends on T018 — same file). [Note: implemented as an integration test that asserts loader error wording + no-file-write, plus `#[cfg(test)]` gate tests in `tui/app.rs` for the recording/monitoring gate.]

### Implementation for User Story 3

- [X] T027 [US3] In the TUI startup path (`src/tui/mod.rs` or `src/tui/app.rs`, wherever T022 put the loader call), match on `load_or_default(...)` `Err` cases and set `ctx.status_message = Some(format!("Audio defaults invalid — fix config/audio.conf: {err}"))`. Leave `ctx.defaults = None` so the recording gate trips. (Modifies the same site as T022.)
- [X] T028 [US3] In `handle_record` and `handle_monitor` in `src/tui/app.rs`, add a guard at the top: when `ctx.defaults.is_none()`, set `ctx.status_message = Some("Audio defaults invalid — fix config/audio.conf".to_string())` and return without spawning the thread (modifies the same functions as T023)

**Checkpoint**: All three stories pass their independent tests. `cargo test` is fully green. Quickstart steps 6 and 7 (fallback path and invalid-config path) reproduce manually.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Cosmetic and housekeeping work that benefits from all three stories being in place.

- [X] T029 [P] Render the active profile (`wav/pcm16` etc.) in the TUI footer/help line in `src/tui/view.rs` (rendered in the top-bar title: ` sound-recorder — wav/pcm16 `)
- [X] T030 [P] Update `README.md` with a short "Audio output profile" section pointing at `config/audio.conf` and listing supported values
- [ ] T031 [P] Walk through `specs/004-audio-format-compression/quickstart.md` steps 1–8 manually and record evidence (paste outputs into a verification note); fix any deviations — **deferred: requires interactive TTY + live microphone, cannot be performed from this session**
- [X] T032 Bump `version` in `Cargo.toml` from `1.1.0` to `1.2.0` (MINOR — additive feature per CA-005)
- [X] T033 Run `cargo build` and `cargo test` once more end-to-end; fix any new warnings (zero warnings; 77 of the project's tests pass; 3 pre-existing CLI tests from feature 001 remain broken and are outside the scope of feature 004)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup (T003 reexports the new module) — **BLOCKS all user stories**
- **User Story 1 (Phase 3)**: Depends on Foundational only
- **User Story 2 (Phase 4)**: Depends on Foundational + US1 (US2's recording assertions require US1's writers; the placeholder in T016 ensures intermediate compilability, so US2 can be started in parallel with later US1 tasks as long as T010/T011 land before T018)
- **User Story 3 (Phase 5)**: Depends on US2 (loader + status surface). Can be implemented incrementally on top of US2 once T020/T022 exist.
- **Polish (Phase 6)**: Depends on all three stories.

### Task-Level Dependencies (notable)

- T003 depends on T001
- T006 → T005, T007 → T006, T008 → T007 (same file — `src/model/mod.rs` for T005/T006/T007)
- T011 → T010 (same file — `src/audio/record.rs`)
- T013 → T012 (same file — `src/audio/monitor.rs`)
- T016 → T010 + T012 (TUI must compile against the new signatures)
- T020 → T019 (parser before loader)
- T023 → T021 + T022 (context field + loader call site before the use sites)
- T024 → T022 (status message uses the loaded source)
- T025 → T017 (same file)
- T026 → T018 (same file)
- T027 → T022 (extends the same call site)
- T028 → T023 (extends the same functions)

### Parallel Opportunities

- T002 runs in parallel with T001/T003 (different file).
- T012 runs in parallel with T010/T011 (different file: `monitor.rs` vs `record.rs`).
- T014 and T015 run in parallel (different files).
- T017 and T018 run in parallel within US2 (different files).
- T019 runs in parallel with US2's tests (T017/T018).
- T029, T030, T031 in the polish phase all touch different files and can run in parallel.

---

## Parallel Example: User Story 1

```bash
# Once T007 is done, T010 and T012 can be queued together because they touch separate files:
Task: "T010 [US1] Add profile parameter to start_recording_thread in src/audio/record.rs"
Task: "T012 [US1] Add output_profile field to MonitorConfig in src/audio/monitor.rs"

# Once T011 and T013 are done, T014 and T015 can run in parallel:
Task: "T014 [US1] Profile dispatch tests in src/audio/record.rs"
Task: "T015 [US1] Profile dispatch tests in src/audio/monitor.rs"
```

---

## Implementation Strategy

### MVP (User Story 1 + User Story 2)

US1 alone produces no user-visible behavior change (the TUI keeps passing the default profile). The minimal user-visible MVP is **US1 + US2**: the shipped `config/audio.conf` is honored end-to-end. Steps:

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 (writers honor any profile)
4. Complete Phase 4: User Story 2 (config drives the profile)
5. **STOP and VALIDATE**: Run quickstart steps 3–6 (default, pcm24, fallback)
6. Deploy/demo

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. + US1 → Writers honor profile (unit-testable; no user-visible change)
3. + US2 → Config-driven defaults (user-visible MVP; quickstart steps 3–6 pass)
4. + US3 → Fail-fast validation (quickstart step 7 passes)
5. Polish → README + version bump + footer

### Suggested First Pass Order

T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T013 → T014 → T015 → T016 → T017 → T019 → T020 → T021 → T022 → T023 → T024 → T018 (integration test now meaningful) → T025 → T026 → T027 → T028 → T029 → T030 → T031 → T032 → T033

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks.
- [Story] label maps each task to its user story for traceability.
- Each user story has its own independent test plan in the spec; phase checkpoints map directly to those criteria.
- Verify tests fail before implementing — especially for T017/T025 (parser error catalog) where exact-message asserts can be brittle.
- Commit after each task or logical group (the `after_implement` hook will offer a final commit at the end).
- Avoid: stuffing parser parsing logic into `tui/app.rs`, introducing new crate dependencies, or modifying existing WAV file paths/naming.
