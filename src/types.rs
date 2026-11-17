use std::fmt;

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub output_path: String,
    pub fps: u32,
    pub quality: Quality,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            output_path: "recording.mp4".to_string(),
            fps: 30,
            quality: Quality::High,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Quality {
    Low,
    Medium,
    High,
}

impl Quality {
    pub fn bitrate(&self) -> i32 {
        match self {
            Quality::Low => 2_500_000,
            Quality::Medium => 5_000_000,
            Quality::High => 10_000_000,
        }
    }
}

pub type Result<T> = std::result::Result<T, CaptureError>;

#[derive(Debug)]
pub enum CaptureError {
    InitFailed(String),
    EncodingFailed(String),
}

impl From<String> for CaptureError {
    fn from(s: String) -> Self {
        CaptureError::InitFailed(s)
    }
}

impl From<ffmpeg_next::Error> for CaptureError {
    fn from(e: ffmpeg_next::Error) -> Self {
        CaptureError::InitFailed(format!("FFmpeg error: {:?}", e))
    }
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaptureError::InitFailed(msg) => write!(f, "Initialization failed: {}", msg),
            CaptureError::EncodingFailed(msg) => write!(f, "Encoding failed: {}", msg),
        }
    }
}

impl std::error::Error for CaptureError {}
