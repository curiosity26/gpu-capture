// macOS DRM capture using CoreGraphics at hardware level
// Captures directly from the display, bypassing the compositor

use ffmpeg_next as ffmpeg;
use std::ffi::CStr;

extern "C" {
    fn gpu_capture_drm_start(width: i32, height: i32, fps: i32) -> i32;
    fn gpu_capture_drm_get_frame() -> *mut u8;
    fn gpu_capture_drm_get_dimensions() -> *mut i8;
    fn gpu_capture_drm_stop() -> i32;
}

pub fn start_capture(config: &crate::types::CaptureConfig) -> crate::types::Result<()> {
    println!("Capturing using macOS DRM (CoreGraphics)...");

    ffmpeg::init().map_err(|e| format!("FFmpeg init failed: {:?}", e))?;

    let width = 1280u32;
    let height = 720u32;

    println!("Capturing {}x{} at {} fps to {}", width, height, config.fps, config.output_path);

    unsafe {
        gpu_capture_drm_start(width as i32, height as i32, config.fps as i32);
    }

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

    let mut oc = ffmpeg::format::output(&config.output_path)?;
    let mut ost = oc.add_stream(encoder.codec().ok_or_else(|| "No encoder codec".to_string())?)?;
    ost.set_parameters(&encoder);
    oc.write_header()?;

    let mut sws = ffmpeg::software::scaling::Context::get(
        ffmpeg::format::Pixel::BGRA, width, height,
        ffmpeg::format::Pixel::YUV420P, width, height,
        ffmpeg::software::scaling::Flags::BILINEAR,
    )?;

    let mut frame_count = 0u64;
    let duration_frames = (config.fps * 3) as u64;
    while frame_count < duration_frames {
        let frame_ptr = unsafe { gpu_capture_drm_get_frame() };
        if frame_ptr.is_null() {
            println!("Waiting for frame...");
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        let mut bgra_frame = ffmpeg::frame::Video::empty();
        unsafe {
            bgra_frame.alloc(ffmpeg::format::Pixel::BGRA, width, height);
        }

        let bgra_stride = bgra_frame.stride(0);
        unsafe {
            let src = frame_ptr as *const u8;
            for y in 0..height {
                let src_row = src.offset(y as isize * width as isize * 4);
                let dst_row = bgra_frame.data_mut(0).as_mut_ptr().offset(y as isize * (bgra_stride as isize / 4));
                std::ptr::copy(src_row, dst_row, (width * 4) as usize);
            }
        }

        let mut yuv_frame = ffmpeg::frame::Video::empty();
        unsafe {
            yuv_frame.alloc(ffmpeg::format::Pixel::YUV420P, width, height);
        }

        sws.run(&bgra_frame, &mut yuv_frame)?;
        yuv_frame.set_pts(Some(frame_count as i64));

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

    encoder.send_eof()?;
    let mut encoded = ffmpeg::Packet::empty();
    while encoder.receive_packet(&mut encoded).is_ok() {
        encoded.set_stream(0);
        encoded.write_interleaved(&mut oc)?;
    }

    oc.write_trailer()?;

    unsafe {
        gpu_capture_drm_stop();
    }

    println!("Captured {} frames to {}", frame_count, config.output_path);
    Ok(())
}

extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}