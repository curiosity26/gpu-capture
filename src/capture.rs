use ffmpeg_next as ffmpeg;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ImageFormat};

pub fn start_capture(config: &crate::types::CaptureConfig) -> crate::types::Result<()> {
    println!("Capturing using X11 (x11rb) + FFmpeg libraries (ffmpeg-next)...");

    // Initialize FFmpeg
    ffmpeg::init().map_err(|e| format!("FFmpeg init failed: {:?}", e))?;

    // Initialize X11 connection
    let (conn, screen_num) = x11rb::rust_connection::RustConnection::connect(Some(":0"))
        .map_err(|e| format!("X11 connection failed: {:?}", e))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];

    let width = 640u16;
    let height = 480u16;

    println!("Capturing {}x{} at {} fps to {}", width, height, config.fps, config.output_path);
    println!("Root window: {}", screen.root);
    println!("Screen depth: {}", screen.root_depth);

    // Create encoder context
    let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
        .ok_or_else(|| "H.264 codec not found".to_string())?;

    let mut enc = ffmpeg::codec::context::Context::new_with_codec(codec)
        .encoder()
        .video()?;

    enc.set_width(width as u32);
    enc.set_height(height as u32);
    enc.set_time_base(ffmpeg::Rational::new(1, config.fps as i32));
    enc.set_frame_rate(Some(ffmpeg::Rational::new(config.fps as i32, 1)));
    enc.set_format(ffmpeg::format::Pixel::YUV420P);

    let mut encoder = enc.open()?;

    // Create output file
    let mut oc = ffmpeg::format::output(&config.output_path)?;
    let mut ost = oc.add_stream(encoder.codec().ok_or_else(|| "No encoder codec".to_string())?)?;
    ost.set_parameters(&encoder);
    oc.write_header()?;

    // Create swscale context for RGB -> YUV420P conversion
    let mut sws = ffmpeg::software::scaling::Context::get(
        ffmpeg::format::Pixel::RGB24, width as u32, height as u32,
        ffmpeg::format::Pixel::YUV420P, width as u32, height as u32,
        ffmpeg::software::scaling::Flags::BILINEAR,
    )?;

    // Capture loop
    let mut frame_count = 0u64;
    while frame_count < (config.fps * 3) as u64 {
        // Try different approaches to capture
        let result = conn.get_image(
            ImageFormat::Z_PIXMAP,
            screen.root,
            0, 0,
            width, height,
            !0u32,
        );

        let image = match result {
            Ok(cookie) => cookie.reply()
                .map_err(|e| format!("X11 error: {:?}", e))?,
            Err(e) => {
                return Err(crate::types::CaptureError::InitFailed(format!("Request failed: {:?}", e)));
            }
        };

        // Convert X11 image to AVFrame
        let mut rgb_frame = ffmpeg::frame::Video::empty();
        unsafe {
            rgb_frame.alloc(ffmpeg::format::Pixel::RGB24, width as u32, height as u32);
        }

        // Copy X11 image data to AVFrame
        let rgb_data = rgb_frame.data_mut(0);
        unsafe {
            std::ptr::copy(image.data.as_ptr(), rgb_data.as_mut_ptr() as *mut u8, image.data.len());
        }

        // Convert RGB to YUV420P
        let mut yuv_frame = ffmpeg::frame::Video::empty();
        unsafe {
            yuv_frame.alloc(ffmpeg::format::Pixel::YUV420P, width as u32, height as u32);
        }

        sws.run(&rgb_frame, &mut yuv_frame)?;
        yuv_frame.set_pts(Some(frame_count as i64));

        // Encode frame
        encoder.send_frame(&yuv_frame)?;

        let mut encoded = ffmpeg::Packet::empty();
        while encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(0);
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

    println!("Captured {} frames to {}", frame_count, config.output_path);
    Ok(())
}
