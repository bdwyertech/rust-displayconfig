use core_graphics::display::CGDisplay;

use crate::monitor_panel::MPDisplayMgr;

pub fn list_displays(verbose: bool, filter_display: Option<u32>) {
    println!("=== Display Information ===\n");

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
        println!("  Width: {} pixels", display.pixels_wide());
        println!("  Height: {} pixels", display.pixels_high());
        println!("  Is main: {}", display.is_main());
        println!("  Is built-in: {}", display.is_builtin());

        // Get current display mode
        if let Some(mode) = display.display_mode() {
            println!("  Current mode:");
            println!("    Width: {}", mode.width());
            println!("    Height: {}", mode.height());
            println!("    Refresh rate: {:.2} Hz", mode.refresh_rate());
        }

        // List all available display modes using MonitorPanel framework
        if verbose {
            list_display_modes(*display_id);
        } else {
            println!("  Use --verbose to see all available display modes");
        }

        println!();
    }
}

fn list_display_modes(display_id: u32) {
    println!("  Available modes:");
    unsafe {
        // Get CGDisplay's current mode so we can mark the corresponding
        // MonitorPanel mode as current when listing.
        let cg_current_mode = CGDisplay::new(display_id).display_mode();

        // Try both new() and shared() methods
        let mgr = MPDisplayMgr::new().or_else(|| MPDisplayMgr::shared());

        if let Some(mgr) = mgr {
            // Try to get displays array
            if let Some(mp_displays) = mgr.displays() {
                // Try to find our display by iterating through all displays
                let mut found = false;
                for mp_display in mp_displays.iter() {
                    let mp_id = mp_display.display_id();
                    if mp_id == display_id as i32 {
                        found = true;

                        if let Some(modes) = mp_display.all_modes() {
                            println!("    Found {} total modes\n", modes.len());

                            // NOTE: Removed verbose description/pointer diagnostics
                            // to keep mode listing concise per user request.

                            // Try to get MonitorPanel's authoritative current mode number.
                            // If present, prefer this exact mode_number as the single
                            // authoritative current entry. Otherwise fall back to
                            // the CG-based heuristic used previously.
                            let mp_current_mode_num: Option<i32> = mp_display
                                .current_mode()
                                .and_then(|m| Some(m.mode_number()));

                            // Separate HiDPI and non-HiDPI modes with their mode numbers for sorting
                            let mut hidpi_modes: Vec<(i32, String)> = Vec::new();
                            let mut standard_modes: Vec<(i32, String)> = Vec::new();

                            for mode in modes.iter() {
                                let width = mode.width();
                                let height = mode.height();
                                let pixels_wide = mode.pixels_wide();
                                let pixels_high = mode.pixels_high();
                                let refresh = mode.refresh_rate();
                                let scale = mode.scale();
                                let mode_num = mode.mode_number();
                                let is_hidpi = mode.is_hidpi();
                                let is_retina = mode.is_retina();
                                let is_native = mode.is_native_mode();
                                let is_default = mode.is_default_mode();
                                let is_visible = mode.is_user_visible();

                                // Only show user-visible modes
                                if is_visible {
                                    let mut mode_info = format!(
                                        "Mode #{}: {}x{}{} @ {}Hz{}{}",
                                        mode_num,
                                        width,
                                        height,
                                        if pixels_wide != width || pixels_high != height {
                                            format!(" ({}x{} pixels)", pixels_wide, pixels_high)
                                        } else {
                                            String::new()
                                        },
                                        refresh,
                                        if scale != 1.0 {
                                            format!(" scale={:.1}x", scale)
                                        } else {
                                            String::new()
                                        },
                                        {
                                            let mut flags = Vec::new();
                                            if is_hidpi {
                                                flags.push("HiDPI");
                                            }
                                            if is_retina {
                                                flags.push("Retina");
                                            }
                                            if is_native {
                                                flags.push("Native");
                                            }
                                            if is_default {
                                                flags.push("Default");
                                            }
                                            if !flags.is_empty() {
                                                format!(" [{}]", flags.join(", "))
                                            } else {
                                                String::new()
                                            }
                                        }
                                    );

                                    // Decide current-mode marking. If MonitorPanel
                                    // exposes a `currentMode`, prefer its `modeNumber`
                                    // as the authoritative single current mode. If not
                                    // available, fall back to the CG-derived heuristic
                                    // used previously (logical size + refresh + pixels).
                                    let mut is_current = false;

                                    if let Some(mp_num) = mp_current_mode_num {
                                        if mp_num == mode_num {
                                            is_current = true;
                                        }
                                    } else if let Some(cg_mode) = &cg_current_mode {
                                        // Use floating-point comparisons to tolerate minor
                                        // differences in reported refresh rates and scaling.
                                        let cg_w_f = cg_mode.width() as f64;
                                        let cg_h_f = cg_mode.height() as f64;
                                        let cg_refresh = cg_mode.refresh_rate();

                                        let refresh_match =
                                            (cg_refresh - (refresh as f64)).abs() < 1.0;

                                        // Compute whether the MonitorPanel mode's pixel
                                        // dimensions equal the CG mode's logical dims
                                        // multiplied by the mode's scale. This narrows
                                        // down HiDPI duplicate entries.
                                        let expected_pixels_w = (cg_w_f * (scale as f64)).round();
                                        let expected_pixels_h = (cg_h_f * (scale as f64)).round();
                                        let pixels_match =
                                            (expected_pixels_w - (pixels_wide as f64)).abs() < 1.0
                                                && (expected_pixels_h - (pixels_high as f64)).abs()
                                                    < 1.0;

                                        // Basic logical size match
                                        let logical_size_match = (cg_w_f - (width as f64)).abs()
                                            < 0.1
                                            && (cg_h_f - (height as f64)).abs() < 0.1;

                                        if logical_size_match && refresh_match && pixels_match {
                                            is_current = true;
                                        }
                                    }

                                    if is_current {
                                        mode_info.push_str(" [Current]");
                                    }

                                    // No extra diagnostics appended here (keeps output concise)

                                    if is_hidpi || is_retina {
                                        hidpi_modes.push((mode_num, mode_info));
                                    } else {
                                        standard_modes.push((mode_num, mode_info));
                                    }
                                }
                            }

                            // Sort modes by mode number
                            hidpi_modes.sort_by_key(|(mode_num, _)| *mode_num);
                            standard_modes.sort_by_key(|(mode_num, _)| *mode_num);

                            // Display HiDPI modes first
                            if !hidpi_modes.is_empty() {
                                println!("    HiDPI/Retina Modes:");
                                for (_, mode_info) in hidpi_modes {
                                    println!("      {}", mode_info);
                                }
                                println!();
                            }

                            // Then standard modes
                            if !standard_modes.is_empty() {
                                println!("    Standard Modes:");
                                for (_, mode_info) in standard_modes {
                                    println!("      {}", mode_info);
                                }
                            }
                        } else {
                            println!("    (no modes available for this display)");
                        }
                        break;
                    }
                }

                if !found {
                    println!("    (display ID {} not found in MonitorPanel)", display_id);
                }
            } else {
                println!("    (no displays array available from manager)");
            }
        } else {
            println!("    (MonitorPanel manager not available)");
        }
    }
}
