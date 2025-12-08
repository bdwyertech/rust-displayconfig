// Display management utility for macOS
// Uses Core Graphics and MonitorPanel.framework APIs

use clap::{Parser, Subcommand};

mod monitor_panel;

mod commands;
use crate::commands::{
    get_brightness, get_display_mode, list_displays, set_brightness, set_display_mode, watch,
};

#[derive(Parser)]
#[command(name = "displayconfig")]
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
    /// Get the current brightness percentage for displays
    GetBrightness {
        /// Filter by display ID
        #[arg(short, long)]
        display: Option<u32>,
    },
    /// Set the brightness percentage for a specific display
    SetBrightness {
        /// Display ID to configure
        #[arg(short, long)]
        display: u32,

        /// Brightness percentage (0-100)
        #[arg(short, long)]
        brightness: u32,
    },

    /// Watch for display configuration changes and print events.
    Watch {},
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
        Commands::GetBrightness { display } => {
            get_brightness(*display);
        }
        Commands::SetBrightness {
            display,
            brightness,
        } => {
            set_brightness(*display, *brightness);
        }
        Commands::Watch {} => {
            watch();
        }
    }
}
