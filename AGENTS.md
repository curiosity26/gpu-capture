# gpu-capture Project Conventions

## Code Style
- No unused imports or variables
- Use `std::process::Command` directly, not `{Command, Stdio}` unless needed
- Error handling: use `map_err` with descriptive messages
- Use `ffmpeg_next` crate for FFmpeg bindings (not raw FFI)

## Architecture
- Platform-specific capture backends in separate modules (capture_x11.rs, capture_wayland.rs)
- Unified API through `Recorder` struct
- Use FFmpeg's x11grab input format for Linux capture (works through Xwayland)
- H.264 encoding via libx264, MP4 output
