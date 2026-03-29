// MonitorPanel framework bindings
// This provides access to detailed display mode information not available via Core Graphics

mod monitor_panel;

// Re-export public types from the monitor_panel module
#[allow(unused_imports)]
pub use monitor_panel::{DisplayID, MPDisplay, MPDisplayMgr, MPDisplayMode};
