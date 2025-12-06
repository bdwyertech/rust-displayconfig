// Rust FFI bindings for MonitorPanel.framework using objc crate
// This provides access to detailed display mode information not available via Core Graphics

use core_graphics::display::CGDirectDisplayID;
use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};

// Safe wrappers for MonitorPanel API
pub struct MPDisplayMgr {
    obj: *mut Object,
}

pub struct MPDisplay {
    obj: *mut Object,
}

pub struct MPDisplayMode {
    obj: *mut Object,
}

impl MPDisplayMgr {
    /// Create a new MPDisplayMgr instance
    pub unsafe fn new() -> Option<Self> {
        let cls = Class::get("MPDisplayMgr")?;
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, init];
        if obj.is_null() {
            None
        } else {
            Some(MPDisplayMgr { obj })
        }
    }

    /// Get the shared MPDisplayMgr instance
    pub unsafe fn shared() -> Option<Self> {
        let cls = Class::get("MPDisplayMgr")?;
        let obj: *mut Object = msg_send![cls, sharedMgr];
        if obj.is_null() {
            None
        } else {
            Some(MPDisplayMgr { obj })
        }
    }

    /// Get all displays
    pub unsafe fn displays(&self) -> Option<Vec<MPDisplay>> {
        let array: *mut Object = msg_send![self.obj, displays];
        if array.is_null() {
            return None;
        }

        let count: usize = msg_send![array, count];
        let mut displays = Vec::with_capacity(count);

        for i in 0..count {
            let display_obj: *mut Object = msg_send![array, objectAtIndex: i];
            if !display_obj.is_null() {
                displays.push(MPDisplay { obj: display_obj });
            }
        }

        Some(displays)
    }

    /// Get display with specific ID
    pub unsafe fn display_with_id(&self, display_id: CGDirectDisplayID) -> Option<MPDisplay> {
        let obj: *mut Object = msg_send![self.obj, displayWithID: display_id as i32];
        if obj.is_null() {
            None
        } else {
            Some(MPDisplay { obj })
        }
    }
}

impl MPDisplay {
    /// Get all modes for this display
    pub unsafe fn all_modes(&self) -> Option<Vec<MPDisplayMode>> {
        let array: *mut Object = msg_send![self.obj, allModes];
        if array.is_null() {
            return None;
        }

        let count: usize = msg_send![array, count];
        let mut modes = Vec::with_capacity(count);

        for i in 0..count {
            let mode_obj: *mut Object = msg_send![array, objectAtIndex: i];
            if !mode_obj.is_null() {
                modes.push(MPDisplayMode { obj: mode_obj });
            }
        }

        Some(modes)
    }

    /// Get the display ID
    pub unsafe fn display_id(&self) -> i32 {
        msg_send![self.obj, displayID]
    }

    /// Get the display name
    pub unsafe fn display_name(&self) -> Option<String> {
        let name: *mut Object = msg_send![self.obj, displayName];
        if name.is_null() {
            return None;
        }
        let cstr: *const i8 = msg_send![name, UTF8String];
        if cstr.is_null() {
            return None;
        }
        let c_str = std::ffi::CStr::from_ptr(cstr);
        Some(c_str.to_string_lossy().into_owned())
    }

    /// Check if display is HiDPI
    pub unsafe fn is_hidpi(&self) -> bool {
        let result: bool = msg_send![self.obj, isHiDPI];
        result
    }

    /// Check if display is Retina
    pub unsafe fn is_retina(&self) -> bool {
        let result: bool = msg_send![self.obj, isRetina];
        result
    }

    /// Get the persistent UUID for this display
    pub unsafe fn uuid(&self) -> Option<String> {
        let uuid: *mut Object = msg_send![self.obj, uuid];
        if uuid.is_null() {
            return None;
        }
        let uuid_string: *mut Object = msg_send![uuid, UUIDString];
        if uuid_string.is_null() {
            return None;
        }
        let cstr: *const i8 = msg_send![uuid_string, UTF8String];
        if cstr.is_null() {
            return None;
        }
        let c_str = std::ffi::CStr::from_ptr(cstr);
        Some(c_str.to_string_lossy().into_owned())
    }

    /// Get the current mode
    pub unsafe fn current_mode(&self) -> Option<MPDisplayMode> {
        let mode: *mut Object = msg_send![self.obj, currentMode];
        if mode.is_null() {
            None
        } else {
            Some(MPDisplayMode { obj: mode })
        }
    }

    /// Set the display mode by mode number
    /// Returns the result code (0 = success)
    pub unsafe fn set_mode_number(&self, mode_number: i32) -> i32 {
        msg_send![self.obj, setModeNumber: mode_number]
    }
}

impl MPDisplayMode {
    /// Get width
    pub unsafe fn width(&self) -> i32 {
        msg_send![self.obj, width]
    }

    /// Get height
    pub unsafe fn height(&self) -> i32 {
        msg_send![self.obj, height]
    }

    /// Get pixels wide
    pub unsafe fn pixels_wide(&self) -> i32 {
        msg_send![self.obj, pixelsWide]
    }

    /// Get pixels high
    pub unsafe fn pixels_high(&self) -> i32 {
        msg_send![self.obj, pixelsHigh]
    }

    /// Get refresh rate
    pub unsafe fn refresh_rate(&self) -> i32 {
        msg_send![self.obj, refreshRate]
    }

    /// Get scale factor
    pub unsafe fn scale(&self) -> f32 {
        msg_send![self.obj, scale]
    }

    /// Check if mode is HiDPI
    pub unsafe fn is_hidpi(&self) -> bool {
        let result: bool = msg_send![self.obj, isHiDPI];
        result
    }

    /// Check if mode is Retina
    pub unsafe fn is_retina(&self) -> bool {
        let result: bool = msg_send![self.obj, isRetina];
        result
    }

    /// Check if mode is native
    pub unsafe fn is_native_mode(&self) -> bool {
        let result: bool = msg_send![self.obj, isNativeMode];
        result
    }

    /// Check if mode is default
    pub unsafe fn is_default_mode(&self) -> bool {
        let result: bool = msg_send![self.obj, isDefaultMode];
        result
    }

    /// Check if mode is user visible
    pub unsafe fn is_user_visible(&self) -> bool {
        let result: bool = msg_send![self.obj, isUserVisible];
        result
    }

    /// Get mode number
    pub unsafe fn mode_number(&self) -> i32 {
        msg_send![self.obj, modeNumber]
    }
}
