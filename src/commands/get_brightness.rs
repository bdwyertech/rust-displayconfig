use crate::monitor_panel::MPDisplayMgr;
use core_graphics::display::CGDisplay;

// Link to CoreDisplay framework for brightness functions
#[link(name = "CoreDisplay", kind = "framework")]
unsafe extern "C" {
    fn CoreDisplay_Display_GetUserBrightness(display: u32) -> f64;
}

pub fn get_brightness(filter_display: Option<u32>) {
    println!("=== Display Brightness Information ===\n");

    // Get list of active displays
    let displays = CGDisplay::active_displays().expect("Failed to get displays");

    // Filter displays if requested
    let display_ids: Vec<_> = if let Some(id) = filter_display {
        if displays.contains(&id) {
            vec![id]
        } else {
            eprintln!("Error: Display ID {} not found", id);
            std::process::exit(1);
        }
    } else {
        displays
    };

    println!("Found {} active display(s):\n", display_ids.len());

    for (idx, display_id) in display_ids.iter().enumerate() {
        let display = CGDisplay::new(*display_id);

        println!("Display {}:", idx + 1);
        println!("  Contextual screen id: {}", display_id);

        // Get persistent screen ID from MonitorPanel
        unsafe {
            if let Some(mgr) = MPDisplayMgr::new().or_else(|| MPDisplayMgr::shared()) {
                if let Some(mp_displays) = mgr.displays() {
                    for mp_display in mp_displays.iter() {
                        if mp_display.display_id() == *display_id as i32 {
                            if let Some(uuid) = mp_display.uuid() {
                                println!("  Persistent screen id: {}", uuid);
                            }
                            break;
                        }
                    }
                }
            }
        }

        println!("  Display Model: {}", display.model_number());
        println!("  Is built-in: {}", display.is_builtin());

        // Get current brightness using CoreDisplay
        // Note: This primarily works for built-in displays
        let brightness = unsafe { CoreDisplay_Display_GetUserBrightness(*display_id) };

        if brightness >= 0.0 && brightness <= 1.0 {
            // Convert to percentage (brightness is returned as 0.0-1.0)
            let percentage = (brightness * 100.0).round() as u32;
            println!("  Brightness: {}%", percentage);
        } else {
            // For external displays, brightness control may not be available
            println!("  Brightness: Not available (external display or unsupported)");
        }

        println!();
    }
}
