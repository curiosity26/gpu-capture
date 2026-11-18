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

### macOS: CGDisplayStream Backend Working ✅
**Working approach:** CGDisplayStream via Swift FFI

**Verified Working:**
- ✅ Swift CGDisplayStream capture implementation
- ✅ FFI bridge between Rust and Swift
- ✅ Live screen capture to MP4 (H.264 encoded)
- ✅ 1280x720 capture at 30fps tested successfully
- ✅ Non-DRM content captured correctly

**Known Limitations:**
- ❌ DRM content (Netflix, Apple TV+, etc.) appears as black frames
- DRM protection is at the GPU hardware level (Apple's secure video pipeline)
- Content is decrypted in GPU secure memory and rendered directly to display
- No API exists to intercept or capture DRM content on macOS

**Build Process:**
- Swift code compiled with `swiftc -parse-as-library -c` to object file
- Object file archived into static library
- Static library linked via build.rs

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

## DRM Capture Strategy

### macOS: Not Possible
DRM protection on macOS is implemented at the GPU hardware level:
- Video content is decrypted in the GPU's secure memory
- Rendered directly to the display output, bypassing the compositor
- No API exists to intercept or capture DRM content
- ScreenCaptureKit and CGDisplayStream both fail for DRM content

### Linux: Kernel Capture Adapter (Planned)
DRM on Linux is exposed via the DRM subsystem and can be captured at the kernel level.

**Approach:** Write a Linux kernel module that hooks into the DRM subsystem to capture framebuffer updates before they're sent to the display controller.

**Technical Details:**

1. **Kernel Module Architecture:**
   - Write a loadable kernel module (`gpu_capture.ko`)
   - Hook into DRM subsystem via `drm_mode_config` callbacks
   - Intercept framebuffer updates and copy to DMA-BUF
   - Expose DMA-BUF via `/dev/gpu_capture` device node

2. **Key DRM APIs:**
   - `DRM_IOCTL_MODE_GETFB` - Query current framebuffer info
   - `DRM_IOCTL_GEM_GET_DMA_BUF_FD` - Export GEM object as DMA-BUF
   - `DRM_IOCTL_MODE_PAGE_FLIP` - Hook page flip events for frame boundaries
   - `drm_framebuffer` struct - Access framebuffer metadata (width, height, pitch)

3. **DMA-BUF Sharing:**
   - Use DMA-BUF for zero-copy buffer sharing between kernel and user-space
   - Kernel module exports framebuffer as DMA-BUF
   - User-space library maps DMA-BUF via `mmap()` and reads frame data
   - No data copying between kernel and user-space for frame content

4. **Frame Synchronization:**
   - Use page flip events to detect frame boundaries
   - Signal user-space via eventfd when new frame is available
   - Prevent tearing by capturing during vertical blanking interval

5. **GPU Driver Compatibility:**
   - Must work with i915 (Intel), amdgpu (AMD), nouveau (NVIDIA open-source)
   - NVIDIA proprietary driver may require separate implementation
   - Test on all major GPU drivers before release

**Developer Setup:**
```bash
# Install kernel headers
sudo apt-get install linux-headers-$(uname -r)

# Build kernel module
make -C /lib/modules/$(uname -r)/build M=$(pwd) modules

# Load module
sudo insmod gpu_capture.ko

# Verify device node exists
ls -la /dev/gpu_capture
```

**Validation Testing:**
1. Capture non-DRM content (desktop, games) - verify no black frames
2. Capture DRM content (Netflix, Prime Video) - verify content appears
3. Test at various resolutions (720p, 1080p, 4K)
4. Test at various frame rates (30fps, 60fps, 120fps)
5. Verify no frame tearing or artifacts
6. Test with multiple GPU drivers (i915, amdgpu, nouveau)

**Usage:**
```rust
use gpu_capture::{Recorder, CaptureConfig};

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

## Remaining Tasks

### Phase 1: Complete Wayland Implementation
- [ ] Implement actual Wayland screencopy capture (not just test frame)
- [ ] Add region/window-specific capture support
- [ ] Add audio capture (PulseAudio/ALSA)

### Phase 2: DRM Capture (Linux)
- [x] GBM + DMA-BUF backend for GPU-level capture
- [ ] Kernel capture adapter module
- [ ] EGLStream support (NVIDIA) for best DRM compatibility

### Phase 3: Cross-Platform
- [ ] Windows backend (Desktop Duplication API)
- [x] macOS backend (CGDisplayStream) - **COMPLETE**

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