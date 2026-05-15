# Developer Quickstart: Continuous Sound-Activated Recording

**Branch**: `003-continuous-recording` | **Date**: 2026-05-15

---

## Prerequisite Reading

1. `specs/003-continuous-recording/spec.md` — feature requirements and user stories
2. `specs/003-continuous-recording/research.md` — design decisions and rationale
3. `specs/003-continuous-recording/data-model.md` — entities, state machine, mutual exclusion table
4. `specs/003-continuous-recording/contracts/monitor-thread-protocol.md` — thread communication contract
5. `specs/003-continuous-recording/contracts/monitor-keybindings.md` — keybindings and UI text

---

## What to Build

Add sound-activated monitoring to the existing TUI recorder. No new crate dependencies.

### New file

**`src/audio/monitor.rs`** — the entire monitoring loop lives here.

```rust
pub struct MonitorConfig { ... }   // threshold, timeouts, durations
pub enum MonitorEvent { ... }      // events sent to the TUI event loop
pub enum MonitoringSubState { ... }// Listening | Capturing

pub fn start_monitoring_thread(
    stop_flag: Arc<AtomicBool>,
    tx: mpsc::Sender<MonitorEvent>,
    recordings_dir: PathBuf,
    config: MonitorConfig,
) -> JoinHandle<()>
```

Internal loop shape (on the background thread):

```
loop {
    sleep(50ms)
    if stop_flag → finalize_or_discard current segment → send event → break

    // process buffered samples from the Mutex (same pattern as record.rs)
    for each new batch:
        peak = max(|s|) for s in batch
        push batch to pre_roll (capped VecDeque)
        if currently_capturing:
            append batch to segment
            if peak < threshold: check silence_timeout
        else:
            if peak >= threshold:
                start new segment (prepend pre_roll drain)
                send SubStateChanged(Capturing)
    
    if capturing && silence_timeout elapsed:
        finalize_segment()  →  save or discard
        send SubStateChanged(Listening)
    
    if capturing && continuous_trigger_limit exceeded:
        send ContinuousTriggering
}
```

The `Arc<Mutex<Vec<i16>>>` pattern from `record.rs` is reused: the cpal data callback pushes raw samples into the mutex-guarded buffer; the monitoring loop drains it on each poll iteration.

### Changes to existing files

**`src/model/mod.rs`**:
- Add `MonitoringHandle { stop_flag, event_rx, thread, sub_state }`
- Add `MonitorEvent` and `MonitoringSubState` enums
- Add `AppState::Monitoring(MonitoringHandle)` variant

**`src/audio/mod.rs`**:
- Add `pub mod monitor;`

**`src/tui/app.rs`**:
- `check_audio_completion`: add arm to poll `MonitoringHandle.event_rx`, dispatch `MonitorEvent` variants
- `handle_monitor()`: new function, mirrors `handle_record()`
- `handle_stop()`: extend to cover `AppState::Monitoring`
- Event loop key handler: add `KeyCode::Char('m') => handle_monitor(ctx)?`
- Reject `p` while `Monitoring`; reject `m` while `Playing`

**`src/tui/view.rs`**:
- Add `[ Monitor ]` button between Record and Play
- Add `is_monitoring` and `is_capturing` booleans from `AppState`
- Update status bar text (see keybindings contract)

---

## Running the Feature

```bash
cargo run
```

Press `m` to start monitoring. Speak or make a sound. After ~1.5 s of silence the segment is saved. Press `s` to stop monitoring. Saved segments appear in the list and can be played with `p`.

---

## Running Tests

```bash
cargo test
```

Unit tests for detection logic (`audio/monitor.rs`) use synthetic `Vec<i16>` data — no microphone required. Integration tests exercise the full state-machine path through `handle_monitor` / `check_audio_completion`.

---

## Key Invariants to Preserve

1. The TUI event loop (main thread) MUST NOT block waiting for audio I/O.
2. `AppState::Monitoring` is only reachable from `AppState::Idle`.
3. `AppState::Playing` is not reachable while `AppState::Monitoring` (and vice versa).
4. Every temp `.tmp.wav` file either becomes a `.wav` file via rename or is deleted — never left behind.
5. The pre-roll `VecDeque` capacity is fixed; pushing samples during silence MUST NOT grow memory unboundedly.
6. No existing tests may regress: `cargo test` must pass before and after this feature.
