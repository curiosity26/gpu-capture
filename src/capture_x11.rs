use ffmpeg_next as ffmpeg;

pub fn start_capture(config: &crate::types::CaptureConfig) -> crate::types::Result<()> {
    println!("Capturing screen using FFmpeg x11grab...");

    // Initialize FFmpeg
    ffmpeg::init().map_err(|e| format!("FFmpeg init failed: {:?}", e))?;

    // Open x11grab input - works through Xwayland on Wayland systems
    let ic = ffmpeg::format::input("x11grab")
        .map_err(|e| format!("Failed to open x11grab: {:?}", e))?;

    // Find video stream
    let stream = ic.streams().best(ffmpeg::media::Type::Video)
        .ok_or("No video stream found")?;

    println!("Stream info: width={}, height={}",
             stream.parameters().width(),
             stream.parameters().height());

    // Create encoder context from input stream parameters
    let mut enc_ctx = ffmpeg::codec::context::Context::new_with_codec(
        ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
            .ok_or("H.264 codec not found")?
    ).encoder().video()
        .map_err(|e| format!("Failed to create encoder: {:?}", e))?;

    enc_ctx.set_width(stream.parameters().width());
    enc_ctx.set_height(stream.parameters().height());
    enc_ctx.set_time_base(ffmpeg::Rational::new(1, config.fps as i32));
    enc_ctx.set_frame_rate(Some(ffmpeg::Rational::new(config.fps as i32, 1)));
    enc_ctx.set_format(ffmpeg::format::Pixel::YUV420P);

    let encoder = enc_ctx.open()
        .map_err(|e| format!("Failed to open encoder: {:?}", e))?;

    // Create output file
    let mut oc = ffmpeg::format::output(&config.output_path)
        .map_err(|e| format!("Failed to create output: {:?}", e))?;

    let mut stream_out = oc.add_stream(ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
        .ok_or("No codec")?)
        .map_err(|e| format!("Failed to add stream: {:?}", e))?;
    stream_out.set_time_base(ffmpeg::Rational::new(1, config.fps as i32));

    oc.write_header()
        .map_err(|e| format!("Failed to write header: {:?}", e))?;

    // Capture loop - read packets from input, encode to output
    let mut frame_count = 0u64;
    for (stream, packet) in ic.packets() {
        if stream.index() != stream.index() {
            continue;
        }

        // Decode input frame
        let mut decoded = ffmpeg::frame::Video::empty();
        if stream.decoder().video()?.send_packet(&packet).is_ok() {
            if stream.decoder().video()?.receive_frame(&mut decoded).is_ok() {
                // Encode to output
                encoder.send_frame(&decoded)
                    .map_err(|e| format!("Failed to send frame: {:?}", e))?;

                let mut encoded = ffmpeg::Packet::empty();
                while encoder.receive_packet(&mut encoded).is_ok() {
                    encoded.set_stream(0);
                    encoded.write_interleaved(&mut oc)
                        .map_err(|e| format!("Failed to write packet: {:?}", e))?;
                }

                frame_count += 1;
                if frame_count >= (config.fps * 3) as u64 {
                    break;
                }
            }
        }
    }

    // Flush encoder and finish output
    encoder.send_eof().ok();
    let mut encoded = ffmpeg::Packet::empty();
    while encoder.receive_packet(&mut encoded).is_ok() {
        encoded.set_stream(0);
        encoded.write_interleaved(&mut oc).ok();
    }

    oc.write_trailer().ok();

    println!("Captured {} frames to {}", frame_count, config.output_path);
    Ok(())
}
