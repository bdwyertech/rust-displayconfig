use std::collections::HashMap;
use std::os::raw::c_void;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use core_graphics::display::CGDisplay;

use crate::monitor_panel::DisplayID;

/// Native watcher using CGDisplayRegisterReconfigurationCallback.
///
/// The CLI prints diagnostics at startup, registers the CoreGraphics callback, then
/// runs the CFRunLoop to receive display reconfiguration events. Ctrl+C triggers
/// clean shutdown with callback deregistration.

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
    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopStop(rl: *mut c_void);
    fn CFRunLoopRun();
}

static SHOULD_STOP: AtomicBool = AtomicBool::new(false);

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
fn capture_state() -> HashMap<DisplayID, Option<(u64, u64, f64)>> {
    let mut map = HashMap::new();
    let displays = CGDisplay::active_displays().unwrap_or_default();
    for &id in &displays {
        let display = CGDisplay::new(id);
        let mode = display
            .display_mode()
            .map(|m| (m.width() as u64, m.height() as u64, m.refresh_rate()));
        map.insert(id, mode);
    }
    map
}

/// Register the native reconfiguration callback and keep the process alive.
pub fn watch() -> Result<()> {
    println!(
        "Watching for display configuration changes (Ctrl+C to stop)..."
    );

    // Diagnostic: show current active displays at startup.
    let initial = capture_state();
    println!(
        "Initial active displays: {:?}",
        initial.keys().collect::<Vec<_>>()
    );

    unsafe {
        CGDisplayRegisterReconfigurationCallback(display_reconfig_callback, ptr::null_mut());
    }

    // Handle SIGINT/SIGTERM for clean shutdown
    ctrlc::set_handler(move || {
        SHOULD_STOP.store(true, Ordering::SeqCst);
        // Stop the CFRunLoop so the main thread unblocks
        unsafe {
            let rl = CFRunLoopGetCurrent();
            if !rl.is_null() {
                CFRunLoopStop(rl);
            }
        }
    })
    .context("Failed to set Ctrl+C handler")?;

    // Block on the CFRunLoop so CoreGraphics delivers callbacks
    unsafe {
        CFRunLoopRun();
    }

    // Cleanup: deregister the callback
    unsafe {
        CGDisplayRemoveReconfigurationCallback(display_reconfig_callback, ptr::null_mut());
    }

    println!("\nStopped watching.");
    Ok(())
}
