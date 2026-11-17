fn main() {
    println!("cargo:warning=build.rs starting");
    
    #[cfg(target_os = "macos")]
    {
        println!("cargo:warning=Compiling Swift module...");
        
        // Compile Swift capture module
        let status = std::process::Command::new("swiftc")
            .args(&[
                "-parse-as-library",
                "-c",
                "src/capture_macos.swift",
                "-o",
                "target/capture_macos.o",
                "-framework",
                "ScreenCaptureKit",
                "-framework",
                "Foundation",
                "-framework",
                "CoreGraphics",
                "-framework",
                "CoreMedia",
                "-framework",
                "CoreVideo",
            ])
            .status()
            .expect("Failed to compile Swift module");

        if !status.success() {
            panic!("Swift compilation failed");
        }

        println!("cargo:warning=Swift compilation successful");

        // Create static library
        let status = std::process::Command::new("ar")
            .args(&["rcs", "target/libcapture_macos.a", "target/capture_macos.o"])
            .status()
            .expect("Failed to create static library");

        if !status.success() {
            panic!("Static library creation failed");
        }

        println!("cargo:warning=Static library created");

        // Link the static library
        println!("cargo:rustc-link-search=native=target");
        println!("cargo:rustc-link-lib=static=capture_macos");
        println!("cargo:warning=Link flags set");
    }
    
    println!("cargo:warning=build.rs finished");
}