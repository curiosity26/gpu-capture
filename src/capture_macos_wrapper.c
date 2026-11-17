#include <stdint.h>
#include <stdlib.h>

// Declare Swift functions
extern int gpu_capture_start(int width, int height, int fps);
extern void *gpu_capture_get_frame();
extern char *gpu_capture_get_dimensions();
extern int gpu_capture_stop();

// Export with C names
int gpu_capture_start_wrapper(int width, int height, int fps) {
    return gpu_capture_start(width, height, fps);
}

void *gpu_capture_get_frame_wrapper() {
    return gpu_capture_get_frame();
}

char *gpu_capture_get_dimensions_wrapper() {
    return gpu_capture_get_dimensions();
}

int gpu_capture_stop_wrapper() {
    return gpu_capture_stop();
}