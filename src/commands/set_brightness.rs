use crate::monitor_panel::MPDisplayMgr;
use core_graphics::display::CGDisplay;

// Link to CoreDisplay framework for brightness functions
#[link(name = "CoreDisplay", kind = "framework")]
unsafe extern "C" {
    fn CoreDisplay_Display_SetUserBrightness(display: u32, brightness: f64);
    fn CoreDisplay_Display_GetUserBrightness(display: u32) -> f64;
}

pub fn set_brightness(display_id: u32, brightness: u32) {
    // Validate brightness percentage
    if brightness > 100 {
        eprintln!("Error: Brightness must be between 0 and 100");
        std::process::exit(1);
    }

    // Get list of active displays
    let displays = CGDisplay::active_displays().expect("Failed to get displays");

    // Check if display exists
    if !displays.contains(&display_id) {
        eprintln!("Error: Display ID {} not found", display_id);
        std::process::exit(1);
    }

    let display = CGDisplay::new(display_id);

    println!("=== Setting Display Brightness ===\n");
    println!("Display ID: {}", display_id);

    // Get persistent screen ID from MonitorPanel for display info
    unsafe {
        if let Some(mgr) = MPDisplayMgr::new().or_else(|| MPDisplayMgr::shared()) {
            if let Some(mp_displays) = mgr.displays() {
                for mp_display in mp_displays.iter() {
                    if mp_display.display_id() == display_id as i32 {
                        if let Some(uuid) = mp_display.uuid() {
                            println!("Persistent screen id: {}", uuid);
                        }
                        break;
                    }
                }
            }
        }
    }

    println!("Display Model: {}", display.model_number());
    println!("Is built-in: {}", display.is_builtin());

    // Get current brightness before setting
    let current_brightness = unsafe { CoreDisplay_Display_GetUserBrightness(display_id) };
    if current_brightness >= 0.0 && current_brightness <= 1.0 {
        let current_percentage = (current_brightness * 100.0).round() as u32;
        println!("Current brightness: {}%", current_percentage);
    }

    // Convert percentage to 0.0-1.0 range
    let brightness_value = brightness as f64 / 100.0;

    // Set the brightness
    unsafe {
        CoreDisplay_Display_SetUserBrightness(display_id, brightness_value);
    }

    println!("New brightness: {}%", brightness);
    println!("\nBrightness updated successfully!");
}
