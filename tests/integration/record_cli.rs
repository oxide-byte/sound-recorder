use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn record_command_creates_wav_file() {
    let output = std::env::temp_dir().join("sound-recorder-integration-record.wav");
    let _ = std::fs::remove_file(&output);

    let started_at = Instant::now();
    let status = Command::new(env!("CARGO_BIN_EXE_sound-recorder"))
        .args(["record", "--output", output.to_str().expect("valid path")])
        .status()
        .expect("failed to launch sound-recorder binary");
    let elapsed = started_at.elapsed();

    assert!(status.success());
    assert!(output.exists());

    let reader = hound::WavReader::open(&output).expect("expected valid wav output");
    let spec = reader.spec();
    let duration_secs = reader.duration() as f64 / spec.sample_rate as f64;

    assert!(elapsed >= Duration::from_millis(9_500));
    assert!((duration_secs - 10.0).abs() < 0.1);

    let _ = std::fs::remove_file(output);
}
