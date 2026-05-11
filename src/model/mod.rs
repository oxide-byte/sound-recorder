#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeviceDirection {
    Input,
    Output,
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub direction: DeviceDirection,
    pub is_available: bool,
}

#[derive(Debug, Clone)]
pub struct RecordingSession {
    pub input_device_id: Option<String>,
    pub output_path: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct WavAsset {
    pub path: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PlaybackRequest {
    pub wav_path: String,
    pub output_device_id: Option<String>,
    pub volume: u8,
}