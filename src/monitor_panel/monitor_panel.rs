// This provides access to detailed display mode information not available via Core Graphics

use std::ptr::NonNull;

use objc2::msg_send;
use objc2::runtime::{AnyClass, AnyObject};

/// Consistent display ID type used throughout the crate.
pub type DisplayID = u32;

// Safe wrappers for MonitorPanel API
#[allow(dead_code)]
pub struct MPDisplayMgr {
    obj: NonNull<AnyObject>,
    /// Whether this instance was obtained via `alloc`/`init` and therefore
    /// owns the Objective-C object (must be released on drop).
    owned: bool,
}

#[allow(dead_code)]
pub struct MPDisplay {
    obj: NonNull<AnyObject>,
}

#[allow(dead_code)]
pub struct MPDisplayMode {
    obj: NonNull<AnyObject>,
}

// MARK: - MPDisplayMgr

#[allow(dead_code)]
impl MPDisplayMgr {
    /// Create a new MPDisplayMgr instance (caller owns the object).
    pub unsafe fn new() -> Option<Self> {
        let cls = AnyClass::get(c"MPDisplayMgr")?;
        let obj: *mut AnyObject = unsafe { msg_send![cls, alloc] };
        let obj: *mut AnyObject = unsafe { msg_send![NonNull::new(obj)?, init] };
        Some(MPDisplayMgr {
            obj: NonNull::new(obj)?,
            owned: true,
        })
    }

    /// Get the shared MPDisplayMgr singleton (not owned — must NOT be released).
    pub unsafe fn shared() -> Option<Self> {
        let cls = AnyClass::get(c"MPDisplayMgr")?;
        let obj: *mut AnyObject = unsafe { msg_send![cls, sharedMgr] };
        Some(MPDisplayMgr {
            obj: NonNull::new(obj)?,
            owned: false,
        })
    }

    /// Convenience: try `new()` first, fall back to `shared()`.
    pub unsafe fn acquire() -> Option<Self> {
        unsafe { Self::new().or_else(|| Self::shared()) }
    }

    /// Get all displays
    pub unsafe fn displays(&self) -> Option<Vec<MPDisplay>> {
        let array: *mut AnyObject = unsafe { msg_send![self.obj.as_ref(), displays] };
        let array = NonNull::new(array)?;

        let count: usize = unsafe { msg_send![array.as_ref(), count] };
        let mut displays = Vec::with_capacity(count);

        for i in 0..count {
            let obj: *mut AnyObject = unsafe { msg_send![array.as_ref(), objectAtIndex: i] };
            if let Some(obj) = NonNull::new(obj) {
                displays.push(MPDisplay { obj });
            }
        }

        Some(displays)
    }

    /// Get display with specific ID
    pub unsafe fn display_with_id(&self, display_id: DisplayID) -> Option<MPDisplay> {
        let obj: *mut AnyObject =
            unsafe { msg_send![self.obj.as_ref(), displayWithID: display_id as i32] };
        Some(MPDisplay {
            obj: NonNull::new(obj)?,
        })
    }

    /// Find a display by its persistent UUID string.
    /// Returns `None` if no display matches.
    pub unsafe fn find_display_by_uuid(&self, uuid: &str) -> Option<MPDisplay> {
        let displays = unsafe { self.displays()? };
        for display in displays {
            if let Some(display_uuid) = unsafe { display.uuid() } {
                if display_uuid.eq_ignore_ascii_case(uuid) {
                    return Some(display);
                }
            }
        }
        None
    }

    /// Find a display by its contextual (Core Graphics) display ID.
    /// Returns `None` if no display matches.
    pub unsafe fn find_display_by_cg_id(&self, cg_id: DisplayID) -> Option<MPDisplay> {
        let displays = unsafe { self.displays()? };
        for display in displays {
            if unsafe { display.display_id() } == cg_id {
                return Some(display);
            }
        }
        None
    }
}

impl Drop for MPDisplayMgr {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                let _: () = msg_send![self.obj.as_ref(), release];
            }
        }
    }
}

// MARK: - MPDisplay

#[allow(dead_code)]
impl MPDisplay {
    /// Get all modes for this display
    pub unsafe fn all_modes(&self) -> Option<Vec<MPDisplayMode>> {
        let array: *mut AnyObject = unsafe { msg_send![self.obj.as_ref(), allModes] };
        let array = NonNull::new(array)?;

        let count: usize = unsafe { msg_send![array.as_ref(), count] };
        let mut modes = Vec::with_capacity(count);

        for i in 0..count {
            let obj: *mut AnyObject = unsafe { msg_send![array.as_ref(), objectAtIndex: i] };
            if let Some(obj) = NonNull::new(obj) {
                modes.push(MPDisplayMode { obj });
            }
        }

        Some(modes)
    }

    /// Get the display ID as a consistent DisplayID type.
    pub unsafe fn display_id(&self) -> DisplayID {
        let raw: i32 = unsafe { msg_send![self.obj.as_ref(), displayID] };
        raw as DisplayID
    }

