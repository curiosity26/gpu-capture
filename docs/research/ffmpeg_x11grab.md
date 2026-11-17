# FFmpeg x11grab Input Format

## Overview
The `x11grab` input format captures the X11 display. Works through Xwayland on Wayland systems.

## Command Line Syntax
```bash
ffmpeg -f x11grab -video_size [WIDTHxHEIGHT] -framerate [FPS] \
    -i :0.0+XOFFSET,YOFFSET [OPTIONS] output.mp4
```

## Key Parameters
- `:0.0` - Display number (use actual display from $DISPLAY)
- `-video_size` - Capture resolution
- `-framerate` - Frames per second
- `+XOFFSET,YOFFSET` - Top-left corner of capture region (optional)

## Rust Implementation via ffmpeg-next
```rust
use ffmpeg_next as ffmpeg;

// Open x11grab input
let ic = ffmpeg::format::input("x11grab")
    .map_err(|e| format!("Failed to open x11grab: {:?}", e))?;

// Find video stream
let stream = ic.streams().best(ffmpeg::media::Type::Video)
    .ok_or("No video stream found")?;

// Get stream parameters
let width = stream.parameters().width();
let height = stream.parameters().height();
```

## Important Notes
- Must set DISPLAY environment variable before calling
- Works with Xwayland (GNOME on Wayland)
- May not capture DRM content (protected surfaces show as black)
