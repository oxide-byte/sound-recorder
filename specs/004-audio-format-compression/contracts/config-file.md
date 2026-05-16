# Contract: Audio Defaults Config File

**Feature**: 004-audio-format-compression
**Owner module**: `src/config/`
**Consumer**: `src/tui/app.rs` (on startup)

This contract defines the surface of `./config/audio.conf` — its on-disk format, the keys it accepts, the values those keys may take, and the exact error messages produced when it is wrong.

---

## File Path

`./config/audio.conf` — resolved relative to the process working directory, identical to how `./recordings/` is resolved. The repository ships a default copy of this file at the same path.

---

## On-Disk Format

Line-based UTF-8 text. Each line is exactly one of:

1. **A blank line** (empty or all-whitespace) — ignored.
2. **A comment** — starts with `#` (leading whitespace allowed). Ignored.
3. **A key-value pair** — `key=value`, with optional surrounding whitespace.

Inline trailing comments are **not** supported (`key=value # comment` is invalid).
Quoted values are **not** supported (`key="value"` becomes the value `"value"`).
Continuation lines are **not** supported.

### Example (the shipped default)

```text
# Sound Recorder — audio defaults
# Edit and restart the app to apply changes.
#
# format: container/format used for new recordings.
#   Supported: wav
#
# compression: bit-depth/sample-format profile.
#   Supported: pcm8, pcm16, pcm24, float32

format=wav
compression=pcm16
```

---

## Recognized Keys

| Key           | Required | Valid values                              | Default if file present but key missing |
|---------------|----------|-------------------------------------------|------------------------------------------|
| `format`      | yes      | `wav` (case-insensitive)                  | n/a — missing required key is an error  |
| `compression` | yes      | `pcm8`, `pcm16`, `pcm24`, `float32`       | n/a — missing required key is an error  |

Any other key is an error (see "Unknown key" below).

---

## Loader API

```rust
pub fn load_or_default(path: &Path) -> Result<AudioDefaultsConfig, AppError>;
```

### Behavior matrix

| Filesystem outcome                         | Return value                                                          |
|-------------------------------------------|-----------------------------------------------------------------------|
| File missing (`io::ErrorKind::NotFound`)  | `Ok(AudioDefaultsConfig { profile: default, source: Fallback })`      |
| File unreadable (other I/O error)         | `Err(AppError::Io(_))`                                                |
| File present, parsed OK, validated OK     | `Ok(AudioDefaultsConfig { profile: parsed,  source: File })`          |
| File present, parse failure               | `Err(AppError::Config(<see error catalog>))`                          |
| File present, validation failure          | `Err(AppError::Config(<see error catalog>))`                          |

`default` here is `AudioOutputProfile::default()` = `wav + pcm16`.

---

## Error Catalog

All messages are stable strings — integration tests assert on them.

| Trigger                                              | Error message                                                                            |
|------------------------------------------------------|------------------------------------------------------------------------------------------|
| Malformed line (no `=`)                              | `config error: line {N}: expected 'key=value', got '{raw}'`                              |
| Empty key (line starts with `=`)                     | `config error: line {N}: empty key`                                                      |
| Empty value (line is `key=`)                         | `config error: line {N}: empty value for key '{key}'`                                    |
| Unknown key                                          | `config error: line {N}: unknown key '{key}' (supported: format, compression)`           |
| Duplicate key                                        | `config error: line {N}: duplicate key '{key}' (already set on line {prior})`            |
| Missing required key after parse                     | `config error: missing required key '{key}' in config/audio.conf`                        |
| Unknown `format`                                     | `config error: unsupported format '{value}'; supported: wav`                             |
| Unknown `compression`                                | `config error: unsupported compression '{value}'; supported: pcm8, pcm16, pcm24, float32`|
| Incompatible (format, compression) pair (future)     | `config error: format '{f}' is not compatible with compression '{c}'`                    |

`{N}` is the 1-based line number; `{raw}` is the original line text (whitespace-trimmed) truncated to 80 chars.

---

## Status Messages (TUI side)

After a successful `load_or_default` call, the TUI surfaces a one-shot status message:

| `source`     | Status message                                                                |
|--------------|--------------------------------------------------------------------------------|
| `File`       | (silent — no status; profile is shown in the footer)                          |
| `Fallback`   | `Using built-in defaults — config/audio.conf not found.`                       |

After a failed load, the TUI surfaces:

> `Audio defaults invalid — fix config/audio.conf: {underlying error message}`

…and **disables** the `r` and `m` actions until the config is reloaded (currently: restart the app).

---

## Test Contracts

Unit tests in `tests/unit/config_parse.rs` MUST cover:

- Blank lines and comments are ignored.
- `key=value` parses with and without surrounding whitespace.
- Case-insensitive identifier parsing (`FORMAT=WAV`, `compression=PCM16` work).
- Each error catalog entry has at least one regression test asserting the exact message text.
- Missing-file path returns `Fallback` source.

Integration test in `tests/integration/config_record.rs` MUST cover:

- Writing a valid config, running one record cycle, and asserting the produced WAV's `WavSpec` matches the configured compression profile.
- Writing an invalid config and asserting `r` / `m` produce a status message and **do not** create any WAV file.