use std::collections::{HashMap, HashSet};

use anyhow::{bail, Result};
use core_graphics::display::CGDisplay;

use crate::monitor_panel::{DisplayID, MPDisplayMgr, MPDisplayMode};

pub fn list_displays(verbose: bool, filter_display: Option<DisplayID>) -> Result<()> {
    println!("=== Display Information ===\n");

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
            list_display_modes(display_id);
        } else {
            println!("  Use --verbose to see all available display modes");
        }

        println!();
    }

    Ok(())
}

// MARK: - Mode listing (decomposed)

#[derive(Debug)]
struct ModeEntry {
    mode_num: i32,
    width: i32,
    height: i32,
    pixels_wide: i32,
    pixels_high: i32,
    refresh: i32,
    scale: f32,
    is_hidpi: bool,
    is_retina: bool,
    is_native: bool,
    is_default: bool,
    depth: i32,
}

/// Collect user-visible modes from an MPDisplay into lightweight entries.
unsafe fn collect_visible_modes(modes: &[MPDisplayMode]) -> Vec<ModeEntry> {
    let mut entries = Vec::new();

    for mode in modes {
        if !unsafe { mode.is_user_visible() } {
            continue;
        }

        let depth = parse_depth_from_description(&unsafe { mode.description() });

        entries.push(ModeEntry {
            mode_num: unsafe { mode.mode_number() },
            width: unsafe { mode.width() },
            height: unsafe { mode.height() },
            pixels_wide: unsafe { mode.pixels_wide() },
            pixels_high: unsafe { mode.pixels_high() },
            refresh: unsafe { mode.refresh_rate() },
            scale: unsafe { mode.scale() },
            is_hidpi: unsafe { mode.is_hidpi() },
            is_retina: unsafe { mode.is_retina() },
            is_native: unsafe { mode.is_native_mode() },
            is_default: unsafe { mode.is_default_mode() },
            depth,
        });
    }

    entries
}

/// Parse "depth = N" from an ObjC description string.
fn parse_depth_from_description(desc: &Option<String>) -> i32 {
    let Some(d) = desc else { return 0 };
    let Some(idx) = d.find("depth = ") else {
        return 0;
    };
    let sub = &d[idx + 8..];
    let end = sub
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(sub.len());
    sub[..end].trim().parse::<i32>().unwrap_or(0)
}

/// Determine which mode numbers should be the canonical "Default" per logical-size group.
///
/// Build a lightweight list of visible modes with metadata so we can choose a single
/// canonical "Default" per logical size group (prefer MonitorPanel current_mode(),
/// otherwise prefer the mode with the highest depth parsed from the mode description).
/// This avoids showing multiple Default flags for identical logical sizes that differ
/// only by pixel format/depth.
fn compute_canonical_defaults(
    entries: &[ModeEntry],
    mp_current_mode_num: Option<i32>,
) -> HashSet<i32> {
    // Group entries by logical size + scale
    let mut groups: HashMap<(i32, i32, i32), Vec<&ModeEntry>> = HashMap::new();
    for e in entries {
        // use width, height, and scale (as i32 scaled by 100)
        let scale_key = (e.scale * 100.0).round() as i32;
        groups.entry((e.width, e.height, scale_key)).or_default().push(e);
    }

    let mut canonical = HashSet::new();
    for candidates in groups.values() {
        // collect candidates that report is_default == true
        let defaults: Vec<&&ModeEntry> = candidates.iter().filter(|e| e.is_default).collect();
        if defaults.is_empty() {
            continue;
        }

        // If MP exposes current mode, prefer it when present
        if let Some(mp_num) = mp_current_mode_num {
            if let Some(found) = defaults.iter().find(|e| e.mode_num == mp_num) {
                canonical.insert(found.mode_num);
                continue;
            }
        }

        // Otherwise prefer highest depth
        let best = defaults.iter().max_by_key(|e| e.depth).unwrap();
        canonical.insert(best.mode_num);
    }

    canonical
}

