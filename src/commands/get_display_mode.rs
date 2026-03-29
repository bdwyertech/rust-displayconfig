use anyhow::{Context, Result};

use crate::monitor_panel::MPDisplayMgr;

pub fn get_display_mode(uuid: &str) -> Result<()> {
    unsafe {
        // Get the MonitorPanel manager
        let mgr = MPDisplayMgr::acquire().context("MonitorPanel manager not available")?;

        // Find the display with the matching UUID
        let mp_display = mgr
            .find_display_by_uuid(uuid)
            .with_context(|| format!("Display with UUID {} not found", uuid))?;

        // Get the current mode
        let current_mode = mp_display
            .current_mode()
            .with_context(|| format!("Could not retrieve current mode for display {}", uuid))?;

        println!("{}", current_mode.mode_number());
        Ok(())
    }
}
