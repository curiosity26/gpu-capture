// DRM + GBM + DMA-BUF screen capture
// Uses GBM to create buffers, DMA-BUF to import/export, and EGL to read display content

use ffmpeg_next as ffmpeg;
use std::ffi::{c_void, CString};

// DRM ioctls
const DRM_IOCTL_GEM_OPEN: u32 = 0x400864A1;
const DRM_IOCTL_GEM_CLOSE: u32 = 0x400864A0;
const DRM_IOCTL_GEM_MMAP: u32 = 0xC02064A3;

// GBM
const GBM_BO_USE_SCANOUT: u32 = 1;
const GBM_BO_USE_RENDERING: u32 = 2;

// EGL
const EGL_DEFAULT_DISPLAY: isize = 0;
const EGL_SURFACE_TYPE: i32 = 0x3033;
const EGL_RENDERABLE_TYPE: i32 = 0x3040;
const EGL_RED_SIZE: i32 = 0x3024;
const EGL_GREEN_SIZE: i32 = 0x3025;
const EGL_BLUE_SIZE: i32 = 0x3026;
const EGL_ALPHA_SIZE: i32 = 0x3027;
const EGL_DEPTH_SIZE: i32 = 0x302A;
const EGL_STENCIL_SIZE: i32 = 0x302B;
const EGL_BUFFER_SIZE: i32 = 0x3020;
const EGL_CONFIG_CAVEAT: i32 = 0x302C;
const EGL_SLOW_CONFIG: i32 = 0x3050;
const EGL_NONE: i32 = 0;
const EGL_WIDTH: i32 = 0x3000;
const EGL_HEIGHT: i32 = 0x3001;
const EGL_PBUFFER_BIT: i32 = 0x0001;
const EGL_OPENGL_ES2_BIT: i32 = 0x0004;
const EGL_OPENGL_ES2_BIT_KHR: i32 = 0x0004;

// GBM function pointers
struct GbmFunctions {
    create_device: Option<unsafe extern "C" fn(i32) -> *mut c_void>,
    device_create_buffer: Option<unsafe extern "C" fn(*mut c_void, i32, i32, u32, u32) -> *mut c_void>,
    buffer_get_fd: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    buffer_get_width: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    buffer_get_height: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    buffer_destroy: Option<unsafe extern "C" fn(*mut c_void)>,
    device_destroy: Option<unsafe extern "C" fn(*mut c_void)>,
}

// EGL function pointers
struct EglFunctions {
    get_display: Option<unsafe extern "C" fn(isize) -> *mut c_void>,
    initialize: Option<unsafe extern "C" fn(*mut c_void, *mut i32, *mut i32) -> i32>,
    terminate: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    choose_config: Option<unsafe extern "C" fn(*mut c_void, *const i32, *mut *mut i32, i32, *mut i32) -> i32>,
    create_pbuffer_surface: Option<unsafe extern "C" fn(*mut c_void, *const i32, *const i32) -> *mut c_void>,
    destroy_surface: Option<unsafe extern "C" fn(*mut c_void, *mut c_void) -> i32>,
    bind_api: Option<unsafe extern "C" fn(i32) -> i32>,
    get_current_display: Option<unsafe extern "C" fn() -> *mut c_void>,
    make_current: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void) -> i32>,
    read_pixels: Option<unsafe extern "C" fn(i32, i32, i32, i32, i32, i32, *mut u8)>,
}

fn load_gbm() -> Result<GbmFunctions, String> {
    let lib = unsafe {
        libc::dlopen(CString::new("libgbm.so.1").unwrap().as_ptr(), libc::RTLD_NOW)
    };
    if lib.is_null() {
        return Err("Failed to load libgbm.so.1".to_string());
    }

    let create_device = unsafe {
        libc::dlsym(lib, CString::new("gbm_device_new_with_fd").unwrap().as_ptr())
    };
    let device_create_buffer = unsafe {
        libc::dlsym(lib, CString::new("gbm_bo_create").unwrap().as_ptr())
    };
    let buffer_get_fd = unsafe {
        libc::dlsym(lib, CString::new("gbm_bo_get_fd").unwrap().as_ptr())
    };
    let buffer_get_width = unsafe {
        libc::dlsym(lib, CString::new("gbm_bo_get_width").unwrap().as_ptr())
    };
    let buffer_get_height = unsafe {
        libc::dlsym(lib, CString::new("gbm_bo_get_height").unwrap().as_ptr())
    };
    let buffer_destroy = unsafe {
        libc::dlsym(lib, CString::new("gbm_bo_destroy").unwrap().as_ptr())
    };
    let device_destroy = unsafe {
        libc::dlsym(lib, CString::new("gbm_device_destroy").unwrap().as_ptr())
    };

    unsafe {
        libc::dlclose(lib);
    }

    Ok(GbmFunctions {
        create_device: Some(unsafe { std::mem::transmute(create_device) }),
        device_create_buffer: Some(unsafe { std::mem::transmute(device_create_buffer) }),
        buffer_get_fd: Some(unsafe { std::mem::transmute(buffer_get_fd) }),
        buffer_get_width: Some(unsafe { std::mem::transmute(buffer_get_width) }),
        buffer_get_height: Some(unsafe { std::mem::transmute(buffer_get_height) }),
        buffer_destroy: Some(unsafe { std::mem::transmute(buffer_destroy) }),
        device_destroy: Some(unsafe { std::mem::transmute(device_destroy) }),
    })
}

