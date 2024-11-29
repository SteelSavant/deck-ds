use std::ptr;

use x11::xlib::Display;

use anyhow::{Context, Result};
use x11::xlib::XCloseDisplay;
use x11::xlib::XOpenDisplay;

// The main handle consists simply of a pointer to the display
type DisplayHandleSys = ptr::NonNull<Display>;
#[derive(Debug)]
pub struct XDisplayHandle {
    sys: DisplayHandleSys,
}

impl XDisplayHandle {
    pub fn open() -> Result<Self> {
        Ok(Self {
            sys: ptr::NonNull::new(unsafe { XOpenDisplay(ptr::null()) })
                .context("Failed to open XDisplay")?,
        })
    }

    pub fn as_ptr(&mut self) -> *mut Display {
        self.sys.as_ptr()
    }
}

impl Drop for XDisplayHandle {
    fn drop(&mut self) {
        unsafe { XCloseDisplay(self.sys.as_ptr()) };
    }
}
