use std::process::Command;

#[test]
fn play_command_fails_for_missing_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_sound-recorder"))
        .args(["play", "--file", "/tmp/does-not-exist.wav", "--volume", "50"])
        .output()
        .expect("failed to run sound-recorder play command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("file does not exist"));
}
