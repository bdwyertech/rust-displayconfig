fn main() {
    // Only compile on macOS
    let target = std::env::var("TARGET").unwrap();
    if !target.contains("apple") {
        panic!("This only works on macOS");
    }

    // Link MonitorPanel framework (private framework in /System/Library/PrivateFrameworks)
    println!("cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks");
    println!("cargo:rustc-link-lib=framework=MonitorPanel");
}
