# Quickstart: Audio Format Compression

**Feature**: 004-audio-format-compression
**Audience**: Developers verifying the feature end-to-end.

## Prerequisites

- Repository on branch `004-audio-format-compression`.
- A working microphone (the same as for features 001/003).
- `cargo` toolchain matching `Cargo.toml`'s `edition = "2024"`.

---

## 1 — Build

```sh
cargo build
```

Expected: clean build, no new dependencies pulled. Verify with `cargo tree --depth 1` — the list should match `Cargo.toml` exactly.

---

## 2 — Default config file is present

```sh
cat config/audio.conf
```

Expected output:

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

## 3 — Run with the default profile (`wav + pcm16`)

```sh
cargo run
```

In the TUI, press `r`, speak for ~2 seconds, press `s`. Then quit with `q`.

Inspect the produced file:

```sh
ls -lt recordings/ | head -3
# pick the newest recording_*.wav, then:
python3 - <<'EOF'
import wave, sys
w = wave.open(sorted(__import__('os').listdir('recordings'))[-1].join(['recordings/','']))
print('channels', w.getnchannels(), 'samp_width_bytes', w.getsampwidth(), 'rate', w.getframerate())
EOF
```

Expected: `samp_width_bytes` = 2 (16-bit), matching `pcm16`.

---

## 4 — Switch to `pcm24` via config file

```sh
sed -i.bak 's/^compression=pcm16$/compression=pcm24/' config/audio.conf
```

Re-run `cargo run`, record again, quit. Confirm the new file has `samp_width_bytes` = 3 (24-bit).

Existing 16-bit recordings under `recordings/` remain untouched — verify by re-checking one of them with the snippet above.

---

## 5 — Monitor mode honors the profile

With `compression=pcm24` still in the config, run `cargo run`, press `m`, make a brief sound, wait for the segment to finalize, press `s`. The new `recording_*.wav` should also be 24-bit.

---

## 6 — Missing config file → fallback path

```sh
mv config/audio.conf config/audio.conf.parked
cargo run
```

Expected status message in the TUI footer:

> `Using built-in defaults — config/audio.conf not found.`

Record and confirm the file is 16-bit (built-in default). Then:

```sh
mv config/audio.conf.parked config/audio.conf
```

---

## 7 — Invalid config → fail-fast, no recording

```sh
printf 'format=wav\ncompression=lossless-magic\n' > config/audio.conf
cargo run
```

Expected status message:

> `Audio defaults invalid — fix config/audio.conf: config error: unsupported compression 'lossless-magic'; supported: pcm8, pcm16, pcm24, float32`

Press `r` — no recording starts, no file is created in `recordings/`. Press `q` to exit and restore the valid config.

---

## 8 — Unit & integration tests

```sh
cargo test
```

Expected: all tests pass, including the new ones:

- `tests/unit/config_parse.rs`
- `tests/unit/profile_validation.rs`
- `tests/integration/config_record.rs`
- `src/audio/record.rs` `#[cfg(test)]` (WAV spec dispatch)
- existing `src/audio/monitor.rs` `#[cfg(test)]` (still green)

---

## Done

If steps 3–7 produce the expected outcomes and `cargo test` is green, feature 004 is verified end-to-end.