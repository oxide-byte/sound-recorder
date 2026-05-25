# TUI Key Bindings Contract

**Feature**: 002-tui-recorder-ui  
**Date**: 2026-05-14

## Overview

All user interaction with the TUI is keyboard-driven. Key bindings are deterministic: each key produces a specific action in a specific application state. Keys that are not valid in the current state are silently ignored (no crash, no spurious action).

## Bindings

| Key | State Required | Action |
|-----|---------------|--------|
| `r` | Idle | Start recording → transition to Recording state |
| `p` | Idle + file selected | Start playback of selected file → transition to Playing state |
| `d` / `Del` | Idle + file selected | Delete selected file from disk and list |
| `a` | Idle + file selected | Amplify selected file (2x gain) |
| `s` | Recording or Playing | Stop active operation → transition to Idle state |
| `↑` / `k` | Any | Move selection up in the WAV file list |
| `↓` / `j` | Any | Move selection down in the WAV file list |
| `q` | Idle | Quit application (clean terminal restore) |
| `Esc` | Idle | Quit application (clean terminal restore) |

## State-Guard Rules

- `r` pressed while Recording or Playing: **ignored**.
- `p` pressed while Recording or Playing: **ignored**.
- `p` pressed while Idle with no file selected (empty list or no highlight): **ignored**; status bar shows `"No file selected"`.
- `d` / `a` pressed while Idle with no file selected: **ignored**.
- `d` / `a` pressed while Recording or Playing: **ignored**; status bar shows `"Stop activity before [deleting/amplifying]"`.
- `s` pressed while Idle: **ignored**.
- `q` / `Esc` pressed while Recording or Playing: **ignored** (user must Stop first).

## Status Bar Feedback

The status bar at the bottom of the TUI reflects the current state:

| State | Status Text |
|-------|-------------|
| Idle, no files | `Ready — press 'r' to record` |
| Idle, files present | `Ready — ↑/↓ select  r record  m monitor  p play  d delete  a amplify  q quit` |
| Recording | `Recording… press 's' to stop` |
| Playing | `Playing <filename> — press 's' to stop` |
| Error | `Error: <message>` (clears on next key press) |