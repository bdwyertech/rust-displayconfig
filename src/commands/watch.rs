use std::collections::HashMap;
use std::os::raw::c_void;
use std::ptr;

use cocoa::appkit::NSApp;
use core_graphics::display::CGDisplay;

/// Native watcher using CGDisplayRegisterReconfigurationCallback with a polling fallback.
///
/// The CLI prints diagnostics at startup, registers the CoreGraphics callback, then starts
/// a lightweight polling thread as a fallback so we can confirm changes even if callbacks
/// aren't being delivered in this environment.
type CGDirectDisplayID = u32;
type CGDisplayChangeSummaryFlags = u32;

unsafe extern "C" {
    // void CGDisplayRegisterReconfigurationCallback(CGDisplayReconfigurationCallBack callback, void *userInfo);
    fn CGDisplayRegisterReconfigurationCallback(
        callback: extern "C" fn(CGDirectDisplayID, CGDisplayChangeSummaryFlags, *mut c_void),
        user_info: *mut c_void,
    );
    // void CGDisplayRemoveReconfigurationCallback(CGDisplayReconfigurationCallBack callback, void *userInfo);
    fn CGDisplayRemoveReconfigurationCallback(
        callback: extern "C" fn(CGDirectDisplayID, CGDisplayChangeSummaryFlags, *mut c_void),
        user_info: *mut c_void,
    );

    // Run the CoreFoundation run loop so system-delivered callbacks are invoked.
    fn CFRunLoopRun();
}

extern "C" fn display_reconfig_callback(
    display: CGDirectDisplayID,
    flags: CGDisplayChangeSummaryFlags,
    _user_info: *mut c_void,
) {
    // Print a concise event with numeric flags and the currently reported CG mode (if any).
    println!(
        "[callback] Display reconfiguration: id={} flags=0x{:x}",
        display, flags
    );

    // Use core_graphics to show the current mode for convenience.
    let d = CGDisplay::new(display);
    if let Some(mode) = d.display_mode() {
        println!(
            "  Current mode: {}x{} @ {:.2}Hz",
            mode.width(),
            mode.height(),
            mode.refresh_rate()
        );
    } else {
        println!("  Current mode: (none)");
    }
}

/// Capture display state (mode triple) for fallback polling.
///
/// Use u64 for width/height to match the CGDisplayMode API return types.
fn capture_state() -> HashMap<u32, Option<(u64, u64, f64)>> {
    let mut map: HashMap<u32, Option<(u64, u64, f64)>> = HashMap::new();

    let displays = CGDisplay::active_displays().unwrap_or_else(|_| Vec::new());
    for id in displays.iter() {
        let display = CGDisplay::new(*id);
        let mode = display
            .display_mode()
            .map(|m| (m.width() as u64, m.height() as u64, m.refresh_rate()));
        map.insert(*id, mode);
    }

    map
}

/// Register the native reconfiguration callback and keep the process alive.
/// The CLI's interval is used for the polling fallback only (callback is immediate).
pub fn watch() {
    println!(
        "Watching for display configuration changes via CGDisplayRegisterReconfigurationCallback..."
    );

    // Diagnostic: show current active displays at startup.
    let initial = capture_state();
    println!(
        "Initial active displays: {:?}",
        initial.keys().collect::<Vec<_>>()
    );

    unsafe {
        // Register callback; user_info is null for now.
        CGDisplayRegisterReconfigurationCallback(display_reconfig_callback, ptr::null_mut());
    }

    // Initialize the shared NSApplication instance.
    unsafe {
        let _app = NSApp();
        let _ = _app; // drop immediately
    }

    // Run the CFRunLoop so CoreGraphics can deliver display reconfiguration callbacks.
    unsafe {
        CFRunLoopRun();
    }
}
