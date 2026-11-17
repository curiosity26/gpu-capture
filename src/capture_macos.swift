import ScreenCaptureKit
import Foundation
import CoreGraphics
import CoreMedia
import CoreVideo

// Global state
var stream: SCStream?
var frameBuffer: UnsafeMutablePointer<UInt8>?
var frameWidth = 0
var frameHeight = 0
var frameBytesPerRow = 0
var captureReady = false

let streamOutput = StreamOutput()
let streamDelegate = StreamDelegate()

class StreamOutput: NSObject, SCStreamOutput {
    func stream(_ stream: SCStream, didOutputSampleBuffer sampleBuffer: CMSampleBuffer, of type: SCStreamOutputType) {
        guard type == .screen else { return }
        
        guard let imageBuffer = CMSampleBufferGetImageBuffer(sampleBuffer) else { return }
        
        CVPixelBufferLockBaseAddress(imageBuffer, .readOnly)
        
        let width = CVPixelBufferGetWidth(imageBuffer)
        let height = CVPixelBufferGetHeight(imageBuffer)
        let bytesPerRow = CVPixelBufferGetBytesPerRow(imageBuffer)
        let baseAddress = CVPixelBufferGetBaseAddress(imageBuffer)
        
        if frameWidth != width || frameHeight != height {
            frameWidth = width
            frameHeight = height
            frameBytesPerRow = bytesPerRow
            if let buf = frameBuffer {
                buf.deallocate()
            }
            frameBuffer = UnsafeMutablePointer<UInt8>.allocate(capacity: bytesPerRow * height)
        }
        
        if let dest = frameBuffer, let src = baseAddress {
            memcpy(dest, src, bytesPerRow * height)
        }
        
        CVPixelBufferUnlockBaseAddress(imageBuffer, .readOnly)
    }
}

class StreamDelegate: NSObject, SCStreamDelegate {
    func stream(_ stream: SCStream, didStopWithError error: Error) {
        print("Stream stopped: \(error)")
    }
}

@_cdecl("gpu_capture_start")
func gpu_capture_start(width: Int, height: Int, fps: Int) -> Int {
    let config = SCStreamConfiguration()
    config.width = width
    config.height = height
    config.pixelFormat = kCVPixelFormatType_32BGRA
    
    let semaphore = DispatchSemaphore(value: 0)
    
    SCShareableContent.getWithCompletionHandler { content, error in
        if let error = error {
            print("Error getting shareable content: \(error)")
            semaphore.signal()
            return
        }
        
        if let display = content?.displays.first {
            let filter = SCContentFilter(display: display, excludingWindows: [])
            let newStream = SCStream(filter: filter, configuration: config, delegate: streamDelegate)
            stream = newStream
            
            try? newStream.addStreamOutput(streamOutput, type: .screen, sampleHandlerQueue: nil)
            
            newStream.startCapture { error in
                if let error = error {
                    print("Start error: \(error)")
                } else {
                    captureReady = true
                }
                semaphore.signal()
            }
        } else {
            print("No displays found")
            semaphore.signal()
        }
    }
    
    // Wait for capture to start
    semaphore.wait()
    
    return captureReady ? 0 : -1
}

@_cdecl("gpu_capture_get_frame")
func gpu_capture_get_frame() -> UnsafeMutablePointer<UInt8>? {
    return frameBuffer
}

@_cdecl("gpu_capture_get_dimensions")
func gpu_capture_get_dimensions() -> UnsafeMutablePointer<CChar> {
    let json = "{\"width\":\(frameWidth),\"height\":\(frameHeight),\"bytes_per_row\":\(frameBytesPerRow)}"
    return strdup(json)
}

@_cdecl("gpu_capture_stop")
func gpu_capture_stop() -> Int {
    stream?.stopCapture()
    stream = nil
    if let buf = frameBuffer {
        buf.deallocate()
    }
    frameBuffer = nil
    return 0
}