/// Check if a mode entry matches the current display mode.
/// Prefer MonitorPanel's current_mode() if available, otherwise fallback to CG.
fn is_current_mode(
    entry: &ModeEntry,
    mp_current_mode_num: Option<i32>,
    cg_current_mode: &Option<core_graphics::display::CGDisplayMode>,
) -> bool {
    if let Some(mp_num) = mp_current_mode_num {
        return mp_num == entry.mode_num;
    }

    if let Some(cg_mode) = cg_current_mode {
        let cg_w = cg_mode.width() as f64;
        let cg_h = cg_mode.height() as f64;
        let cg_refresh = cg_mode.refresh_rate();

        let refresh_match = (cg_refresh - (entry.refresh as f64)).abs() < 1.0;
        let expected_pw = (cg_w * (entry.scale as f64)).round();
        let expected_ph = (cg_h * (entry.scale as f64)).round();
        let pixels_match = (expected_pw - entry.pixels_wide as f64).abs() < 1.0
            && (expected_ph - entry.pixels_high as f64).abs() < 1.0;
        let logical_match =
            (cg_w - entry.width as f64).abs() < 0.1 && (cg_h - entry.height as f64).abs() < 0.1;

        return logical_match && refresh_match && pixels_match;
    }

    false
}

/// Format a single mode entry into a display string.
fn format_mode_entry(entry: &ModeEntry, canonical_defaults: &HashSet<i32>, is_current: bool) -> String {
    let pixel_suffix = if entry.pixels_wide != entry.width || entry.pixels_high != entry.height {
        format!(" ({}x{} pixels)", entry.pixels_wide, entry.pixels_high)
    } else {
        String::new()
    };

    let scale_suffix = if entry.scale != 1.0 {
        format!(" scale={:.1}x", entry.scale)
    } else {
        String::new()
    };

    let mut flags = Vec::new();
    if entry.is_hidpi {
        flags.push("HiDPI");
    }
    if entry.is_retina {
        flags.push("Retina");
    }
    if entry.is_native {
        flags.push("Native");
    }
    if entry.is_default && canonical_defaults.contains(&entry.mode_num) {
        flags.push("Default");
    }
    if is_current {
        flags.push("Current");
    }

    let flags_str = if flags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", flags.join(", "))
    };

    format!(
        "Mode #{}: {}x{}{} @ {}Hz{}{}",
        entry.mode_num, entry.width, entry.height, pixel_suffix, entry.refresh, scale_suffix, flags_str
    )
}

fn list_display_modes(display_id: DisplayID) {
    println!("  Available modes:");
    unsafe {
        // Get CGDisplay's current mode so we can mark the corresponding
        // MonitorPanel mode as current when listing.
        let cg_current_mode = CGDisplay::new(display_id).display_mode();

        // Try both new() and shared() methods
        let Some(mgr) = MPDisplayMgr::acquire() else {
            println!("    (MonitorPanel manager not available)");
            return;
        };
        let Some(mp_display) = mgr.find_display_by_cg_id(display_id) else {
            println!("    (display ID {} not found in MonitorPanel)", display_id);
            return;
        };
        let Some(modes) = mp_display.all_modes() else {
            println!("    (no modes available for this display)");
            return;
        };

        println!("    Found {} total modes\n", modes.len());

        let mp_current_mode_num = mp_display.current_mode().map(|m| m.mode_number());
        let entries = collect_visible_modes(&modes);
        let canonical_defaults = compute_canonical_defaults(&entries, mp_current_mode_num);

        let mut hidpi_modes: Vec<(i32, String)> = Vec::new();
        let mut standard_modes: Vec<(i32, String)> = Vec::new();

        for entry in &entries {
            let current = is_current_mode(entry, mp_current_mode_num, &cg_current_mode);
            let line = format_mode_entry(entry, &canonical_defaults, current);

            if entry.is_hidpi || entry.is_retina {
                hidpi_modes.push((entry.mode_num, line));
            } else {
                standard_modes.push((entry.mode_num, line));
            }
        }

        // Sort modes by mode number
        hidpi_modes.sort_by_key(|(n, _)| *n);
        standard_modes.sort_by_key(|(n, _)| *n);

        // Display HiDPI modes first
        if !hidpi_modes.is_empty() {
            println!("    HiDPI/Retina Modes:");
            for (_, info) in &hidpi_modes {
                println!("      {}", info);
            }
            println!();
        }

        // Then standard modes
        if !standard_modes.is_empty() {
            println!("    Standard Modes:");
            for (_, info) in &standard_modes {
                println!("      {}", info);
            }
        }
    }
}
