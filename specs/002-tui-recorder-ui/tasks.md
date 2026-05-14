# Tasks: TUI Recorder Workflow

**Input**: Design documents from `specs/002-tui-recorder-ui/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Integration tests included for audio engine wiring (required by Constitution Check CA-003).  
Manual TUI render verification acceptable for visual state indicators.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no shared dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths are included in all descriptions

---

## Phase 1: Setup — Remove CLI Infrastructure

**Purpose**: Delete the CLI layer that is being replaced by the TUI. Establishes a clean compile baseline before new code is written.

- [X] T001 Remove `clap` from `Cargo.toml` dependency list
- [X] T002 [P] Delete `src/cli/mod.rs` and `src/cli/commands.rs` (entire `src/cli/` directory)
- [X] T003 [P] Rewrite `src/model/mod.rs` — replace `AudioDevice`, `RecordingSession`, `WavAsset`, `PlaybackRequest` with `WavFileEntry`, `RecordingHandle`, `PlaybackHandle`, `AppState`, `TuiContext` per `specs/002-tui-recorder-ui/data-model.md`

**Checkpoint**: `cargo check` passes with no CLI references. ✅

---

## Phase 2: Foundational — TUI Shell (also satisfies US3)

**Purpose**: Establish the working TUI skeleton that `main.rs` launches directly. All user story work depends on this shell existing. This phase also fully satisfies User Story 3 (Launch Directly into TUI).

**⚠️ CRITICAL**: No user story feature work can begin until this phase compiles and runs.

- [X] T004 [P] Rewrite `src/lib.rs` — remove `mod cli` declaration; retain `pub mod audio`, `pub mod model`, `pub mod tui`, `pub mod error`
- [X] T005 [P] Rewrite `src/main.rs` — remove all `clap` and `cli` imports; call `tui::run_tui()` directly; on `Err` print to stderr and exit with code 1
- [X] T006 Rewrite `src/tui/mod.rs` — implement `run_tui()`: enter crossterm alternate screen, enable raw mode, init `ratatui::Terminal`, call `app::run_event_loop`, disable raw mode and leave alternate screen on return (both on `Ok` and `Err`)
- [X] T007 Implement `src/tui/app.rs` — `run_event_loop(terminal)`: poll crossterm events with 100 ms timeout; `TuiContext` starts as `AppState::Idle`, empty `wav_files`, no selection; handle `'q'` and `Esc` to return when Idle; call `view::render` each tick
- [X] T008 [P] Implement `src/tui/view.rs` — `render(frame, ctx)`: three labeled button areas (`Record`, `Play`, `Stop`) at top; scrollable `WavFileEntry` list in center; status bar at bottom showing `ctx.status_message` or default idle hint text; visually distinguish the selected list item

**Checkpoint**: `cargo run` launches TUI directly, renders layout with empty file list, exits cleanly on `q`. US3 is fully satisfied. ✅

---

## Phase 3: User Story 1 — Record and Save from TUI (Priority: P1) 🎯 MVP

**Goal**: User activates Record, speaks, activates Stop, and a new WAV file with an auto-generated timestamp name appears in the file list.

**Independent Test**: `cargo run`, press `r`, wait 3+ seconds, press `s`. Confirm a new `recording_*.wav` exists in `./recordings/` and is visible in the TUI file list.

- [X] T009 [P] [US1] Add `ensure_recordings_dir(dir: &Path) -> Result<(), AppError>` and `generate_wav_filename() -> String` (format: `recording_YYYYMMDD_HHMMSS_mmm.wav` using `std::time::SystemTime`) to `src/audio/record.rs`
- [X] T010 [US1] Refactor `src/audio/record.rs` — replace `record_to_wav` sleep-based blocking with `start_recording_thread(stop_flag: Arc<AtomicBool>, tx: Sender<Result<PathBuf, AppError>>, recordings_dir: PathBuf) -> JoinHandle<()>`: spawns background thread that opens default input device, captures samples, checks `stop_flag` in the audio data callback, and on flag set writes temp WAV then renames to final path and sends `Ok(path)` via `tx`; on any error removes partial temp file and sends `Err` via `tx`
- [X] T011 [US1] Implement `scan_wav_files(dir: &Path) -> Vec<WavFileEntry>` in `src/tui/app.rs` — reads the recordings directory, filters entries ending in `.wav`, returns `Vec<WavFileEntry>` sorted by name descending (newest first); called on startup and after each recording completes
- [X] T012 [US1] Add `'r'` key handling in `src/tui/app.rs` `run_event_loop` — when `AppState::Idle`: call `ensure_recordings_dir`, generate output path, create `Arc<AtomicBool>` stop flag and `mpsc::channel`, spawn thread via `start_recording_thread`, store `RecordingHandle` in `AppState::Recording`; set status to `"Recording… press 's' to stop"`
- [X] T013 [US1] Add `'s'` key handling (Recording branch) in `src/tui/app.rs` — when `AppState::Recording`: set `stop_flag` to `true`; on next tick `try_recv` on `result_rx`: on `Ok(path)` refresh `wav_files` via `scan_wav_files`, set status to saved filename, join thread, transition to `AppState::Idle`; on `Err(e)` set status to error message, join thread, transition to Idle
- [X] T014 [P] [US1] Update `src/tui/view.rs` `render` — highlight `Record` button and show recording state indicator when `AppState::Recording`; show refreshed `wav_files` list after stop

**Checkpoint**: Full record→stop→file-in-list flow works end-to-end. US1 independently satisfied. ✅

---

## Phase 4: User Story 2 — Play Stored WAV Files from TUI (Priority: P2)

**Goal**: User selects a WAV file from the list, activates Play, hears audio, can stop early with Stop or wait for natural completion.

**Independent Test**: Ensure at least one WAV exists in `./recordings/`, `cargo run`, highlight a file with `↓`, press `p`, confirm audio plays, press `s` to stop early.

- [X] T015 [US2] Refactor `src/audio/playback.rs` — replace blocking polling loop with `start_playback_thread(stop_flag: Arc<AtomicBool>, tx: Sender<Result<(), AppError>>, wav_path: PathBuf) -> JoinHandle<()>`: spawns background thread that reads WAV samples into memory, builds cpal output stream, checks `stop_flag` on each output frame boundary; on flag set or all frames consumed, drops stream and sends `Ok(())` via `tx`; on error sends `Err` via `tx`
- [X] T016 [US2] Add `'p'` key handling in `src/tui/app.rs` — when `AppState::Idle` and `selected_index` is `Some(i)`: get `path` from `wav_files[i]`, create stop flag + channel, spawn thread via `start_playback_thread`, store `PlaybackHandle` in `AppState::Playing`; set status to `"Playing <filename> — press 's' to stop"`; when Idle with no selection: set status to `"No file selected"` and remain Idle
- [X] T017 [US2] Add `'s'` key handling (Playing branch) in `src/tui/app.rs` — when `AppState::Playing`: set `stop_flag` to `true`; handle result via `try_recv` on next tick: join thread, transition to `AppState::Idle`, clear status
- [X] T018 [US2] Add natural playback completion check in `src/tui/app.rs` event loop tick — each tick call `try_recv` on `Playing` `result_rx`: on `Ok(())` join thread and transition to Idle; on `Err(e)` join thread, set status to error, transition to Idle
- [X] T019 [P] [US2] Update `src/tui/view.rs` — highlight `Play` button and selected file row when `AppState::Playing`; show source filename in status bar; return `Stop`-only hint in status when playing

**Checkpoint**: Full select→play→stop flow and natural playback completion both work. US2 independently satisfied. ✅

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Edge case guards, terminal safety, and version bump required by the plan's Constitution Check.

- [X] T020 [P] Guard invalid key presses in `src/tui/app.rs` — `'r'` and `'p'` while Recording/Playing: no-op; `'s'` while Idle: no-op; `'q'`/`Esc` while Recording/Playing: set status to `"Stop before quitting"` and remain in current state
- [X] T021 [P] Register panic hook in `src/tui/mod.rs` using `std::panic::set_hook` — hook calls `crossterm::terminal::disable_raw_mode()` and `crossterm::execute!(LeaveAlternateScreen)` before the default panic output, ensuring terminal is always restored
- [X] T022 [P] Bump version in `Cargo.toml` from `0.1.0` to `1.0.0` (breaking change: CLI removal per CA-005)
- [X] T023 Manual end-to-end validation per `specs/002-tui-recorder-ui/quickstart.md` — run `cargo build --release`, exercise record→stop→play→stop flow, confirm terminal restores cleanly on quit and after forced kill

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 ⚠️ BLOCKS all user story phases
- **Phase 3 (US1)**: Depends on Phase 2 — can start after foundational TUI shell compiles
- **Phase 4 (US2)**: Depends on Phase 2 — can start in parallel with Phase 3 (different files)
- **Phase 5 (Polish)**: Depends on Phase 3 and Phase 4 completion

### User Story Dependencies

- **US3 (P3 — TUI Launch)**: Satisfied entirely by Phase 2 foundational work
- **US1 (P1 — Record)**: Depends on Phase 2; `audio/record.rs` and `tui/app.rs` recording path
- **US2 (P2 — Play)**: Depends on Phase 2; `audio/playback.rs` and `tui/app.rs` playback path; does NOT depend on US1 (playback engine is independent)

### Within Each Phase

- Tasks marked `[P]` target different files and have no shared incomplete dependencies — safe to run in parallel
- T010 (recording thread) must complete before T012/T013 (TUI wiring) within US1
- T015 (playback thread) must complete before T016/T017/T018 (TUI wiring) within US2

---

## Parallel Execution Example: Phase 2

```
# These four tasks touch different files and can run in parallel:
T004  src/lib.rs
T005  src/main.rs
T006  src/tui/mod.rs   (depends on T004/T005 compiling first)
T007  src/tui/app.rs   (depends on T006 interface)
T008  src/tui/view.rs  (independent of T007 internals)
```

## Parallel Execution Example: Phase 3 + Phase 4

```
# Once Phase 2 is done, US1 and US2 audio threads can be worked in parallel:
T009/T010  src/audio/record.rs    (US1 audio engine)
T015       src/audio/playback.rs  (US2 audio engine)
```

---

## Implementation Strategy

### MVP First (US1 + US3 — 2 phases)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T008) → US3 done
3. Complete Phase 3: US1 Record (T009–T014)
4. **STOP and VALIDATE**: press `r`, speak, press `s`, confirm new WAV in list
5. Ship MVP

### Incremental Delivery

1. Phase 1 + 2 → `cargo run` shows TUI shell (US3 done)
2. Phase 3 → Record→Save works (US1 done, MVP)
3. Phase 4 → Play works (US2 done)
4. Phase 5 → Polish and version bump

---

## Notes

- `[P]` tasks modify different files with no shared incomplete dependencies
- `[US]` labels map tasks to spec.md user stories for traceability
- Audio threads are `!Send`-aware: `cpal::Stream` must be dropped inside the spawning thread
- The `TuiContext` struct lives entirely in `src/tui/app.rs`; `view.rs` receives it by reference for rendering only
- `clap` is fully removed — do not add it back for any reason in this feature
