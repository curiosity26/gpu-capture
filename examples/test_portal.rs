use gpu_capture::capture_portal;

fn main() {
    println!("Creating screencast session...");
    match capture_portal::create_screencast_session() {
        Ok(session_path) => {
            println!("Session created: {}", session_path);
            println!("Starting screencast...");
            match capture_portal::start_screencast(&session_path) {
                Ok(_) => println!("Screencast started!"),
                Err(e) => println!("Failed to start screencast: {}", e),
            }
        }
        Err(e) => println!("Failed to create session: {}", e),
    }
}
