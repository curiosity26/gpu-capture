// macOS screen capture using ScreenCaptureKit via Swift FFI
// The Swift implementation is compiled into libgpu_capture.dylib

use ffmpeg_next as ffmpeg;
use std::ffi::CStr;

// FFI function declarations
extern "C" {
    fn gpu_capture_start(width: i32, height: i32, fps: i32) -> i32;
    fn gpu_capture_get_frame() -> *mut u8;
    fn gpu_capture_get_dimensions() -> *mut i8;
    fn gpu_capture_stop() -> i32;
}

pub fn start_capture(config: &crate::types::CaptureConfig) -> crate::types::Result<()> {
    println!("Capturing using macOS ScreenCaptureKit...");

    // Initialize FFmpeg
    ffmpeg::init().map_err(|e| format!("FFmpeg init failed: {:?}", e))?;

    let width = 1280u32;
    let height = 720u32;

    println!("Capturing {}x{} at {} fps to {}", width, height, config.fps, config.output_path);

    // Start the Swift capture stream
    unsafe {
        gpu_capture_start(width as i32, height as i32, config.fps as i32);
    }

    // Get actual dimensions from the stream
    let actual_width = width;
    let actual_height = height;
    let bytes_per_row = width * 4;
    unsafe {
        let dims = gpu_capture_get_dimensions();
        if !dims.is_null() {
            let dims_str = CStr::from_ptr(dims).to_string_lossy();
            println!("Stream dimensions: {}", dims_str);
            // Parse JSON to get actual dimensions (simplified)
            free(dims as *mut std::ffi::c_void);
        }
    }

    // Create encoder context
    let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
        .ok_or_else(|| "H.264 codec not found".to_string())?;

    let mut enc = ffmpeg::codec::context::Context::new_with_codec(codec)
        .encoder()
        .video()?;

    enc.set_width(actual_width);
    enc.set_height(actual_height);
    enc.set_time_base(ffmpeg::Rational::new(1, config.fps as i32));
    enc.set_frame_rate(Some(ffmpeg::Rational::new(config.fps as i32, 1)));
    enc.set_format(ffmpeg::format::Pixel::YUV420P);

    let mut encoder = enc.open()?;

    // Create output file
    let mut oc = ffmpeg::format::output(&config.output_path)?;
    let mut ost = oc.add_stream(encoder.codec().ok_or_else(|| "No encoder codec".to_string())?)?;
    ost.set_parameters(&encoder);
    oc.write_header()?;

    // Create swscale context for BGRA -> YUV420P conversion
    let mut sws = ffmpeg::software::scaling::Context::get(
        ffmpeg::format::Pixel::BGRA, actual_width, actual_height,
        ffmpeg::format::Pixel::YUV420P, actual_width, actual_height,
        ffmpeg::software::scaling::Flags::BILINEAR,
    )?;

    // Capture frames
    let mut frame_count = 0u64;
    let duration_frames = (config.fps * 3) as u64;
    while frame_count < duration_frames {
        // Get frame from Swift capture
        let frame_ptr = unsafe { gpu_capture_get_frame() };
        if frame_ptr.is_null() {
            println!("Waiting for frame...");
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        // Create FFmpeg frame and copy data
        let mut bgra_frame = ffmpeg::frame::Video::empty();
        unsafe {
            bgra_frame.alloc(ffmpeg::format::Pixel::BGRA, actual_width, actual_height);
        }

        // Copy frame data from Swift buffer
        let bgra_stride = bgra_frame.stride(0);
        unsafe {
            let src = frame_ptr as *const u8;
            for y in 0..actual_height {
                let src_row = src.offset(y as isize * bytes_per_row as isize);
                let dst_row = bgra_frame.data_mut(0).as_mut_ptr().offset(y as isize * (bgra_stride as isize / 4));
                std::ptr::copy(src_row, dst_row, (actual_width * 4) as usize);
            }
        }

        // Convert BGRA to YUV420P
        let mut yuv_frame = ffmpeg::frame::Video::empty();
        unsafe {
            yuv_frame.alloc(ffmpeg::format::Pixel::YUV420P, actual_width, actual_height);
        }

        sws.run(&bgra_frame, &mut yuv_frame)?;
        yuv_frame.set_pts(Some(frame_count as i64));

        // Encode frame
        encoder.send_frame(&yuv_frame)?;

        let mut encoded = ffmpeg::Packet::empty();
        while encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(0);
            // Set proper timestamp and duration based on frame count
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

    // Stop Swift capture
    unsafe {
        gpu_capture_stop();
    }

    println!("Captured {} frames to {}", frame_count, config.output_path);
    Ok(())
}

// Free C string
extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}