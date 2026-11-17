import Foundation
import CoreGraphics
import CoreVideo
import CoreMedia

// Global state
var frameBuffer: UnsafeMutablePointer<UInt8>?
var frameWidth = 0
var frameHeight = 0
var frameBytesPerRow = 0
var captureReady = false

@_cdecl("gpu_capture_drm_start")
func gpu_capture_drm_start(width: Int, height: Int, fps: Int) -> Int {
    print("Starting DRM capture with CoreGraphics...")
    
    frameWidth = width
    frameHeight = height
    frameBytesPerRow = width * 4
    frameBuffer = UnsafeMutablePointer<UInt8>.allocate(capacity: frameBytesPerRow * height)
    
    // Capture a test frame to verify it works
    let displayID = CGMainDisplayID()
    let cgImage = CGDisplayCreateImage(displayID)
    
    if let cgImage = cgImage {
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let bitmapInfo = CGBitmapInfo(rawValue: CGImageAlphaInfo.noneSkipFirst.rawValue | 32)
        let context = CGContext(data: frameBuffer, width: width, height: height, bitsPerComponent: 8, bytesPerRow: frameBytesPerRow, space: colorSpace, bitmapInfo: bitmapInfo.rawValue)
        
        if let context = context {
            context.draw(cgImage, in: CGRect(x: 0, y: 0, width: width, height: height))
        }
        
        captureReady = true
        print("DRM capture started - test frame captured")
    } else {
        print("Error capturing test frame")
        return -1
    }
    
    return 0
}

@_cdecl("gpu_capture_drm_get_frame")
func gpu_capture_drm_get_frame() -> UnsafeMutablePointer<UInt8>? {
    // Capture a new frame each time
    let displayID = CGMainDisplayID()
    let cgImage = CGDisplayCreateImage(displayID)
    
    if let cgImage = cgImage {
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let bitmapInfo = CGBitmapInfo(rawValue: CGImageAlphaInfo.noneSkipFirst.rawValue | 32)
        let context = CGContext(data: frameBuffer, width: frameWidth, height: frameHeight, bitsPerComponent: 8, bytesPerRow: frameBytesPerRow, space: colorSpace, bitmapInfo: bitmapInfo.rawValue)
        
        if let context = context {
            context.draw(cgImage, in: CGRect(x: 0, y: 0, width: frameWidth, height: frameHeight))
        }
    }
    
    return frameBuffer
}

@_cdecl("gpu_capture_drm_get_dimensions")
func gpu_capture_drm_get_dimensions() -> UnsafeMutablePointer<CChar> {
    let json = "{\"width\":\(frameWidth),\"height\":\(frameHeight),\"bytes_per_row\":\(frameBytesPerRow)}"
    return strdup(json)
}

@_cdecl("gpu_capture_drm_stop")
func gpu_capture_drm_stop() -> Int {
    if let buf = frameBuffer {
        buf.deallocate()
    }
    frameBuffer = nil
    
    print("DRM capture stopped")
    return 0
}