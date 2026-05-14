# Audio Thread Protocol Contract

**Feature**: 002-tui-recorder-ui  
**Date**: 2026-05-14

## Overview

Audio operations (recording and playback) run on dedicated background threads. The TUI event loop remains on the main thread and coordinates with audio threads via shared atomics and channels. This contract defines the protocol for both directions of communication.

## Recording Thread Protocol

### Startup

```
TUI (main thread)                    Recording Thread
─────────────────────────────────────────────────────
create Arc<AtomicBool> stop_flag     ← receives stop_flag clone
create mpsc::channel()               ← receives tx half
spawn thread(stop_flag, tx, out_dir)
store RecordingHandle {
  stop_flag, result_rx, thread
}
transition AppState → Recording
```

### During Capture

The recording thread:
1. Opens the default input device via `cpal`.
2. Captures samples into an in-memory buffer in the audio callback.
3. On each audio callback iteration, checks `stop_flag.load(Ordering::Relaxed)`.
4. When `stop_flag` is `true`, signals the stream to stop.

### Stop and Finalization

```
TUI stop action                      Recording Thread
─────────────────────────────────────────────────────
stop_flag.store(true, Relaxed)  ──►  detects flag
                                     drops cpal stream
                                     writes temp WAV
                                     renames temp → final
                                     tx.send(Ok(final_path))
result_rx.try_recv() → Ok(path)
refresh WAV file list
transition AppState → Idle
```

### Error Path

If an error occurs during capture or WAV write:
- Thread sends `tx.send(Err(AppError::...))`.
- TUI receives error on next tick via `try_recv`.
- Any partial temp file is deleted before sending the error.
- TUI transitions to Idle and sets `status_message` to the error text.

---

## Playback Thread Protocol

### Startup

```
TUI (main thread)                    Playback Thread
─────────────────────────────────────────────────────
create Arc<AtomicBool> stop_flag     ← receives stop_flag clone
create mpsc::channel()               ← receives tx half
spawn thread(stop_flag, tx, path)
store PlaybackHandle {
  stop_flag, result_rx, thread, source_path
}
transition AppState → Playing
```

### During Playback

The playback thread:
1. Opens the default output device via `cpal`.
2. Reads WAV samples into memory using `hound`.
3. Writes output frames in the audio callback.
4. On each frame boundary, checks `stop_flag.load(Ordering::Relaxed)`.
5. When `stop_flag` is `true` or all frames are consumed, exits the playback loop.

### Natural Completion

When all frames are written:
- Thread sends `tx.send(Ok(()))`.
- TUI receives `Ok(())` via `try_recv` on the next tick.
- TUI transitions to Idle.

### TUI-Initiated Stop

```
TUI stop action                      Playback Thread
─────────────────────────────────────────────────────
stop_flag.store(true, Relaxed)  ──►  detects flag
                                     exits playback loop
                                     drops cpal stream
                                     tx.send(Ok(()))
result_rx.try_recv() → Ok(())
transition AppState → Idle
```

### Error Path

- Thread sends `tx.send(Err(AppError::...))`.
- TUI transitions to Idle and sets `status_message` to the error text.

---

## Shared Invariants

1. `stop_flag` is only written by the **main thread** (TUI) and only read by the **audio thread**.
2. The `tx` channel sender is owned by the audio thread; `result_rx` is owned by the TUI.
3. The audio thread **never** panics — all errors are sent via `tx` as `Err` variants.
4. After a stop or completion, the `thread` `JoinHandle` is joined by the TUI on the next event-loop tick to ensure clean thread shutdown.
5. The cpal `Stream` is always dropped inside the audio thread (it is `!Send`).