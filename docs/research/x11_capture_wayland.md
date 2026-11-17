# X11 Screen Capture on Wayland/Xwayland

## Problem
X11's `GetImage` request fails with a Match error (error_code 8) when trying to capture the root window on Wayland with Xwayland.

## Root Cause
Under Wayland with Xwayland, the X11 root window (ID 0x405 / 1029) is not the actual screen. The screen is managed by the Wayland compositor, and the X11 root window is just a virtual window that Xwayland creates. The `GetImage` request cannot capture the actual screen content through this virtual window.

## Verified Facts
- Root window ID: 0x405 (1029 decimal)
- Screen depth: 24 planes
- X11 connection works (can connect to :0)
- `GetImage` request fails with Match error (bad_value: 1029)

## Workarounds
1. **Use Wayland screencopy protocol** - Requires wlroots-based compositor (not GNOME/Mutter)
2. **Use gnome-screenshot CLI tool** - Works but spawns external process (not ideal for library)
3. **Use FFmpeg's x11grab via CLI** - Works but spawns external process
4. **Use XShmGetImage** - May have same issue as GetImage

## Conclusion
For a pure library solution on Wayland, we need to use the Wayland screencopy protocol directly. For X11-only systems, the GetImage approach works fine.
