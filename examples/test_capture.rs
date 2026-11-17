use gpu_capture::{Recorder, CaptureConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CaptureConfig {
        output_path: "recordings/test_output.mp4".to_string(),
        fps: 30,
        quality: gpu_capture::Quality::High,
    };

    let mut recorder = Recorder::new(config)?;
    println!("Starting capture...");
    recorder.start()?;
    println!("Capture complete!");
    let path = recorder.stop()?;
    println!("Output: {}", path);
    Ok(())
}