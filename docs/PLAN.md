# gpu-capture

Cross-platform GPU-accelerated screen/video capture library in Rust.

## Architecture

```
Screen Capture Backend (platform-specific)
    ↓ raw frames (RGB/BGR)
Color Conversion (swscale: RGB → YUV420P)
    ↓ YUV frames
Video Encoder (x264 via FFmpeg)
    ↓ encoded packets
MP4 Muxer (FFmpeg)
    ↓ output.mp4
```

## Current Status

### Linux: Wayland Backend Working ✅
**Working approach:** Wayland screencopy protocol (test frame encoding verified)

**Verified Working:**
- ✅ FFmpeg encoding via ffmpeg-next crate (H.264, MP4)
- ✅ X11 connection via x11rb crate
- ✅ Wayland screencopy protocol (test frame)
- ✅ swscale RGB→YUV420P conversion

**Known Issues:**
- ❌ X11 GetImage fails on Wayland/Xwayland (Match error)
- ❌ FFmpeg library x11grab not available (only CLI binary has it)

### macOS: ScreenCaptureKit Backend Working ✅
**Working approach:** ScreenCaptureKit via Swift FFI

**Verified Working:**
- ✅ Swift ScreenCaptureKit capture implementation
- ✅ FFI bridge between Rust and Swift
- ✅ Live screen capture to MP4 (H.264 encoded)
- ✅ 1280x720 capture at 30fps tested successfully

**Build Process:**
- Swift code compiled with `swiftc -parse-as-library -c` to object file
- Object file archived into static library
- Static library linked via build.rs

## Remaining Tasks

### Phase 1: Complete Wayland Implementation
- [ ] Implement actual Wayland screencopy capture (not just test frame)
- [ ] Add region/window-specific capture support
- [ ] Add audio capture (PulseAudio/ALSA)

### Phase 2: DRM Capture (Linux)
- [x] GBM + DMA-BUF backend for GPU-level capture
- [ ] EGLStream support (NVIDIA) for best DRM compatibility

### Phase 3: Cross-Platform
- [ ] Windows backend (Desktop Duplication API)
- [x] macOS backend (ScreenCaptureKit) - **COMPLETE**

### Phase 4: UI Application
- [ ] CLI tool with recording controls
- [ ] GUI application (TUI or desktop)

## Dependencies

### Rust Crates
- `ffmpeg-next` - FFmpeg bindings for encoding/muxing
- `x11rb` - X11 protocol bindings
- `crossbeam-channel` - Thread-safe communication
- `libc` - C standard library bindings

### System Libraries (Ubuntu/Debian)
```bash
apt-get install libavcodec-dev libavformat-dev libavfilter-dev \
    libavdevice-dev libswscale-dev libswresample-dev libx264-dev pkg-config
```

## API Design

```rust
use gpu_capture::{Recorder, CaptureConfig, Quality};

let config = CaptureConfig {
    output_path: "recording.mp4".to_string(),
    fps: 30,
    quality: Quality::High,
};

let mut recorder = Recorder::new(config)?;
recorder.start()?;
// ... record for some time ...
let output_path = recorder.stop()?;
```
