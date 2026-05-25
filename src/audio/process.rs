use std::path::Path;
use crate::error::AppError;
use crate::model::{AudioOutputProfile, CompressionProfile, SupportedFormat};
use crate::audio::record::write_samples_to_wav;

pub fn amplify_wav(path: &Path, factor: f32) -> Result<(), AppError> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|err| AppError::Audio(format!("failed to open wav for amplification: {err}")))?;
    
    let spec = reader.spec();
    
    // Read samples as f32 for easier scaling
    let samples_f32: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Int, 8) => reader
            .samples::<i8>()
            .map(|s| s.map(|v| v as f32 / i8::MAX as f32))
            .collect::<Result<Vec<_>, _>>(),
        (hound::SampleFormat::Int, 16) => reader
            .samples::<i16>()
            .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
            .collect::<Result<Vec<_>, _>>(),
        (hound::SampleFormat::Int, 24) => reader
            .samples::<i32>()
            .map(|s| s.map(|v| v as f32 / 8388607.0)) // 2^23 - 1
            .collect::<Result<Vec<_>, _>>(),
        (hound::SampleFormat::Float, 32) => reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>(),
        _ => return Err(AppError::Audio(format!(
            "unsupported wav format for amplification: {:?} {}-bit",
            spec.sample_format, spec.bits_per_sample
        ))),
    }.map_err(|err| AppError::Audio(format!("failed to read samples: {err}")))?;

    // Amplify and convert back to i16 (as write_samples_to_wav expects i16)
    // Note: write_samples_to_wav in record.rs handles the conversion from i16 to the target profile
    let amplified_samples_i16: Vec<i16> = samples_f32
        .into_iter()
        .map(|s| (s * factor).clamp(-1.0, 1.0))
        .map(|s| (s * i16::MAX as f32) as i16)
        .collect();

    let profile = AudioOutputProfile {
        format: SupportedFormat::Wav,
        compression: match (spec.sample_format, spec.bits_per_sample) {
            (hound::SampleFormat::Int, 8) => CompressionProfile::Pcm8,
            (hound::SampleFormat::Int, 16) => CompressionProfile::Pcm16,
            (hound::SampleFormat::Int, 24) => CompressionProfile::Pcm24,
            (hound::SampleFormat::Float, 32) => CompressionProfile::Float32,
            _ => CompressionProfile::Pcm16, // Fallback
        },
    };

    write_samples_to_wav(path, &amplified_samples_i16, spec.sample_rate, spec.channels, profile)?;

    Ok(())
}