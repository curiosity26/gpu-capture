# gpu-capture

Cross-platform GPU-accelerated screen/video capture library in Rust. Captures any video playing on the computer, including DRM-protected content where possible.

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

## Platform Support

| Platform | Status | Backend |
|----------|--------|---------|
| Linux | Working | Wayland screencopy protocol (test frame) |
| macOS | In Progress | ScreenCaptureKit (SCStream) |
| Windows | Planned | Desktop Duplication API |

## Quick Start

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
println!("Recorded to: {}", output_path);
```

## Setup

### Linux (Ubuntu/Debian)

```bash
# Install FFmpeg dependencies
sudo apt-get install libavcodec-dev libavformat-dev libavfilter-dev \
    libavdevice-dev libswscale-dev libswresample-dev libx264-dev pkg-config

# Build
cargo build

# Run test
cargo run --example test_capture
```

### macOS

```bash
# Install FFmpeg via Homebrew
brew install ffmpeg

# Build
cargo build
```

### Windows

```bash
# Install FFmpeg development libraries
# (instructions TBD)
```

## Dependencies

### Rust Crates
- `ffmpeg-next` - FFmpeg bindings for encoding/muxing
- `x11rb` - X11 protocol bindings (Linux)
- `crossbeam-channel` - Thread-safe communication
- `libc` - C standard library bindings
- `zbus` - D-Bus client for Linux portal integration
- `tokio` - Async runtime for D-Bus operations

### System Libraries
- FFmpeg (libavcodec, libavformat, libswscale, etc.)
- libx264 (H.264 encoder)
- libdrm (Linux DRM capture)
- libportal (Linux Wayland portal integration)

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run capture example
cargo run --example test_capture

# Run portal example (Linux)
cargo run --example test_portal
```

## Documentation

- [Project Plan](docs/PLAN.md) - Detailed architecture and roadmap
- [Research](docs/research/) - Platform-specific capture research
  - [FFmpeg x11grab](docs/research/ffmpeg_x11grab.md)
  - [FFmpeg Library vs CLI](docs/research/ffmpeg_library_vs_cli.md)
  - [X11 Capture on Wayland](docs/research/x11_capture_wayland.md)
  - [DRM Capture on Wayland](docs/research/drm_capture_wayland.md)
  - [Wayland Capture Interfaces](docs/research/wayland_capture_interfaces.md)

## License

TBD
