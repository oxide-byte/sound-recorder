# Research: TUI Recorder Workflow

**Feature**: 002-tui-recorder-ui  
**Date**: 2026-05-14  
**Status**: Complete — no NEEDS CLARIFICATION items (confirmed by user: no new technical stack)

## Design Decisions

### 1. Recording Stop Mechanism

**Decision**: Use `Arc<AtomicBool>` as a stop flag shared between the TUI event loop and the recording thread.

**Rationale**: The current `record.rs` implementation blocks for a fixed 10-second duration via `thread::sleep`. TUI-controlled recording requires the Stop action to halt capture at any point. `AtomicBool` is lock-free, zero-cost to check in the audio callback, and compatible with the existing `Arc`-sharing pattern already used for `captured_samples`.

**Alternatives considered**:
- `mpsc::channel` for stop signal: works but adds message latency; `AtomicBool` is simpler for a binary flag.
- `Mutex<bool>`: correct but heavier; `AtomicBool` suffices for a single-bit signal.

---

### 2. Playback Stop Mechanism

**Decision**: Same `Arc<AtomicBool>` pattern for playback.

**Rationale**: The current `play_wav` polls a `PlaybackState.finished` flag in a busy loop. Reusing `AtomicBool` for the stop signal allows the TUI to trigger early termination without changing the fundamental polling approach; the audio callback checks the flag on each frame.

**Alternatives considered**:
- Dropping the stream from outside the thread: `cpal::Stream` is `!Send`; it must be dropped from the thread that built it.

---

### 3. TUI–Audio Thread Communication

**Decision**: One `std::sync::mpsc::Receiver<Result<PathBuf, AppError>>` for recording completion; one `std::sync::mpsc::Receiver<Result<(), AppError>>` for playback completion. The TUI polls these receivers on each event-loop tick using `try_recv`.

**Rationale**: `mpsc` channels are the idiomatic Rust way to pass ownership of results back from a background thread. `try_recv` is non-blocking, preserving event-loop responsiveness.

**Alternatives considered**:
- Shared `Arc<Mutex<Option<Result>>>`: achieves the same goal but requires explicit locking on every tick; channel is cleaner.
- Async runtime (tokio): adds a dependency and complexity; std threads with channels are sufficient for two concurrent audio operations.

---

### 4. WAV File Naming

**Decision**: `recording_YYYYMMDD_HHMMSS_mmm.wav` using `std::time::SystemTime` converted to UTC. Example: `recording_20260514_120000_000.wav`.

**Rationale**: Timestamp-based names are unique within a session, human-readable, and sort lexicographically in chronological order. No external crate is required — `SystemTime::now()` and duration arithmetic produce the needed fields.

**Alternatives considered**:
- UUID: unique but not human-readable or sortable by time.
- Sequential counter: requires reading existing files and finding the next number; race-prone if files are deleted externally.

---

### 5. Recordings Directory

**Decision**: `./recordings/` relative to the process CWD, auto-created with `std::fs::create_dir_all` on TUI startup.

**Rationale**: Simple, discoverable, no configuration required. Consistent with the spec assumption.

**Alternatives considered**:
- OS-appropriate data directory (e.g., `~/.local/share/sound-recorder`): better for installed tools; out of scope for this version per spec Assumptions section.

---

### 6. TUI Event Loop Tick Rate

**Decision**: Poll crossterm events with a 100 ms timeout per iteration.

**Rationale**: 100 ms gives a 10 Hz refresh rate — sufficient for a status indicator and file-list update. It balances CPU usage against UI responsiveness. Completing a recording or playback result will appear within the next tick at most.

**Alternatives considered**:
- 16 ms (60 fps): unnecessary CPU for this use case; no animation needed.
- 500 ms: too sluggish for status feedback after Stop.

---

### 7. clap Removal

**Decision**: Remove `clap` from `Cargo.toml`. `main.rs` calls `tui::run_tui()` directly.

**Rationale**: No CLI argument parsing is needed. Removing `clap` reduces compile time and binary size.

**Alternatives considered**:
- Keep `clap` for a future `--recordings-dir` flag: speculative; remove per constitution principle V (no speculative abstractions).

---

## Constitution Check (Post-Research)

All design decisions use only the existing dependency set. The `AtomicBool` + `mpsc` threading model is the simplest correct approach for two background audio operations. No new risk items introduced.