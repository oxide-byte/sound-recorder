# Contract: Monitor Keybindings

**Branch**: `003-continuous-recording` | **Date**: 2026-05-15  
**Scope**: TUI keyboard input contract for the monitoring feature.

---

## Key Assignments

| Key     | State(s) where active          | Action                                          |
|---------|--------------------------------|-------------------------------------------------|
| `m`     | Idle                           | Start sound-activated monitoring                |
| `s`     | Monitoring (any sub-state)     | Stop monitoring (extended from Recording/Playing stop) |
| `s`     | Recording                      | Stop manual recording (existing — unchanged)    |
| `s`     | Playing                        | Stop playback (existing — unchanged)            |
| `r`     | Idle                           | Start manual recording (existing — unchanged)   |
| `p`     | Idle (with file selected)      | Start playback (existing — updated)           |
| `p`     | Playing                        | Cycle Playback Mode (Single → Continuous → Loop → Single) |
| `q`/Esc | Idle only                      | Quit (existing — unchanged)                     |
| `↑`/`k` | Any                            | Navigate list up (existing — unchanged)         |
| `↓`/`j` | Any                            | Navigate list down (existing — unchanged)       |

---

## Rejection Behavior

| Key | Active State    | Response                                                  |
|-----|-----------------|-----------------------------------------------------------|
| `m` | Monitoring      | Silently ignored (already monitoring)                     |
| `m` | Recording       | Silently ignored                                          |
| `m` | Playing         | Status message: `"Stop playback before monitoring"`       |
| `p` | Monitoring      | Status message: `"Stop monitoring before playback"`       |
| `r` | Monitoring      | Silently ignored                                          |

---

## Status Bar Labels by State

| App State                        | Status bar content                                              |
|----------------------------------|-----------------------------------------------------------------|
| Idle, no files                   | `"Ready — press 'r' to record, 'm' to monitor"` (updated)      |
| Idle, files present              | `"Ready — ↑/↓ select  r record  m monitor  p play  q quit"` (updated) |
| Monitoring (Listening)           | `"Monitoring — listening for sound…  's' to stop"`              |
| Monitoring (Capturing)           | `"Capturing — sound detected, recording segment…  's' to stop"` |
| Recording                        | `"Recording… press 's' to stop"` (existing — unchanged)         |
| Playing                          | `"Playing {filename} {mode_indicator} — press 's' to stop"` (updated)|

---

## Button Bar Additions

The existing button bar displays `[ Record ]  [ Play ]  [ Stop ]`. This feature adds a `[ Monitor ]` button between Record and Play.

```
[ Record ]  [ Monitor ]  [ Play ]  [ Stop ]
```

| Button      | Active style (fg color)    | Inactive/disabled style      |
|-------------|----------------------------|------------------------------|
| Record      | Red + Bold (when recording)| Green (idle) / DarkGray      |
| Monitor     | Yellow + Bold (when monitoring or capturing) | Green (idle) / DarkGray |
| Play        | Cyan + Bold (when playing) | Green (idle, file selected) / DarkGray |
| Stop        | Yellow + Bold (when any active audio) | DarkGray (idle) |

The `[ Monitor ]` button uses **Yellow** in active state to visually distinguish it from recording (Red) and playback (Cyan).

---

## Help Text in Status Bar (Idle State)

The default idle status bar text is extended from:

```
Ready — ↑/↓ select  r record  p play  q quit
```

to:

```
Ready — ↑/↓ select  r record  m monitor  p play  q quit
```

The empty-files variant is updated from:

```
Ready — press 'r' to record
```

to:

```
Ready — press 'r' to record or 'm' to monitor
```