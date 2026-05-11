use sound_recorder::audio::record::validate_record_request;

#[test]
fn rejects_empty_output_path() {
    let err = validate_record_request("   ").expect_err("expected validation error");
    assert!(err.to_string().contains("output path cannot be empty"));
}

#[test]
fn rejects_non_existing_parent_directory() {
    let err = validate_record_request("/this/path/should/not/exist/record.wav")
        .expect_err("expected validation error");
    assert!(err.to_string().contains("output directory does not exist"));
}
