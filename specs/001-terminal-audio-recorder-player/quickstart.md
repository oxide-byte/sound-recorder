# Quickstart

## Prerequisites

- Rust toolchain compatible with `edition = "2024"`
- Available system audio input/output devices

## Build

```bash
cargo build
```

## List devices

```bash
cargo run -- list-devices
```

Expected output shape (tab-separated rows):

```text
default-input	input	Default Input	available
default-output	output	Default Output	available
```

## Record a WAV file (~10s)

```bash
cargo run -- record --output ./sample.wav
```

Expected success output:

```text
recorded ./sample.wav
```

Notes:

- `--input-device <id>` is accepted by CLI but currently does not alter device
  selection (default input is used).

## Play a WAV file

```bash
cargo run -- play --file ./sample.wav --volume 50
```

Expected success output:

```text
played ./sample.wav
```

Notes:

- `--output-device <id>` is accepted by CLI but currently does not alter device
  selection (default output is used).

## Common error examples

Missing file:

```bash
cargo run -- play --file /tmp/does-not-exist.wav --volume 50
```

```text
error: invalid argument: file does not exist: /tmp/does-not-exist.wav
```

Invalid volume:

```bash
cargo run -- play --file ./sample.wav --volume 101
```

```text
error: invalid argument: volume must be between 0 and 100, got 101
```

## Verification

Run regression checks tied to the documented contracts:

```bash
cargo test --test integration
cargo test --test playback_validation
cargo test --test record_validation
```