fn main() {
    // Only compile on macOS
    let target = std::env::var("TARGET").unwrap();
    if !target.contains("apple") {
        panic!("This demo only works on macOS");
    }

    // Link CoreGraphics framework
    println!("cargo:rustc-link-lib=framework=CoreGraphics");

    // Link Foundation framework (required for Objective-C runtime)
    println!("cargo:rustc-link-lib=framework=Foundation");

    // Link MonitorPanel framework (private framework in /System/Library/PrivateFrameworks)
    println!("cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks");
    println!("cargo:rustc-link-lib=framework=MonitorPanel");
}
