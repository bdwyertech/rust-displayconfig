use anyhow::{bail, Context, Result};

use crate::monitor_panel::MPDisplayMgr;

pub fn set_display_mode(uuid: &str, mode_number: i32) -> Result<()> {
    println!("=== Setting Display Mode ===\n");

    unsafe {
        // Get the MonitorPanel manager
        let mgr = MPDisplayMgr::acquire().context("MonitorPanel manager not available")?;

        // Find the display with the matching UUID
        let mp_display = mgr
            .find_display_by_uuid(uuid)
            .with_context(|| format!("Display with UUID {} not found", uuid))?;

        let display_id = mp_display.display_id();

        // Verify the mode exists
        let modes = mp_display
            .all_modes()
            .with_context(|| format!("Could not retrieve modes for display {}", uuid))?;

        if !modes.iter().any(|m| m.mode_number() == mode_number) {
            bail!(
                "Mode #{} not found for display {}. Use 'list --verbose' to see available modes.",
                mode_number,
                uuid
            );
        }

        // Set the mode
        println!(
            "Setting display {} (ID: {}) to mode #{}...",
            uuid, display_id, mode_number
        );
        let result = mp_display.set_mode_number(mode_number);

        if result == 0 {
            println!("✓ Successfully set display mode");
        } else {
            bail!("Failed to set display mode (error code: {})", result);
        }
    }

    Ok(())
}
