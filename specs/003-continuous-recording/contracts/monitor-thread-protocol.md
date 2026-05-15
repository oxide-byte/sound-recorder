# Contract: Monitor Thread Protocol

**Branch**: `003-continuous-recording` | **Date**: 2026-05-15  
**Scope**: Communication contract between the monitoring background thread (`audio/monitor.rs`) and the TUI event loop (`tui/app.rs`).

---

## Overview

The monitoring thread and the TUI event loop communicate via two mechanisms:

| Direction           | Mechanism                           | Semantics                        |
|---------------------|-------------------------------------|----------------------------------|
| Event loop → Thread | `Arc<AtomicBool>` (`stop_flag`)     | Set to `true` to request stop    |
| Thread → Event loop | `mpsc::Sender<MonitorEvent>`        | Push events; event loop polls    |

This is identical in shape to the existing `RecordingHandle` and `PlaybackHandle` protocols.

---

## MonitorEvent Variants

### `SubStateChanged(MonitoringSubState)`

Sent when the monitoring thread transitions between `Listening` and `Capturing`.

- Sent when: threshold crossing detected (→ `Capturing`), or silence timeout expires / discard completes (→ `Listening`).
- Event loop action: update `handle.sub_state`; re-render view.

### `SegmentSaved(PathBuf)`

Sent when a segment has been successfully renamed from `.tmp.wav` to its final `.wav` path.

- Sent when: rename succeeds and the segment duration met the minimum.
- Event loop action: refresh `ctx.wav_files`; update `ctx.status_message` to `"Saved: {name}"`.
- The thread stays alive and transitions back to `Listening`.

### `SegmentDiscarded { reason: String }`

Sent when a segment has been evaluated and discarded without producing a file.

- Sent when: silence timeout expired and segment duration was below `min_clip_duration`.
- Event loop action: update `ctx.status_message` to `"Discarded: {reason}"`.
- The thread stays alive and transitions back to `Listening`.

### `ContinuousTriggering`

Sent when the monitoring thread has been continuously in `Capturing` state without any finalization for more than 30 seconds.

- Sent once per 30-second window (not repeated every frame).
- Event loop action: update `ctx.status_message` to `"Warning: threshold may be too low"`. Monitoring continues.

### `Failed(AppError)`

Sent when an unrecoverable error occurs (device unavailable, stream error, file write failure, etc.).

- Sent immediately before the thread exits.
- Event loop action: join the thread; transition `ctx.app_state` to `AppState::Idle`; display error message.

---

## Stop Protocol

1. Event loop sets `stop_flag.store(true, Ordering::Relaxed)`.
2. Event loop sets `ctx.status_message = Some("Stopping…")`.
3. On the next polling iteration (within 50 ms), the monitoring thread detects `stop_flag` and:
   - If `Listening`: exits immediately; no file written.
   - If `Capturing` and segment ≥ `min_clip_duration`: finalizes and saves the segment, sends `SegmentSaved`, then exits.
   - If `Capturing` and segment < `min_clip_duration`: discards, sends `SegmentDiscarded`, then exits.
4. The thread sends no further events after the stop-triggered finalize/discard.
5. Event loop detects `Failed` or `Disconnected` on the channel (thread exited), joins the thread, transitions to `AppState::Idle`.

**Maximum stop latency**: one polling interval (50 ms) + WAV write time for the current segment (bounded by segment size in memory). Target: ≤2 s for typical segments.

---

## Invariants

- The thread MUST send exactly one terminal event (`SegmentSaved`, `SegmentDiscarded`, or `Failed`) for every in-progress segment before exiting.
- After the stop flag is set, the thread MUST send at most one more `SegmentSaved` or `SegmentDiscarded` event (for the current segment) before exiting.
- `SubStateChanged(Listening)` MUST always follow `SegmentSaved` or `SegmentDiscarded` during normal (non-stop) operation.
- The thread MUST NOT send any events after exiting.

---

## Integration Test Requirements

- **Test 1 (save path)**: Feed synthetic sample data above the threshold for > 500 ms, then silence for > 1 500 ms. Verify `SegmentSaved` event received and a `.wav` file exists at the returned path.
- **Test 2 (discard path)**: Feed synthetic data above threshold for < 500 ms, then silence > 1 500 ms. Verify `SegmentDiscarded` event received and no `.wav` file created.
- **Test 3 (stop during capture — save)**: Feed > 500 ms of above-threshold data, set stop flag before silence timeout. Verify `SegmentSaved` event and file exists.
- **Test 4 (stop during capture — discard)**: Feed < 500 ms of above-threshold data, set stop flag. Verify `SegmentDiscarded` event and no file.
- **Test 5 (stop during silence)**: Set stop flag while in `Listening` sub-state. Verify thread exits, no file, no `SegmentSaved`.
