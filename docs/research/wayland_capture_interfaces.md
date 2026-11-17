# Wayland Capture Interfaces

## Available on This System (GNOME/Mutter)
- wl_output (id=3, version=4)
- zxdg_output_manager_v1 (id=4, version=3)

## Not Available
- zwlr_screencopy_manager_v1 (wlroots screencopy protocol)
- Any other screencopy or capture interfaces

## Conclusion
For screen capture on this GNOME/Mutter system, the wlroots screencopy protocol is not available. Must use:
1. FFmpeg x11grab via CLI (works but spawns process)
2. Wayland screencopy protocol (requires wlroots compositor)

## Next Steps
Implement Wayland screencopy protocol for actual capture instead of test frames. For GNOME/Mutter systems, fall back to FFmpeg x11grab via CLI.
