use ffmpeg_next as ffmpeg;

fn main() {
    ffmpeg::init().unwrap();
    
    // Try to list available protocols
    let mut p = std::ptr::null_mut();
    unsafe {
        // Use avio_enum_protocols to list protocols
        while let Some(name) = ffmpeg::ffi::avio_enum_protocols(&mut p, 0) {
            let s = std::ffi::CStr::from_ptr(name);
            println!("Protocol: {}", s.to_string_lossy());
        }
    }
}
