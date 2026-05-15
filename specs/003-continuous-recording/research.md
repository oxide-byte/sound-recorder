# Research: Continuous Sound-Activated Recording

**Branch**: `003-continuous-recording` | **Date**: 2026-05-15  
**Purpose**: Resolve technical unknowns before Phase 1 design

---

## R-001: Detection Algorithm

**Decision**: Peak amplitude detection per sample batch.

On each batch of incoming `i16` samples, compute `max(|sample|)`. If this value exceeds `threshold_fraction * i16::MAX` (default fraction: 0.02, i.e., ~655 out of 32 767), classify the batch as "sound"; otherwise classify it as "silence."

**Rationale**:  
- Incremental: runs in O(n) on each cpal callback batch with no accumulation across batches.  
- No ML, no FFT, no additional crates.  
- Unit-testable with synthetic `Vec<i16>` data without a real audio device.  
- Sufficient for general-purpose trigger detection (speech, claps, music).

**Alternatives considered**:  
- *RMS energy*: Slightly more perceptually natural (sustained low-amplitude tone would not trigger peak but would trigger RMS). Adds a division per batch. Rejected because the added perceptual accuracy is not required by the spec, and peak detection is simpler.  
- *Zero-crossing rate*: Suitable for speech detection but ignores amplitude entirely. Rejected as insufficient for general sound detection.

**Default threshold value**: `0.02` (2 % of full scale). This value is documented in `MonitorConfig` as the default and can be changed at the call site. Refinement is expected during planning or by the user at compile time.

---

## R-002: Pre-Roll Buffer

**Decision**: `VecDeque<i16>` with a capped capacity of `(sample_rate × channels × pre_roll_secs)` samples.

Before each monitoring session begins, allocate the deque and set its maximum length. On every cpal callback, push the new batch onto the back of the deque and drain from the front if the deque exceeds the cap. This maintains a rolling window of the most recent `pre_roll_secs` of audio at all times.

When a threshold crossing is detected (transition from silence to sound), drain the deque entirely and prepend its contents to the new `SoundSegment`'s sample buffer.

**Rationale**:  
- `VecDeque` supports O(1) front-drain and O(1) back-push amortized.  
- Capacity is fixed: memory usage during silence is bounded to `2 × sample_rate × channels × pre_roll_secs` bytes (≈ 2 × 44 100 × 1 × 0.4 ≈ 35 KB for mono 44.1 kHz — negligible).  
- No extra thread or timer required.

**Default pre-roll duration**: 400 ms (`MonitorConfig::pre_roll = Duration::from_millis(400)`).

---

## R-003: Silence Timeout Tracking

**Decision**: Record an `Option<Instant>` (`last_above_threshold`) updated inside the cpal callback. Check elapsed time in the monitoring loop's poll iteration.

On each batch: if peak amplitude ≥ threshold, update `last_above_threshold = Some(Instant::now())`. In the polling loop (`thread::sleep(50ms)` cadence), check: if currently capturing and `last_above_threshold.elapsed() >= silence_timeout`, finalize the current segment.

**Rationale**:  
- No extra timer thread.  
- `Instant::elapsed()` is safe to call from any thread.  
- Accuracy is within one polling interval (50 ms), which is well within the 1.5 s default silence timeout.

**Alternatives considered**:  
- *Separate timer thread sending a signal*: Adds concurrency complexity. Rejected because polling is sufficient and consistent with the existing record.rs polling pattern.  
- *Counting silence samples instead of wall time*: Would be inaccurate if the cpal callback fires at variable rates. Rejected.

**Default silence timeout**: 1 500 ms (`MonitorConfig::silence_timeout = Duration::from_millis(1500)`).

---

## R-004: Minimum Clip Duration Enforcement

**Decision**: At finalization time, compute `captured_samples / (sample_rate × channels)` in seconds. If less than `min_clip_secs`, delete the temp file (if started) and send a `MonitorEvent::SegmentDiscarded { reason }` event.

**Rationale**:  
- No samples need to be inspected; only count is needed.  
- The check happens synchronously at finalization, so no extra state is required during capture.

