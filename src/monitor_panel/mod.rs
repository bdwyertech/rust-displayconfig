mod monitor_panel;

// Re-export public types from the monitor_panel module
pub use monitor_panel::MPDisplayMgr;

// These types are used internally but can be accessed via monitor_panel:: if needed
#[allow(unused_imports)]
pub use monitor_panel::{MPDisplay, MPDisplayMode};
