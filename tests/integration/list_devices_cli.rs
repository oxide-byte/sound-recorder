use std::process::Command;

#[test]
fn list_devices_command_outputs_input_or_output_rows() {
    let output = Command::new(env!("CARGO_BIN_EXE_sound-recorder"))
        .arg("list-devices")
        .output()
        .expect("failed to run list-devices command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("input") || stdout.contains("output"));
}