**Default minimum clip duration**: 500 ms (`MonitorConfig::min_clip_duration = Duration::from_millis(500)`).

---

## R-005: Temporary File Safety

**Decision**: Consistent with existing `record.rs` pattern.

1. At segment start: do not open a file yet; buffer samples in `Vec<i16>`.  
2. At finalization: generate a timestamp filename; write to `{name}.tmp.wav`; rename to `{name}.wav` on success; delete `.tmp.wav` on failure or discard.

**Rationale**:  
- Temp files are never visible in the recordings list (the list only shows `.wav` files, not `.tmp.wav`).  
- Atomic rename ensures the final file appears complete.  
- Matches the existing `record.rs` approach, so no new patterns are introduced.

**On monitoring stop during capture**: If the captured duration meets minimum, finalize as above. If below minimum, delete temp (no temp exists yet in the buffering approach — the temp is only created at write time) and discard.

---

## R-006: Channel Protocol — Background Thread to Event Loop

**Decision**: `mpsc::Sender<MonitorEvent>` passed to the monitoring thread; polled via `try_recv()` in the TUI event loop on every frame.

```
MonitorEvent:
  SubStateChanged(MonitoringSubState)   // Monitoring ↔ Capturing transitions
  SegmentSaved(PathBuf)                 // A segment was finalized and saved
  SegmentDiscarded { reason: String }   // A segment was discarded (too short)
  ContinuousTriggering                  // Threshold likely set too low
  Failed(AppError)                      // Unrecoverable error; thread is exiting
```

`MonitoringSubState` carries the values `Listening` (waiting for sound) and `Capturing` (actively accumulating a sound event).

**Rationale**:  
- Single channel, single consumer — matches the existing recording and playback pattern exactly.  
- No `Arc<Mutex<>>` needed for sub-state display; the event loop receives updates through the same channel used for results.  
- The event loop already polls `try_recv()` on every frame for recording and playback results; extending this to monitoring is a direct pattern reuse.

---

## R-007: Continuous Triggering Detection

**Decision**: Track wall-clock time since the last silence window (a period where `last_above_threshold` was None or elapsed beyond `silence_timeout / 2`). If the monitoring thread has been continuously in "Capturing" state for more than 30 seconds without any finalization, send `MonitorEvent::ContinuousTriggering`.

**Rationale**:  
- Simple wall-clock comparison, no counter state.  
- 30-second threshold is long enough to not false-trigger on normal long sounds (speech, music) but short enough to detect constant triggering in a noisy environment promptly.  
- The TUI displays a warning status message on receipt of this event without interrupting monitoring.

---

## R-008: Audio Device Error Handling

**Decision**: The cpal stream error callback, currently `|err| eprintln!(...)`, is changed in `monitor.rs` to send a `MonitorEvent::Failed(AppError::Audio(...))` via a cloned `tx` and then signal the stop flag so the monitoring loop exits.

**Rationale**:  
- Device disconnect is surfaced as a cpal stream error. Making the error callback send on the channel is the cleanest way to propagate it to the TUI event loop.  
- The monitoring loop exits on receipt of the stop signal, allowing the thread to join cleanly.  
- Consistent with the error propagation used in `record.rs` (errors are sent on the result channel, not panicked).

---

## R-009: Version Bump

**Decision**: v1.0.0 → v1.1.0 (MINOR).

**Rationale**:  
- New `m` keybinding and `AppState::Monitoring` variant are additive.  
- No existing keybinding, state, or file format is changed incompatibly.  
- MINOR is correct per semver for backward-compatible new functionality.

---

## Summary of Defaults

| Parameter            | Default  | Constant in `MonitorConfig`         |
|----------------------|----------|-------------------------------------|
| Threshold fraction   | 0.02     | `threshold_fraction: f32`           |
| Silence timeout      | 1 500 ms | `silence_timeout: Duration`         |
| Pre-roll duration    | 400 ms   | `pre_roll: Duration`                |
| Min clip duration    | 500 ms   | `min_clip_duration: Duration`       |
| Continuous trigger   | 30 s     | (internal constant in monitor.rs)   |
