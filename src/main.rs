// Display management utility for macOS
// Uses Core Graphics and MonitorPanel.framework APIs

use clap::{Parser, Subcommand};
use core_graphics::display::CGDisplay;

mod monitor_panel;
use monitor_panel::MPDisplayMgr;

#[derive(Parser)]
#[command(name = "displayplacer")]
#[command(author, version, about = "Display management utility for macOS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all displays and their available modes
    List {
        /// Show verbose output with all mode details
        #[arg(short, long)]
        verbose: bool,

        /// Filter by display ID
        #[arg(short, long)]
        display: Option<u32>,
    },
    /// Get the current mode number for a specific display
    GetMode {
        /// Persistent screen ID (UUID) to query
        #[arg(short, long)]
        display: String,
    },
    /// Set the display mode for a specific display
    SetMode {
        /// Persistent screen ID (UUID) to configure
        #[arg(short, long)]
        display: String,

        /// Mode number to set
        #[arg(short, long)]
        mode: i32,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List { verbose, display } => {
            list_displays(*verbose, *display);
        }
        Commands::GetMode { display } => {
            get_display_mode(display);
        }
        Commands::SetMode { display, mode } => {
            set_display_mode(display, *mode);
        }
    }
}

fn list_displays(verbose: bool, filter_display: Option<u32>) {
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
            println!("    Mode ID: {}", mode.mode_id());
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
    println!("  Available modes (via MonitorPanel):");
    unsafe {
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
                                    let mode_info = format!(
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

fn get_display_mode(uuid: &str) {
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

fn set_display_mode(uuid: &str, mode_number: i32) {
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
