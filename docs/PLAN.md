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

## Current Status: Wayland Backend Working ✅

**Working approach:** Wayland screencopy protocol (test frame encoding verified)

### Verified Working
- ✅ FFmpeg encoding via ffmpeg-next crate (H.264, MP4)
- ✅ X11 connection via x11rb crate
- ✅ Wayland screencopy protocol (test frame)
- ✅ swscale RGB→YUV420P conversion

### Known Issues
- ❌ X11 GetImage fails on Wayland/Xwayland (Match error)
- ❌ FFmpeg library x11grab not available (only CLI binary has it)

## Key Findings

### FFmpeg Library vs CLI
- FFmpeg **CLI binary** has x11grab support
- FFmpeg **shared libraries** (libavdevice.so) do NOT have x11grab on this system
- For library-based capture, must use X11/Wayland APIs directly

### X11 on Wayland
- X11 connection works via x11rb
- Root window ID is correct (0x405)
- GetImage fails with Match error (error_code 8)
- Root cause: Xwayland virtual window doesn't contain actual screen content

### Wayland Approach
- Use wlroots screencopy protocol
- Requires wlroots-based compositor (not GNOME/Mutter)
- Test frame encoding verified working

## Remaining Tasks

### Phase 1: Complete Wayland Implementation
- [ ] Implement actual Wayland screencopy capture (not just test frame)
- [ ] Add region/window-specific capture support
- [ ] Add audio capture (PulseAudio/ALSA)

### Phase 2: DRM Capture (Linux)
- [ ] GBM + DMA-BUF backend for GPU-level capture
- [ ] EGLStream support (NVIDIA) for best DRM compatibility

### Phase 3: Cross-Platform
- [ ] Windows backend (Desktop Duplication API)
- [ ] macOS backend (ScreenCaptureKit)

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
