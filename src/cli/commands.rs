use crate::audio;
use crate::error::AppError;

pub fn list_devices() -> Result<(), AppError> {
    let devices = audio::devices::list_devices()?;

    for device in devices {
        println!(
            "{}\t{}\t{}\t{}",
            device.id,
            match device.direction {
                crate::model::DeviceDirection::Input => "input",
                crate::model::DeviceDirection::Output => "output",
            },
            device.name,
            if device.is_available {
                "available"
            } else {
                "unavailable"
            }
        );
    }

    Ok(())
}

pub fn record(output: String, input_device: Option<String>) -> Result<(), AppError> {
    audio::record::record_to_wav(&output, input_device.as_deref())?;
    println!("recorded {output}");
    Ok(())
}

pub fn play(file: String, output_device: Option<String>, volume: u8) -> Result<(), AppError> {
    audio::playback::play_wav(&file, output_device.as_deref(), volume)?;
    println!("played {file}");
    Ok(())
}