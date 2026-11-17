import ScreenCaptureKit
import Foundation

// Capture a single frame using ScreenCaptureKit
func captureFrame() -> CGImage? {
    return nil
}

// Export for C/Rust FFI
@_exported
func gpu_capture_start(config_path: UnsafePointer<CChar>) -> Int {
    return 0
}

@_exported
func gpu_capture_stop() -> Int {
    return 0
}
