use crate::error::AppError;
use crate::model::{AudioDevice, DeviceDirection};

pub fn list_devices() -> Result<Vec<AudioDevice>, AppError> {
    let _host = cpal::default_host();

    Ok(vec![
        AudioDevice {
            id: "default-input".to_string(),
            name: "Default Input".to_string(),
            direction: DeviceDirection::Input,
            is_available: true,
        },
        AudioDevice {
            id: "default-output".to_string(),
            name: "Default Output".to_string(),
            direction: DeviceDirection::Output,
            is_available: true,
        },
    ])
}
