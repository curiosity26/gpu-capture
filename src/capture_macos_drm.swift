import Foundation
import Metal
import MetalKit
import CoreGraphics
import CoreVideo
import CoreMedia
import VideoToolbox

// Global state
var metalDevice: MTLDevice?
var metalCommandQueue: MTLCommandQueue?
var captureTexture: MTLTexture?
var frameBuffer: UnsafeMutablePointer<UInt8>?
var frameWidth = 0
var frameHeight = 0
var frameBytesPerRow = 0
var captureReady = false

let frameAvailableHandler: CGDisplayStreamFrameAvailableHandler = { status, displayTime, surface, update in
    if status == CGDisplayStreamFrameStatus.frameComplete, let surface = surface {
        // IOSurface contains the frame data
        let width = IOSurfaceGetWidth(surface)
        let height = IOSurfaceGetHeight(surface)
        
        if frameWidth != width || frameHeight != height {
            frameWidth = width
            frameHeight = height
            frameBytesPerRow = width * 4
            if let buf = frameBuffer {
                buf.deallocate()
            }
            frameBuffer = UnsafeMutablePointer<UInt8>.allocate(capacity: frameBytesPerRow * height)
        }
        
        // Copy IOSurface data to our buffer
        let baseAddress = IOSurfaceGetBaseAddress(surface)
        let src = baseAddress.bindMemory(to: UInt8.self, capacity: frameBytesPerRow * height)
        memcpy(frameBuffer!, src, frameBytesPerRow * height)
        
        captureReady = true
    }
}

@_cdecl("gpu_capture_drm_start")
func gpu_capture_drm_start(width: Int, height: Int, fps: Int) -> Int {
    print("Starting GPU-level DRM capture with CGDisplayStream...")
    
    frameWidth = width
    frameHeight = height
    frameBytesPerRow = width * 4
    frameBuffer = UnsafeMutablePointer<UInt8>.allocate(capacity: frameBytesPerRow * height)
    
    // Get Metal device
    guard let device = MTLCreateSystemDefaultDevice() else {
        print("Error: No Metal device")
        return -1
    }
    metalDevice = device
    
    // Create command queue
    let cmdQueue = device.makeCommandQueue()
    metalCommandQueue = cmdQueue
    
    // Create display stream
    let displayID = CGMainDisplayID()
    let stream = CGDisplayStream(display: displayID, outputWidth: width, outputHeight: height, pixelFormat: Int32(kCVPixelFormatType_32BGRA), properties: nil, handler: frameAvailableHandler)
    
    // Start the stream
    stream?.start()
    
    // Wait for first frame
    Thread.sleep(forTimeInterval: 1.0)
    
    print("Metal device: \(device.name)")
    print("GPU capture ready")
    
    captureReady = true
    return 0
}

@_cdecl("gpu_capture_drm_get_frame")
func gpu_capture_drm_get_frame() -> UnsafeMutablePointer<UInt8>? {
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
    
    print("GPU capture stopped")
    return 0
}