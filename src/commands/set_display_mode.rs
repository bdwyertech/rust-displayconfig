use crate::monitor_panel::MPDisplayMgr;

pub fn set_display_mode(uuid: &str, mode_number: i32) {
    println!("=== Setting Display Mode ===\n");

    unsafe {
        // Get the MonitorPanel manager
        let mgr = MPDisplayMgr::new().or_else(|| MPDisplayMgr::shared());

        if let Some(mgr) = mgr {
            if let Some(mp_displays) = mgr.displays() {
                // Find the display with the matching UUID
                let mut found = false;
                for mp_display in mp_displays.iter() {
                    if let Some(display_uuid) = mp_display.uuid() {
                        if display_uuid.eq_ignore_ascii_case(uuid) {
                            found = true;
                            let display_id = mp_display.display_id();

                            // Verify the mode exists
                            if let Some(modes) = mp_display.all_modes() {
                                let mode_exists =
                                    modes.iter().any(|m| m.mode_number() == mode_number);

                                if !mode_exists {
                                    eprintln!(
                                        "Error: Mode #{} not found for display with UUID {}",
                                        mode_number, uuid
                                    );
                                    eprintln!(
                                        "Use 'list --verbose' to see available modes for this display"
                                    );
                                    std::process::exit(1);
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
                                    eprintln!(
                                        "✗ Failed to set display mode (error code: {})",
                                        result
                                    );
                                    std::process::exit(1);
                                }
                            } else {
                                eprintln!(
                                    "Error: Could not retrieve modes for display with UUID {}",
                                    uuid
                                );
                                std::process::exit(1);
                            }
                            break;
                        }
                    }
                }

                if !found {
                    eprintln!("Error: Display with UUID {} not found", uuid);
                    eprintln!("Use 'list' to see available displays and their UUIDs");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: Could not get displays from MonitorPanel");
                std::process::exit(1);
            }
        } else {
            eprintln!("Error: MonitorPanel manager not available");
            std::process::exit(1);
        }
    }
}
