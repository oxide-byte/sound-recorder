use sound_recorder::error::AppError;
use sound_recorder::model::{AudioOutputProfile, CompressionProfile, SupportedFormat};

// ── SupportedFormat ───────────────────────────────────────────────────────────

#[test]
fn supported_format_accepts_canonical_id() {
    assert_eq!(SupportedFormat::from_id("wav").unwrap(), SupportedFormat::Wav);
}

#[test]
fn supported_format_is_case_insensitive() {
    assert_eq!(SupportedFormat::from_id("WAV").unwrap(), SupportedFormat::Wav);
    assert_eq!(SupportedFormat::from_id("Wav").unwrap(), SupportedFormat::Wav);
}

#[test]
fn supported_format_trims_whitespace() {
    assert_eq!(SupportedFormat::from_id("  wav  ").unwrap(), SupportedFormat::Wav);
}

#[test]
fn supported_format_rejects_unknown_id_with_exact_message() {
    let err = SupportedFormat::from_id("flac").expect_err("expected error");
    match err {
        AppError::Config(msg) => {
            assert_eq!(msg, "unsupported format 'flac'; supported: wav");
        }
        other => panic!("expected AppError::Config, got {other:?}"),
    }
}

#[test]
fn supported_format_rejects_empty_id() {
    let err = SupportedFormat::from_id("").expect_err("expected error");
    match err {
        AppError::Config(msg) => assert_eq!(msg, "unsupported format ''; supported: wav"),
        other => panic!("expected AppError::Config, got {other:?}"),
    }
}

#[test]
fn supported_format_round_trip() {
    for variant in [SupportedFormat::Wav] {
        let id = variant.as_id();
        assert_eq!(SupportedFormat::from_id(id).unwrap(), variant);
    }
}

// ── CompressionProfile ────────────────────────────────────────────────────────

#[test]
fn compression_accepts_all_canonical_ids() {
    assert_eq!(CompressionProfile::from_id("pcm8").unwrap(), CompressionProfile::Pcm8);
    assert_eq!(CompressionProfile::from_id("pcm16").unwrap(), CompressionProfile::Pcm16);
    assert_eq!(CompressionProfile::from_id("pcm24").unwrap(), CompressionProfile::Pcm24);
    assert_eq!(CompressionProfile::from_id("float32").unwrap(), CompressionProfile::Float32);
}

#[test]
fn compression_is_case_insensitive() {
    assert_eq!(CompressionProfile::from_id("PCM16").unwrap(), CompressionProfile::Pcm16);
    assert_eq!(CompressionProfile::from_id("Float32").unwrap(), CompressionProfile::Float32);
}

#[test]
fn compression_rejects_unknown_id_with_exact_message() {
    let err = CompressionProfile::from_id("lossless-magic").expect_err("expected error");
    match err {
        AppError::Config(msg) => assert_eq!(
            msg,
            "unsupported compression 'lossless-magic'; supported: pcm8, pcm16, pcm24, float32"
        ),
        other => panic!("expected AppError::Config, got {other:?}"),
    }
}

#[test]
fn compression_round_trip_for_every_variant() {
    for variant in [
        CompressionProfile::Pcm8,
        CompressionProfile::Pcm16,
        CompressionProfile::Pcm24,
        CompressionProfile::Float32,
    ] {
        let id = variant.as_id();
        assert_eq!(CompressionProfile::from_id(id).unwrap(), variant);
    }
}

// ── AudioOutputProfile ────────────────────────────────────────────────────────

#[test]
fn audio_output_profile_validates_all_v1_pairs() {
    for compression in [
        CompressionProfile::Pcm8,
        CompressionProfile::Pcm16,
        CompressionProfile::Pcm24,
        CompressionProfile::Float32,
    ] {
        let profile = AudioOutputProfile::validated(SupportedFormat::Wav, compression)
            .unwrap_or_else(|e| panic!("expected wav+{compression:?} to validate, got {e:?}"));
        assert_eq!(profile.format, SupportedFormat::Wav);
        assert_eq!(profile.compression, compression);
    }
}

#[test]
fn audio_output_profile_default_is_wav_pcm16() {
    let profile = AudioOutputProfile::default();
    assert_eq!(profile.format, SupportedFormat::Wav);
    assert_eq!(profile.compression, CompressionProfile::Pcm16);
}
