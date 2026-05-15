# Data Model: Continuous Sound-Activated Recording

**Branch**: `003-continuous-recording` | **Date**: 2026-05-15

---

## Entities

### MonitorConfig

Configuration parameters for a monitoring session. Passed by value to the monitoring thread at startup. Not mutable after the thread starts.

| Field                | Type       | Default     | Description                                                                 |
|----------------------|------------|-------------|-----------------------------------------------------------------------------|
| `threshold_fraction` | `f32`      | `0.02`      | Fraction of `i16::MAX` used as the sound-detection amplitude cutoff (0–1). |
| `silence_timeout`    | `Duration` | 1 500 ms    | How long silence must last after sound before a segment is finalized.       |
| `pre_roll`           | `Duration` | 400 ms      | Length of the rolling pre-roll buffer prepended to each new segment.        |
| `min_clip_duration`  | `Duration` | 500 ms      | Minimum finalized segment duration; segments below this are discarded.      |

Validation rules:
- `threshold_fraction` must be in `(0.0, 1.0]`.
- `silence_timeout` must be > `Duration::ZERO`.
- `pre_roll` must be ≤ `silence_timeout` (pre-roll longer than silence timeout would make every sound event bridge its own timeout).
- `min_clip_duration` must be ≤ `silence_timeout`.

---

### MonitoringHandle

Held inside `AppState::Monitoring`. Owned by the TUI event loop.

| Field        | Type                                    | Description                                                            |
|--------------|-----------------------------------------|------------------------------------------------------------------------|
| `stop_flag`  | `Arc<AtomicBool>`                       | Set to `true` by the event loop to signal the monitoring thread to stop.|
| `event_rx`   | `mpsc::Receiver<MonitorEvent>`          | Polled by the event loop on each frame to receive status updates.       |
| `thread`     | `JoinHandle<()>`                        | The monitoring background thread; joined when stop completes.           |
| `sub_state`  | `MonitoringSubState`                    | Most recently received sub-state; used by the view for status display.  |

`MonitoringHandle` is a new struct analogous to the existing `RecordingHandle` and `PlaybackHandle`.

---

### MonitorEvent

Sent by the monitoring thread to the TUI event loop via the `event_rx` channel.

```
MonitorEvent
├── SubStateChanged(MonitoringSubState)       // Listening ↔ Capturing transition
├── SegmentSaved(PathBuf)                     // Segment finalized and renamed to final path
├── SegmentDiscarded { reason: String }       // Segment discarded (below minimum duration)
├── ContinuousTriggering                      // Threshold likely too low; warning only
└── Failed(AppError)                          // Unrecoverable error; thread is exiting
```

---

### MonitoringSubState

The current activity within the monitoring session, tracked by the event loop from `SubStateChanged` events. Used by the view to render status labels.

```
MonitoringSubState
├── Listening      // Monitoring is active; no sound above threshold detected
└── Capturing      // A sound event is in progress; samples are being accumulated
```

---

### AppState (extended)

The existing `AppState` enum gains one new variant.

```
AppState
├── Idle
├── Recording(RecordingHandle)     // existing
├── Playing(PlaybackHandle)        // existing
└── Monitoring(MonitoringHandle)   // NEW
```

No existing variants are modified. `Monitoring` is mutually exclusive with `Recording` and `Playing` by construction (only reachable from `Idle`).

---

### SoundSegment (internal to `audio/monitor.rs`, not a public type)

Represents one in-progress sound event. Not exposed outside the monitoring thread.

| Field              | Type          | Description                                                                 |
|--------------------|---------------|-----------------------------------------------------------------------------|
| `samples`          | `Vec<i16>`    | Accumulated samples including the prepended pre-roll.                        |
| `start_timestamp`  | `String`      | Timestamp captured when the segment began (used for the final filename).     |
| `sample_rate`      | `u32`         | Inherited from the cpal stream config.                                       |
| `channels`         | `u16`         | Inherited from the cpal stream config.                                       |

`SoundSegment` is created when a threshold crossing is detected and dropped (either after successful write or after discard).

---

### PreRollBuffer (internal to `audio/monitor.rs`, not a public type)

A bounded circular buffer of recent audio samples maintained continuously during monitoring.

| Field      | Type              | Description                                                      |
|------------|-------------------|------------------------------------------------------------------|
| `buf`      | `VecDeque<i16>`   | Rolling window of the most recent `pre_roll` duration of audio. |
| `capacity` | `usize`           | Maximum number of samples = `sample_rate × channels × pre_roll_secs`. |

Operations:
- `push_batch(samples)`: extend back, drain front to maintain capacity.
- `drain() -> Vec<i16>`: return and clear all contents (called on threshold crossing).

---

## State Machine: AppState Transitions

```
                  'm' pressed (Idle)
                  ┌──────────────────────────────────────────┐
                  ▼                                          │
┌──────┐     ┌────────────────────────────────┐             │
│ Idle │────▶│ Monitoring(Listening)          │─────────────┘
└──────┘     │   pre-roll buffer rolling      │  stop ('s') during Listening
  ▲          │   no active segment            │  → Idle (no file written)
  │          └────────────┬───────────────────┘
  │                       │ threshold crossed
  │                       ▼
  │          ┌────────────────────────────────┐
  │          │ Monitoring(Capturing)          │◀─────────────┐
  │          │   segment samples accumulating │              │ sound resumes
  │          └────────────┬───────────────────┘              │ (silence < timeout)
  │                       │                                  │
  │              ┌────────┴────────────────────────────────┐ │
  │              │        silence_timeout elapsed?          │ │
  │              │                                         │ │
  │        Yes  ▼                       No (short pause)   ▼ │
  │   ┌──────────────┐                 continue Capturing ──┘
  │   │ duration ≥   │
  │   │ min_clip?    │
  │   └──────┬───────┘
  │     Yes  │            No
  │          ▼            ▼
  │   ┌──────────────┐  ┌──────────────────────────────┐
  │   │ write temp   │  │ discard; send SegmentDiscarded│
  │   │ rename final │  │ → back to Monitoring(Listening)│
  │   │ SegmentSaved │  └──────────────────────────────┘
  │   │ → Listening  │
  │   └──────────────┘
  │
  │   stop ('s') during Capturing
  │          ▼
  │   ┌──────────────────────────────────┐
  │   │ duration ≥ min_clip?             │
  │   │  Yes: finalize & save            │
  │   │  No: discard + status message    │
  └───┤ → AppState::Idle                 │
      └──────────────────────────────────┘

  Error at any point:
      MonitorEvent::Failed → AppState::Idle + error status message
```

---

## State Machine: Mutual Exclusion

| Current State             | `m` (monitor) | `r` (record) | `p` (play)  | `s` (stop)  |
|---------------------------|---------------|--------------|-------------|-------------|
| Idle                      | ✅ Start       | ✅ Start     | ✅ Start    | No-op       |
| Monitoring (any sub-state)| Ignored       | Ignored      | ❌ Rejected | ✅ Stop     |
| Recording                 | Ignored       | Ignored      | Ignored     | ✅ Stop     |
| Playing                   | ❌ Rejected   | Ignored      | Ignored     | ✅ Stop     |

"Ignored" = silently no-op (existing behavior for `r` and `p` while busy).  
"Rejected" = action blocked with a visible status message.
