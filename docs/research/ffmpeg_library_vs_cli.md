# FFmpeg Library vs CLI for Screen Capture

## Finding: x11grab Support Differs

On this Ubuntu/ARM64 system, the FFmpeg **command-line binary** has x11grab support,
but the **shared libraries** (libavdevice.so) do not.

### Verified Working (CLI)
```bash
ffmpeg -f x11grab -video_size 640x480 -framerate 30 -i :0.0 -t 1 -c:v libx264 output.mp4
```
Produces valid H.264 MP4 (verified with ffprobe).

### Not Working (Library via ffmpeg-next crate)
```rust
let ic = ffmpeg::format::input_with_dictionary(":0.0", opts)?;
```
Fails with: `ffmpeg::Error(1330794744: Protocol not found)`

Even after calling `avdevice_register_all()`, the x11grab protocol is not available
in the shared library version.

## Conclusion
For now, use the subprocess approach (spawning ffmpeg CLI) for screen capture.
The library approach would work on systems where FFmpeg is built with x11grab
support in the shared libraries (e.g., custom FFmpeg builds).

## Alternative: Rebuild FFmpeg
To use the library approach, rebuild FFmpeg with:
```bash
./configure --enable-shared --enable-x11grab
make && make install
```