fn load_egl() -> Result<EglFunctions, String> {
    let lib = unsafe {
        libc::dlopen(CString::new("libEGL.so.1").unwrap().as_ptr(), libc::RTLD_NOW)
    };
    if lib.is_null() {
        return Err("Failed to load libEGL.so.1".to_string());
    }

    let get_display = unsafe {
        libc::dlsym(lib, CString::new("eglGetDisplay").unwrap().as_ptr())
    };
    let initialize = unsafe {
        libc::dlsym(lib, CString::new("eglInitialize").unwrap().as_ptr())
    };
    let terminate = unsafe {
        libc::dlsym(lib, CString::new("eglTerminate").unwrap().as_ptr())
    };
    let choose_config = unsafe {
        libc::dlsym(lib, CString::new("eglChooseConfig").unwrap().as_ptr())
    };
    let create_pbuffer_surface = unsafe {
        libc::dlsym(lib, CString::new("eglCreatePbufferSurface").unwrap().as_ptr())
    };
    let destroy_surface = unsafe {
        libc::dlsym(lib, CString::new("eglDestroySurface").unwrap().as_ptr())
    };
    let bind_api = unsafe {
        libc::dlsym(lib, CString::new("eglBindAPI").unwrap().as_ptr())
    };
    let get_current_display = unsafe {
        libc::dlsym(lib, CString::new("eglGetCurrentDisplay").unwrap().as_ptr())
    };
    let make_current = unsafe {
        libc::dlsym(lib, CString::new("eglMakeCurrent").unwrap().as_ptr())
    };
    let read_pixels = unsafe {
        libc::dlsym(lib, CString::new("eglReadPixels").unwrap().as_ptr())
    };

    unsafe {
        libc::dlclose(lib);
    }

    Ok(EglFunctions {
        get_display: Some(unsafe { std::mem::transmute(get_display) }),
        initialize: Some(unsafe { std::mem::transmute(initialize) }),
        terminate: Some(unsafe { std::mem::transmute(terminate) }),
        choose_config: Some(unsafe { std::mem::transmute(choose_config) }),
        create_pbuffer_surface: Some(unsafe { std::mem::transmute(create_pbuffer_surface) }),
        destroy_surface: Some(unsafe { std::mem::transmute(destroy_surface) }),
        bind_api: Some(unsafe { std::mem::transmute(bind_api) }),
        get_current_display: Some(unsafe { std::mem::transmute(get_current_display) }),
        make_current: Some(unsafe { std::mem::transmute(make_current) }),
        read_pixels: Some(unsafe { std::mem::transmute(read_pixels) }),
    })
}

