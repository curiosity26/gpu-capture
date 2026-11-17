#include <stdio.h>
#include <stdlib.h>

extern int gpu_capture_start(int width, int height, int fps);
extern void *gpu_capture_get_frame();
extern char *gpu_capture_get_dimensions();
extern int gpu_capture_stop();

int main() {
    printf("Testing Swift FFI...\n");
    
    int result = gpu_capture_start(1280, 720, 30);
    printf("gpu_capture_start returned: %d\n", result);
    
    char *dims = gpu_capture_get_dimensions();
    printf("Dimensions: %s\n", dims);
    free(dims);
    
    gpu_capture_stop();
    printf("Done\n");
    
    return 0;
}