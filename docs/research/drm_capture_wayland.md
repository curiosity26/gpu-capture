# DRM Capture on Wayland with Virtio GPU

## System Configuration
- GPU: Red Hat Virtio 1.0 GPU (Parallels VM)
- DRM driver: virtio_gpu
- DRM devices: /dev/dri/card1, /dev/dri/renderD128

## Findings

### DRM Device Access
- Can open /dev/dri/card1 and /dev/dri/renderD128
- DRM_IOCTL_GET_MAGIC works (returns 1)
- DRM_IOCTL_VERSION segfaults (not supported by virtio_gpu)

### KMS Ioctls Not Supported
- DRM_IOCTL_MODE_GETRESOURCES: Works, returns 0 framebuffers
- DRM_IOCTL_MODE_GETCRTC: "No such file or directory" (CRTC IDs don't exist)
- DRM_IOCTL_MODE_GETFB2: "No such file or directory" (FB IDs don't exist)

### Root Cause
The virtio GPU driver uses a different API for display management than standard KMS. It doesn't create framebuffer objects that userspace can query. The Wayland compositor manages the display directly through the virtio GPU's proprietary interface.

## Conclusion
For screen capture on this system, DRM/KMS capture is not possible. Must use:
1. Wayland screencopy protocol (requires wlroots compositor)
2. X11 capture (doesn't work on Wayland/Xwayland)
3. FFmpeg x11grab via CLI (works but spawns process)

## Next Steps
Implement Wayland screencopy protocol for actual capture instead of test frames.
