use anyhow::{bail, Result};
use core_graphics::display::CGDisplay;

use crate::monitor_panel::{DisplayID, MPDisplayMgr};

// Link to CoreDisplay framework for brightness functions
#[link(name = "CoreDisplay", kind = "framework")]
unsafe extern "C" {
    fn CoreDisplay_Display_SetUserBrightness(display: u32, brightness: f64);
    fn CoreDisplay_Display_GetUserBrightness(display: u32) -> f64;
}

pub fn set_brightness(display_id: DisplayID, brightness: u32) -> Result<()> {
    // Validate brightness percentage
    if brightness > 100 {
        bail!("Brightness must be between 0 and 100");
    }

    // Get list of active displays
    // Get list of active displays
    let displays = CGDisplay::active_displays().map_err(|e| anyhow::anyhow!("Failed to get displays (CG error: {})", e))?;

    // Check if display exists
    if !displays.contains(&display_id) {
        bail!("Display ID {} not found", display_id);
    }

    let display = CGDisplay::new(display_id);

    println!("=== Setting Display Brightness ===\n");
    println!("Display ID: {}", display_id);

    // Get persistent screen ID from MonitorPanel for display info
    unsafe {
        if let Some(mgr) = MPDisplayMgr::acquire() {
            if let Some(mp_display) = mgr.find_display_by_cg_id(display_id) {
                if let Some(uuid) = mp_display.uuid() {
                    println!("Persistent screen id: {}", uuid);
                }
            }
        }
    }

    println!("Display Model: {}", display.model_number());
    println!("Is built-in: {}", display.is_builtin());

    // Get current brightness before setting
    let current_brightness = unsafe { CoreDisplay_Display_GetUserBrightness(display_id) };
    if (0.0..=1.0).contains(&current_brightness) {
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

    Ok(())
}
