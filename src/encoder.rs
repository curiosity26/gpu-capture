use ffmpeg_next as ffmpeg;

pub struct VideoEncoder {
    encoder: ffmpeg::encoder::Video,
    output: ffmpeg::format::context::Output,
}

impl VideoEncoder {
    pub fn new(path: &str, w: u32, h: u32, fps: u32, bitrate: i32) -> Result<Self, String> {
        ffmpeg::init().map_err(|e| format!("FFmpeg init failed: {:?}", e))?;

        let codec = ffmpeg::encoder::find(ffmpeg::codec::Id::H264)
            .ok_or("H.264 encoder not found")?;

        let mut enc = ffmpeg::codec::context::Context::new_with_codec(codec)
            .encoder()
            .video()
            .map_err(|e| format!("Failed to create video encoder: {:?}", e))?;

        enc.set_width(w);
        enc.set_height(h);
        enc.set_time_base(ffmpeg::Rational::new(1, fps as i32));
        enc.set_frame_rate(Some(ffmpeg::Rational::new(fps as i32, 1)));
        enc.set_format(ffmpeg::format::Pixel::YUV420P);

        if bitrate > 0 {
            enc.set_bit_rate(bitrate as usize);
        }

        let opened = enc.open()
            .map_err(|e| format!("Failed to open encoder: {:?}", e))?;

        let mut output = ffmpeg::format::output(path)
            .map_err(|e| format!("Failed to create output file: {:?}", e))?;

        let mut stream = output.add_stream(opened.codec().ok_or("No codec")?)
            .map_err(|e| format!("Failed to add stream: {:?}", e))?;
        stream.set_time_base(ffmpeg::Rational::new(1, fps as i32));

        output.write_header()
            .map_err(|e| format!("Failed to write header: {:?}", e))?;

        Ok(Self { encoder: opened, output })
    }

    pub fn encode_frame(&mut self, frame: &ffmpeg::frame::Video) -> Result<(), String> {
        self.encoder.send_frame(frame)
            .map_err(|e| format!("Failed to send frame: {:?}", e))?;

        let mut packet = ffmpeg::Packet::empty();
        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(0);
            packet.write_interleaved(&mut self.output)
                .map_err(|e| format!("Failed to write packet: {:?}", e))?;
        }

        Ok(())
    }

    pub fn flush(mut self) -> Result<(), String> {
        self.encoder.send_eof()
            .map_err(|e| format!("Failed to flush encoder: {:?}", e))?;

        let mut packet = ffmpeg::Packet::empty();
        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(0);
            packet.write_interleaved(&mut self.output)
                .map_err(|e| format!("Failed to write packet: {:?}", e))?;
        }

        self.output.write_trailer()
            .map_err(|e| format!("Failed to write trailer: {:?}", e))?;

        Ok(())
    }
}
