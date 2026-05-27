# Data Model: TUI Recorder Workflow

**Feature**: 002-tui-recorder-ui  
**Date**: 2026-05-14

## Entities

### AppState (enum)

The top-level state machine for the TUI application. Exactly one variant is active at any time.

| Variant | Fields | Description |
|---------|--------|-------------|
| `Idle` | — | No audio operation running; controls available |
| `Recording(RecordingHandle)` | see below | Audio capture active on background thread |
| `Playing(PlaybackHandle)` | see below | Audio playback active on background thread |

**State Transitions**:

```
Idle ──[press Record]──► Recording
Recording ──[press Stop]──► Idle  (WAV file written, list refreshed)
Recording ──[thread error]──► Idle  (error message shown, no file written)
Idle ──[press Play + selection]──► Playing
Playing ──[press Stop]──► Idle
Playing ──[playback complete]──► Idle
Playing ──[thread error]──► Idle  (error message shown)
```

**Invariant**: Record and Play are mutually exclusive. The Stop control is only active when state is `Recording` or `Playing`.

---

### RecordingHandle

Owned by `AppState::Recording`. Holds the resources for one active recording session.

| Field | Type | Description |
|-------|------|-------------|
| `stop_flag` | `Arc<AtomicBool>` | Set to `true` by TUI to signal the recording thread to stop |
| `result_rx` | `mpsc::Receiver<Result<PathBuf, AppError>>` | Receives the saved file path (or error) when the thread finishes |
| `thread` | `JoinHandle<()>` | Background thread handle |

**Lifecycle**: Created on Record activation. Consumed on Stop activation or thread error. When `stop_flag` is set, the recording thread finalizes the WAV, renames temp→final, and sends the path through `result_rx`.

---

### PlaybackHandle

Owned by `AppState::Playing`. Holds the resources for one active playback session.

| Field | Type | Description |
|-------|------|-------------|
| `stop_flag` | `Arc<AtomicBool>` | Set to `true` by TUI to signal the playback thread to stop |
| `result_rx` | `mpsc::Receiver<Result<(), AppError>>` | Receives `Ok(())` on completion or `Err` on failure |
| `thread` | `JoinHandle<()>` | Background thread handle |
| `source_path` | `PathBuf` | Path of the file being played (for status display) |

**Lifecycle**: Created on Play activation. Consumed on Stop or natural completion (playback reaches end of file).

---

### WavFileEntry

Represents one file in the TUI file list. The list is rebuilt from the recordings directory after each recording and on startup.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Filename only (e.g., `recording_20260514_120000_000.wav`) |
| `path` | `PathBuf` | Absolute path to the WAV file |
| `created_at` | `String` | Human-readable creation timestamp (e.g., `2026-05-27 17:57:00`) |

**UI / Columns**: The TUI now renders a table with two columns: `Name` and `Created At` (see view rendering).

**Ordering**: Files are still sorted lexicographically by `name` descending (most recent first, given timestamp naming).

**Validation**: Only files with `.wav` extension are included. Non-WAV entries in the directory are silently ignored.

---

### TuiContext (transient, not persisted)

Tracks UI cursor state within the TUI session.

| Field | Type | Description |
|-------|------|-------------|
| `selected_index` | `Option<usize>` | Index into `wav_files` of the currently highlighted list item |
| `wav_files` | `Vec<WavFileEntry>` | Current snapshot of the recordings directory |
| `status_message` | `Option<String>` | Transient status or error text shown in the status bar |
| `app_state` | `AppState` | Current state machine variant |

---

## Removed Entities

The following entities from `model/mod.rs` are no longer needed and will be removed:

- `AudioDevice` — device listing was a CLI-only feature; TUI always uses the default host device.
- `PlaybackRequest` — replaced by `PlaybackHandle`.
- `RecordingSession` (current shape) — replaced by `RecordingHandle`; the `status: String` field is replaced by the state machine variant.
- `WavAsset` — replaced by `WavFileEntry`.