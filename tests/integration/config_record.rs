use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sound_recorder::audio::record::write_samples_to_wav;
use sound_recorder::config::{load_or_default, ConfigSource};
use sound_recorder::model::{AudioOutputProfile, CompressionProfile, SupportedFormat};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_temp_dir(label: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "sound_recorder_cfg_{}_{}_{}_{}",
        label,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos(),
        n,
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn synth_samples() -> Vec<i16> {
    (0..4096i32).map(|n| ((n * 17) as i16).wrapping_mul(3)).collect()
}

#[test]
fn loads_valid_config_and_writer_honors_pcm24() {
    let dir = unique_temp_dir("valid_pcm24");
    let cfg_path = dir.join("audio.conf");
    fs::write(&cfg_path, "format=wav\ncompression=pcm24\n").unwrap();

    let defaults = load_or_default(&cfg_path).unwrap();
    assert_eq!(defaults.source, ConfigSource::File);
    assert_eq!(defaults.profile.format, SupportedFormat::Wav);
    assert_eq!(defaults.profile.compression, CompressionProfile::Pcm24);

    let wav_path = dir.join("out.wav");
    let samples = synth_samples();
    write_samples_to_wav(&wav_path, &samples, 44_100, 1, defaults.profile).unwrap();

    let spec = hound::WavReader::open(&wav_path).unwrap().spec();
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);
    assert_eq!(spec.bits_per_sample, 24);
    assert_eq!(spec.channels, 1);
    assert_eq!(spec.sample_rate, 44_100);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_file_falls_back_to_built_in_defaults() {
    let dir = unique_temp_dir("missing");
    let cfg_path = dir.join("does-not-exist.conf");

    let defaults = load_or_default(&cfg_path).unwrap();
    assert_eq!(defaults.source, ConfigSource::Fallback);
    assert_eq!(defaults.profile, AudioOutputProfile::default());

    let wav_path = dir.join("fallback.wav");
    let samples = synth_samples();
    write_samples_to_wav(&wav_path, &samples, 44_100, 1, defaults.profile).unwrap();

    let spec = hound::WavReader::open(&wav_path).unwrap().spec();
    assert_eq!(spec.sample_format, hound::SampleFormat::Int);
    assert_eq!(spec.bits_per_sample, 16);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn invalid_compression_value_returns_documented_error_and_no_file_is_written() {
    use sound_recorder::config::load_or_default;
    use sound_recorder::error::AppError;

    let dir = unique_temp_dir("invalid");
    let cfg_path = dir.join("audio.conf");
    fs::write(&cfg_path, "format=wav\ncompression=lossless-magic\n").unwrap();

    let err = load_or_default(&cfg_path).expect_err("expected validation error");

    match err {
        AppError::Config(msg) => {
            assert_eq!(
                msg,
                "unsupported compression 'lossless-magic'; supported: pcm8, pcm16, pcm24, float32"
            );
            assert_eq!(
                format!("{}", AppError::Config(msg)),
                "config error: unsupported compression 'lossless-magic'; supported: pcm8, pcm16, pcm24, float32"
            );
        }
        other => panic!("expected AppError::Config, got {other:?}"),
    }

    let entries: Vec<_> = fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).collect();
    let wavs = entries
        .iter()
        .filter(|e| {
            e.path()
                .extension()
                .map(|x| x.eq_ignore_ascii_case("wav"))
                .unwrap_or(false)
        })
        .count();
    assert_eq!(wavs, 0, "no WAV file should be written for invalid config");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn loads_float32_config_and_writer_honors_it() {
    let dir = unique_temp_dir("valid_float32");
    let cfg_path = dir.join("audio.conf");
    fs::write(&cfg_path, "format=wav\ncompression=float32\n").unwrap();

    let defaults = load_or_default(&cfg_path).unwrap();
    assert_eq!(defaults.profile.compression, CompressionProfile::Float32);

    let wav_path = dir.join("out.wav");
    let samples = synth_samples();
    write_samples_to_wav(&wav_path, &samples, 48_000, 2, defaults.profile).unwrap();

    let spec = hound::WavReader::open(&wav_path).unwrap().spec();
    assert_eq!(spec.sample_format, hound::SampleFormat::Float);
    assert_eq!(spec.bits_per_sample, 32);
    assert_eq!(spec.channels, 2);
    assert_eq!(spec.sample_rate, 48_000);

    let _ = fs::remove_dir_all(&dir);
}
