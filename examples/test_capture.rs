use gpu_capture::Recorder;

fn main() {
    let mut recorder = Recorder::new(gpu_capture::CaptureConfig::default()).expect("Failed to create recorder");

    println!("Starting screen capture...");
    recorder.start().expect("Failed to start recording");

    println!("Recording in progress...");
}