pub fn start_capture(config: &crate::types::CaptureConfig) -> crate::types::Result<()> {
    println!("Capturing using DRM + GBM + DMA-BUF...");

    // Initialize FFmpeg
    ffmpeg::init().map_err(|e| format!("FFmpeg init failed: {:?}", e))?;

    let width = 1280u32;
    let height = 720u32;

    println!("Capturing {}x{} at {} fps to {}", width, height, config.fps, config.output_path);

    // Open DRM device
    let drm_fd = unsafe {
        libc::open(CString::new("/dev/dri/renderD128").unwrap().as_ptr(), libc::O_RDWR)
    };
    if drm_fd < 0 {
        return Err(crate::types::CaptureError::InitFailed("Failed to open /dev/dri/renderD128".to_string()));
    }

    // Load GBM
    let gbm = load_gbm()?;
    let gbm_device = unsafe {
        gbm.create_device.unwrap()(drm_fd)
    };

    // Load EGL
    let egl = load_egl()?;
    let display = unsafe {
        egl.get_display.unwrap()(EGL_DEFAULT_DISPLAY)
    };

    let mut major = 0;
    let mut minor = 0;
    unsafe {
        egl.initialize.unwrap()(display, &mut major, &mut minor);
    }

    // Choose EGL config
    let attrs = [
        EGL_SURFACE_TYPE,
        EGL_PBUFFER_BIT,
        EGL_RENDERABLE_TYPE,
        EGL_OPENGL_ES2_BIT,
        EGL_RED_SIZE,
        8,
        EGL_GREEN_SIZE,
        8,
        EGL_BLUE_SIZE,
        8,
        EGL_ALPHA_SIZE,
        8,
        EGL_NONE,
    ];

    let mut configs: *mut i32 = std::ptr::null_mut();
    let mut num_configs = 0;
    unsafe {
        egl.choose_config.unwrap()(display, attrs.as_ptr(), &mut configs, 1, &mut num_configs);
    }

    // Create GBM buffer
    let gbm_bo = unsafe {
        gbm.device_create_buffer.unwrap()(gbm_device, width as i32, height as i32, 0x34325241, GBM_BO_USE_SCANOUT | GBM_BO_USE_RENDERING)
    };

    // Get DMA-BUF fd
    let dma_buf_fd = unsafe {
        gbm.buffer_get_fd.unwrap()(gbm_bo)
    };

    // Create EGL surface from GBM buffer
    let surface_attrs = [
        EGL_WIDTH,
        width as i32,
        EGL_HEIGHT,
        height as i32,
        EGL_NONE,
    ];

    let surface = unsafe {
        egl.create_pbuffer_surface.unwrap()(display, configs, surface_attrs.as_ptr())
    };

    // Bind OpenGL ES API
    unsafe {
        egl.bind_api.unwrap()(0x30A2); // EGL_OPENGL_ES2_BIT
    }

    // Make current
    unsafe {
        egl.make_current.unwrap()(display, surface, surface, std::ptr::null_mut());
    }

    // Create encoder context
    let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
        .ok_or_else(|| "H.264 codec not found".to_string())?;

    let mut enc = ffmpeg::codec::context::Context::new_with_codec(codec)
        .encoder()
        .video()?;

    enc.set_width(width);
    enc.set_height(height);
    enc.set_time_base(ffmpeg::Rational::new(1, config.fps as i32));
    enc.set_frame_rate(Some(ffmpeg::Rational::new(config.fps as i32, 1)));
    enc.set_format(ffmpeg::format::Pixel::YUV420P);

    let mut encoder = enc.open()?;

    // Create output file
    let mut oc = ffmpeg::format::output(&config.output_path)?;
    let mut ost = oc.add_stream(encoder.codec().ok_or_else(|| "No encoder codec".to_string())?)?;
    ost.set_parameters(&encoder);
    oc.write_header()?;

    // Create swscale context for RGBA -> YUV420P conversion
    let mut sws = ffmpeg::software::scaling::Context::get(
        ffmpeg::format::Pixel::RGBA, width, height,
        ffmpeg::format::Pixel::YUV420P, width, height,
        ffmpeg::software::scaling::Flags::BILINEAR,
    )?;

    // Capture frames
    let mut frame_count = 0u64;
    let duration_frames = (config.fps * 3) as u64;
    while frame_count < duration_frames {
        // Read pixels from EGL surface
        let mut rgba_data = vec![0u8; (width * height * 4) as usize];
        unsafe {
            egl.read_pixels.unwrap()(0, 0, width as i32, height as i32, 0x1908, 0x4B22, rgba_data.as_mut_ptr());
        }

        // Create FFmpeg frame
        let mut rgba_frame = ffmpeg::frame::Video::empty();
        unsafe {
            rgba_frame.alloc(ffmpeg::format::Pixel::RGBA, width, height);
        }

        // Copy pixel data
        let rgba_ptr = rgba_frame.data_mut(0).as_mut_ptr();
        unsafe {
            std::ptr::copy(rgba_data.as_ptr(), rgba_ptr, rgba_data.len());
        }

        // Convert RGBA to YUV420P
        let mut yuv_frame = ffmpeg::frame::Video::empty();
        unsafe {
            yuv_frame.alloc(ffmpeg::format::Pixel::YUV420P, width, height);
        }

        sws.run(&rgba_frame, &mut yuv_frame)?;
        yuv_frame.set_pts(Some(frame_count as i64));

        // Encode frame
        encoder.send_frame(&yuv_frame)?;

        let mut encoded = ffmpeg::Packet::empty();
        while encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(0);
            let pts = (frame_count as u32 * 1000 / config.fps) as i64;
            encoded.set_pts(Some(pts));
            encoded.set_duration(1000 / config.fps as i64);
            encoded.write_interleaved(&mut oc)?;
        }

        frame_count += 1;
        std::thread::sleep(std::time::Duration::from_millis(1000 / config.fps as u64));
    }

    // Flush encoder
    encoder.send_eof()?;
    let mut encoded = ffmpeg::Packet::empty();
    while encoder.receive_packet(&mut encoded).is_ok() {
        encoded.set_stream(0);
        encoded.write_interleaved(&mut oc)?;
    }

    oc.write_trailer()?;

    // Cleanup
    unsafe {
        egl.destroy_surface.unwrap()(display, surface);
        gbm.buffer_destroy.unwrap()(gbm_bo);
        egl.terminate.unwrap()(display);
        gbm.device_destroy.unwrap()(gbm_device);
        libc::close(drm_fd);
        libc::close(dma_buf_fd);
    }

    println!("Captured {} frames to {}", frame_count, config.output_path);
    Ok(())
}