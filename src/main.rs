// Display management utility for macOS
// Uses Core Graphics and MonitorPanel.framework APIs

use clap::{Parser, Subcommand};

mod monitor_panel;

mod commands;
use crate::commands::{get_display_mode, list_displays, set_display_mode};

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
