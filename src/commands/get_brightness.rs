use anyhow::{bail, Result};
use core_graphics::display::CGDisplay;

use crate::monitor_panel::{DisplayID, MPDisplayMgr};

// Link to CoreDisplay framework for brightness functions
#[link(name = "CoreDisplay", kind = "framework")]
unsafe extern "C" {
    fn CoreDisplay_Display_GetUserBrightness(display: u32) -> f64;
}

pub fn get_brightness(filter_display: Option<DisplayID>) -> Result<()> {
    println!("=== Display Brightness Information ===\n");

    // Get list of active displays
    let displays = CGDisplay::active_displays().map_err(|e| anyhow::anyhow!("Failed to get displays (CG error: {})", e))?;

    // Filter displays if requested
    let display_ids: Vec<DisplayID> = if let Some(id) = filter_display {
        if displays.contains(&id) {
            vec![id]
        } else {
            bail!("Display ID {} not found", id);
        }
    } else {
        displays
    };

    println!("Found {} active display(s):\n", display_ids.len());

    for (idx, &display_id) in display_ids.iter().enumerate() {
        let display = CGDisplay::new(display_id);

        println!("Display {}:", idx + 1);
        println!("  Contextual screen id: {}", display_id);

        // Get persistent screen ID from MonitorPanel
        unsafe {
            if let Some(mgr) = MPDisplayMgr::acquire() {
                if let Some(mp_display) = mgr.find_display_by_cg_id(display_id) {
                    if let Some(uuid) = mp_display.uuid() {
                        println!("  Persistent screen id: {}", uuid);
                    }
                }
            }
        }

        println!("  Display Model: {}", display.model_number());
        println!("  Is built-in: {}", display.is_builtin());

        // Get current brightness using CoreDisplay
        // Note: This primarily works for built-in displays
        let brightness = unsafe { CoreDisplay_Display_GetUserBrightness(display_id) };

        if (0.0..=1.0).contains(&brightness) {
            // Convert to percentage (brightness is returned as 0.0-1.0)
            let percentage = (brightness * 100.0).round() as u32;
            println!("  Brightness: {}%", percentage);
        } else {
            // For external displays, brightness control may not be available
            println!("  Brightness: Not available (external display or unsupported)");
        }

        println!();
    }

    Ok(())
}
