//! GPU Capture - Cross-platform screen/video capture library
//!
//! Captures any video playing on the computer, including DRM-protected content where possible.
//! Uses platform-specific capture backends with FFmpeg encoding to MP4 output.

pub mod types;
pub mod encoder;
pub mod capture_portal;
pub mod capture_wayland;
pub mod capture_macos;
pub mod capture_macos_drm;
pub mod capture_drm;

pub use types::{CaptureConfig, Quality};
use crate::types::*;
/// Main recorder API
pub struct Recorder {
    config: CaptureConfig,
}

impl Recorder {
    pub fn new(config: CaptureConfig) -> Result<Self> {
        Ok(Self { config })
    }

    pub fn start(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            // Try DRM capture first, fall back to Wayland
            match capture_drm::start_capture(&self.config) {
                Ok(_) => {}
                Err(e) => {
                    println!("DRM capture failed: {}", e);
                    capture_wayland::start_capture(&self.config)?;
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            capture_macos_drm::start_capture(&self.config)?;
        }

        Ok(())
    }

    pub fn stop(&mut self) -> Result<String> {
        Ok(self.config.output_path.clone())
    }
}
