use sound_recorder::config::{parse_config, ConfigSource};
use sound_recorder::error::AppError;
use sound_recorder::model::{CompressionProfile, SupportedFormat};

fn parse_err(input: &str) -> String {
    match parse_config(input).expect_err("expected error") {
        AppError::Config(msg) => msg,
        other => panic!("expected AppError::Config, got {other:?}"),
    }
}

// ── happy path ────────────────────────────────────────────────────────────────

#[test]
fn parses_minimal_valid_config() {
    let cfg = parse_config("format=wav\ncompression=pcm16\n").unwrap();
    assert_eq!(cfg.profile.format, SupportedFormat::Wav);
    assert_eq!(cfg.profile.compression, CompressionProfile::Pcm16);
    assert_eq!(cfg.source, ConfigSource::File);
}

#[test]
fn parses_with_comments_and_blank_lines() {
    let text = "\
# leading comment
#
   # indented comment

format=wav

# another comment
compression=pcm24
";
    let cfg = parse_config(text).unwrap();
    assert_eq!(cfg.profile.compression, CompressionProfile::Pcm24);
}

#[test]
fn parses_with_surrounding_whitespace() {
    let cfg = parse_config("  format = wav  \n\tcompression\t=\tfloat32\n").unwrap();
    assert_eq!(cfg.profile.format, SupportedFormat::Wav);
    assert_eq!(cfg.profile.compression, CompressionProfile::Float32);
}

#[test]
fn parses_case_insensitive_keys_and_values() {
    let cfg = parse_config("FORMAT=WAV\nCompression=PCM16\n").unwrap();
    assert_eq!(cfg.profile.format, SupportedFormat::Wav);
    assert_eq!(cfg.profile.compression, CompressionProfile::Pcm16);
}

// ── parser errors ─────────────────────────────────────────────────────────────

#[test]
fn errors_on_malformed_line() {
    let msg = parse_err("format=wav\nthis is not a kv pair\ncompression=pcm16\n");
    assert_eq!(
        msg,
        "line 2: expected 'key=value', got 'this is not a kv pair'"
    );
}

#[test]
fn errors_on_empty_key() {
    let msg = parse_err("=wav\n");
    assert_eq!(msg, "line 1: empty key");
}

#[test]
fn errors_on_empty_value() {
    let msg = parse_err("format=\n");
    assert_eq!(msg, "line 1: empty value for key 'format'");
}

#[test]
fn errors_on_unknown_key_preserving_original_case() {
    let msg = parse_err("format=wav\nCompresion=pcm16\n");
    assert_eq!(
        msg,
        "line 2: unknown key 'Compresion' (supported: format, compression)"
    );
}

#[test]
fn errors_on_duplicate_key() {
    let msg = parse_err("format=wav\ncompression=pcm16\ncompression=pcm24\n");
    assert_eq!(
        msg,
        "line 3: duplicate key 'compression' (already set on line 2)"
    );
}

#[test]
fn errors_on_missing_required_format() {
    let msg = parse_err("compression=pcm16\n");
    assert_eq!(msg, "missing required key 'format' in config/audio.conf");
}

#[test]
fn errors_on_missing_required_compression() {
    let msg = parse_err("format=wav\n");
    assert_eq!(
        msg,
        "missing required key 'compression' in config/audio.conf"
    );
}

// ── validation errors (US3 territory but parser surfaces them) ────────────────

#[test]
fn errors_on_unknown_format_value() {
    let msg = parse_err("format=flac\ncompression=pcm16\n");
    assert_eq!(msg, "unsupported format 'flac'; supported: wav");
}

#[test]
fn errors_on_unknown_compression_value() {
    let msg = parse_err("format=wav\ncompression=lossless-magic\n");
    assert_eq!(
        msg,
        "unsupported compression 'lossless-magic'; supported: pcm8, pcm16, pcm24, float32"
    );
}