    /// Get the display name
    pub unsafe fn display_name(&self) -> Option<String> {
        let name: *mut AnyObject = unsafe { msg_send![self.obj.as_ref(), displayName] };
        let name = NonNull::new(name)?;
        let cstr: *const i8 = unsafe { msg_send![name.as_ref(), UTF8String] };
        if cstr.is_null() {
            return None;
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(cstr) };
        Some(c_str.to_string_lossy().into_owned())
    }

    /// Check if display is HiDPI
    pub unsafe fn is_hidpi(&self) -> bool {
        unsafe { msg_send![self.obj.as_ref(), isHiDPI] }
    }

    /// Check if display is Retina
    pub unsafe fn is_retina(&self) -> bool {
        unsafe { msg_send![self.obj.as_ref(), isRetina] }
    }

    /// Get the persistent UUID for this display
    pub unsafe fn uuid(&self) -> Option<String> {
        let uuid: *mut AnyObject = unsafe { msg_send![self.obj.as_ref(), uuid] };
        let uuid = NonNull::new(uuid)?;
        let uuid_string: *mut AnyObject = unsafe { msg_send![uuid.as_ref(), UUIDString] };
        let uuid_string = NonNull::new(uuid_string)?;
        let cstr: *const i8 = unsafe { msg_send![uuid_string.as_ref(), UTF8String] };
        if cstr.is_null() {
            return None;
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(cstr) };
        Some(c_str.to_string_lossy().into_owned())
    }

    /// Get the current mode
    pub unsafe fn current_mode(&self) -> Option<MPDisplayMode> {
        let mode: *mut AnyObject = unsafe { msg_send![self.obj.as_ref(), currentMode] };
        Some(MPDisplayMode {
            obj: NonNull::new(mode)?,
        })
    }

    /// Set the display mode by mode number
    /// Returns the result code (0 = success)
    pub unsafe fn set_mode_number(&self, mode_number: i32) -> i32 {
        unsafe { msg_send![self.obj.as_ref(), setModeNumber: mode_number] }
    }
}

// MARK: - MPDisplayMode

#[allow(dead_code)]
impl MPDisplayMode {
    /// Get width
    pub unsafe fn width(&self) -> i32 {
        unsafe { msg_send![self.obj.as_ref(), width] }
    }

    /// Get height
    pub unsafe fn height(&self) -> i32 {
        unsafe { msg_send![self.obj.as_ref(), height] }
    }

    /// Get pixels wide
    pub unsafe fn pixels_wide(&self) -> i32 {
        unsafe { msg_send![self.obj.as_ref(), pixelsWide] }
    }

    /// Get pixels high
    pub unsafe fn pixels_high(&self) -> i32 {
        unsafe { msg_send![self.obj.as_ref(), pixelsHigh] }
    }

    /// Get refresh rate
    pub unsafe fn refresh_rate(&self) -> i32 {
        unsafe { msg_send![self.obj.as_ref(), refreshRate] }
    }

    /// Get scale factor
    pub unsafe fn scale(&self) -> f32 {
        unsafe { msg_send![self.obj.as_ref(), scale] }
    }

    /// Check if mode is HiDPI
    pub unsafe fn is_hidpi(&self) -> bool {
        unsafe { msg_send![self.obj.as_ref(), isHiDPI] }
    }

    /// Check if mode is Retina
    pub unsafe fn is_retina(&self) -> bool {
        unsafe { msg_send![self.obj.as_ref(), isRetina] }
    }

    /// Check if mode is native
    pub unsafe fn is_native_mode(&self) -> bool {
        unsafe { msg_send![self.obj.as_ref(), isNativeMode] }
    }

    /// Check if mode is default
    pub unsafe fn is_default_mode(&self) -> bool {
        unsafe { msg_send![self.obj.as_ref(), isDefaultMode] }
    }

    /// Check if mode is user visible
    pub unsafe fn is_user_visible(&self) -> bool {
        unsafe { msg_send![self.obj.as_ref(), isUserVisible] }
    }

    /// Get mode number
    pub unsafe fn mode_number(&self) -> i32 {
        unsafe { msg_send![self.obj.as_ref(), modeNumber] }
    }

    /// Return the underlying Objective-C object pointer as a usize.
    pub unsafe fn object_ptr(&self) -> usize {
        self.obj.as_ptr() as usize
    }

    /// Return the Objective-C `-description` for the mode, if available.
    /// This often contains more internal metadata useful for debugging.
    pub unsafe fn description(&self) -> Option<String> {
        let desc: *mut AnyObject = unsafe { msg_send![self.obj.as_ref(), description] };
        let desc = NonNull::new(desc)?;
        let cstr: *const i8 = unsafe { msg_send![desc.as_ref(), UTF8String] };
        if cstr.is_null() {
            return None;
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(cstr) };
        Some(c_str.to_string_lossy().into_owned())
    }
}
