extern crate x11;
use std::ffi::CString;

fn main() {
    let disp: *mut x11::xlib::Display;
    // Open display
    disp = unsafe {
        x11::xlib::XOpenDisplay(std::ptr::null())
    };

    unsafe {
        let s = CString::new("data data data data").expect("CString::new failed");
        x11::xlib::XStoreName(disp, x11::xlib::XDefaultRootWindow(disp), s.as_ptr());
    }
    
    unsafe {
        x11::xlib::XCloseDisplay(disp);
    }
}

