use gpu_capture::{Recorder, CaptureConfig};
use std::process::Command;

fn analyze_video(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nAnalyzing video for black frames...");
    
    // Use ffprobe to get frame info
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-count_frames",
            "-select_streams", "v:0",
            "-show_entries", "stream=nb_frames",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path,
        ])
        .output()?;
    
    let frames = String::from_utf8(output.stdout)?.trim().parse::<i32>()?;
    println!("Total frames: {}", frames);
    
    // Check for black frames using ffmpeg
    let output = Command::new("ffmpeg")
        .args(&[
            "-i", path,
            "-vf", "blackdetect=d=1:pix_th=5",
            "-f", "null",
            "-",
        ])
        .output()?;
    
    let stderr = String::from_utf8(output.stderr)?;
    let black_lines: Vec<&str> = stderr.lines()
        .filter(|l| l.contains("black_start"))
        .collect();
    
    if black_lines.is_empty() {
        println!("No black frames detected! DRM capture working correctly.");
    } else {
        println!("Black frames detected:");
        for line in black_lines {
            println!("  {}", line);
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CaptureConfig {
        output_path: "recordings/drm_test.mp4".to_string(),
        fps: 30,
        quality: gpu_capture::Quality::High,
    };

    println!("DRM Capture Test");
    println!("================");
    println!("Capturing for 10 seconds...");
    
    let mut recorder = Recorder::new(config)?;
    recorder.start()?;
    
    // Capture for 10 seconds
    std::thread::sleep(std::time::Duration::from_secs(10));
    
    let path = recorder.stop()?;
    println!("\nCapture complete!");
    println!("Output: {}", path);
    
    // Analyze for black frames
    analyze_video(&path)?;
    
    Ok(())
}