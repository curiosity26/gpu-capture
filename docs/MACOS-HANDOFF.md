# macOS Handoff

Project: gpu-capture — Cross-platform GPU-accelerated screen/video capture library in Rust
Repo: https://github.com/curiosity26/gpu-capture (clone this)
Working directory: ~/Projects/gpu-capture

## Goal
Build a Rust library that captures ANY video playing on the computer, including DRM-protected content. Must work on Mac, Windows, and Linux. Output to MP4 using H.264 encoding via FFmpeg libraries (not CLI). This is a LIBRARY, not a CLI tool — users will import it and use the API programmatically.

## Architecture
Platform-specific capture backends behind a unified Rust API:
  Screen Capture Backend → raw frames → swscale (RGB→YUV420P) → x264 encoder → MP4 muxer → output.mp4

The library exposes a simple API:
```rust
let config = CaptureConfig { output_path: "recording.mp4", fps: 30, quality: Quality::High };
let mut recorder = Recorder::new(config)?;
recorder.start()?;
// ... record ...
let path = recorder.stop()?;
```

## Current State
- **Linux:** Working (test frame encoding verified). Uses Wayland screencopy protocol. Research done on DRM capture (GBM+DMA-BUF), X11 capture (fails on Wayland/Xwayland), and xdg-desktop-portal integration. See docs/research/ for details.
- **macOS:** Started but incomplete. capture_macos.rs has test frame encoding. capture_macos.swift is a stub that needs real ScreenCaptureKit implementation.
- **Windows:** Not started (Desktop Duplication API planned).

## What You Need to Do (macOS)
1. Clone the repo: `git clone https://github.com/curiosity26/gpu-capture ~/Projects/gpu-capture`
2. Install dependencies: `brew install ffmpeg` (for FFmpeg libraries)
3. Implement real ScreenCaptureKit capture in src/capture_macos.swift:
   - Use SCShareableContent to list available screens/windows
   - Use SCStream for live capture with delegate callbacks
   - Configure SCStreamConfiguration for resolution/format
   - Capture CGImage frames from the stream
   - Export functions via FFI for Rust to call
4. Compile Swift into a dynamic library (.dylib)
5. Update src/capture_macos.rs to call the real Swift functions via FFI
6. Test that it captures actual screen content (not just test frames)
7. Update docs/PLAN.md with progress

## Key Technical Decisions
- Use FFmpeg libraries via ffmpeg-next crate (NOT spawning ffmpeg CLI process)
- Use swscale for color conversion (RGB→YUV420P)
- Use x264 for H.264 encoding
- Platform-specific backends in separate modules (capture_macos.rs, capture_linux.rs, etc.)
- Unified API through Recorder struct

## Relevant Files
- src/lib.rs — Main library API
- src/types.rs — CaptureConfig, Quality, Rect types
- src/encoder.rs — FFmpeg encoder (working)
- src/capture_macos.rs — macOS capture (needs real implementation)
- src/capture_macos.swift — Swift FFI stub (needs real implementation)
- docs/PLAN.md — Detailed project plan and progress
- docs/research/ — Platform-specific research

## Dependencies
- Swift 5.9+ (for ScreenCaptureKit)
- macOS 11.0+ (ScreenCaptureKit introduced)
- FFmpeg libraries (brew install ffmpeg)
- Rust toolchain (cargo)

## Note
The Linux work is complete. Focus on getting macOS capture working with real screen content. DRM capture (GBM+DMA-BUF on Linux, hardware capture on Mac) is a future phase after basic capture works.
