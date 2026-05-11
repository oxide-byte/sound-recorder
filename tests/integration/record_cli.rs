use std::process::Command;

#[test]
fn record_command_creates_wav_file() {
    let output = std::env::temp_dir().join("sound-recorder-integration-record.wav");
    let _ = std::fs::remove_file(&output);

    let status = Command::new(env!("CARGO_BIN_EXE_sound-recorder"))
        .args(["record", "--output", output.to_str().expect("valid path")])
        .status()
        .expect("failed to launch sound-recorder binary");

    assert!(status.success());
    assert!(output.exists());

    let _ = std::fs::remove_file(output);
}
