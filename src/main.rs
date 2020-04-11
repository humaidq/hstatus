extern crate x11;
use std::ffi::CString;

pub struct DesktopStatus {
    disp: *mut x11::xlib::Display,
}

impl DesktopStatus {
    pub fn new() -> Self {
        DesktopStatus {disp: unsafe {
            x11::xlib::XOpenDisplay(std::ptr::null())
        }}
    }
    pub fn set_status(self, stat: &str) {
        unsafe {
            let s = CString::new(stat).expect("CString::new failed at set_status");
            x11::xlib::XStoreName(self.disp, x11::xlib::XDefaultRootWindow(self.disp), s.as_ptr());
        }
    }
}

impl Drop for DesktopStatus {
    fn drop(&mut self) {
        unsafe {
            x11::xlib::XCloseDisplay(self.disp);
        }
    }
}

fn main() {
    // Open display
    let status: DesktopStatus = DesktopStatus::new();
    status.set_status("hi");
}

