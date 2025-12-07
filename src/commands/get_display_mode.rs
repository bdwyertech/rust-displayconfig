use crate::monitor_panel::MPDisplayMgr;

pub fn get_display_mode(uuid: &str) {
    unsafe {
        // Get the MonitorPanel manager
        let mgr = MPDisplayMgr::new().or_else(|| MPDisplayMgr::shared());

        if let Some(mgr) = mgr {
            if let Some(mp_displays) = mgr.displays() {
                // Find the display with the matching UUID
                for mp_display in mp_displays.iter() {
                    if let Some(display_uuid) = mp_display.uuid() {
                        if display_uuid.eq_ignore_ascii_case(uuid) {
                            // Get the current mode
                            if let Some(current_mode) = mp_display.current_mode() {
                                let mode_number = current_mode.mode_number();
                                println!("{}", mode_number);
                                return;
                            } else {
                                eprintln!("Error: Could not retrieve current mode for display with UUID {}", uuid);
                                std::process::exit(1);
                            }
                        }
                    }
                }

                eprintln!("Error: Display with UUID {} not found", uuid);
                eprintln!("Use 'list' to see available displays and their UUIDs");
                std::process::exit(1);
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
