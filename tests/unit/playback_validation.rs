use sound_recorder::audio::playback::play_wav;

#[test]
fn rejects_out_of_range_volume() {
    let temp = std::env::temp_dir().join("sound-recorder-playback-validation.wav");
    std::fs::write(&temp, b"stub").expect("create temp file");

    let err = play_wav(
        temp.to_str().expect("valid path"),
        None,
        101,
    )
    .expect_err("expected invalid volume error");

    assert!(err.to_string().contains("volume must be between 0 and 100"));
    let _ = std::fs::remove_file(temp);
}
