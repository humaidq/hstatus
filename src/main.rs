extern crate x11;
use std::ffi::CString;
use chrono::prelude::*;
use std::{thread, time, string};
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Clone, Copy)]
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
            x11::xlib::XSync(self.disp, 0);
        }
    }
    pub fn close(self) {
        unsafe {
            x11::xlib::XCloseDisplay(self.disp);
        }
    }

}


fn main() {
    let status: DesktopStatus = DesktopStatus::new();
    loop {
        println!("Update");
        let mut stat = String::new();
        // bat
        let bat = get_battery();
        match bat {
            Some(perc) => {
                stat.push_str("B:");
                stat.push_str(perc.as_str());
            }
            None => println!("Cannot get battery percentage")
        }
        // time
        let local: DateTime<Local> = Local::now();
        stat.push_str("UK:");
        stat.push_str(local.with_timezone(&chrono::FixedOffset::east(3600)).format("%I:%M").to_string().as_str());
        stat.push(' ');
        stat.push_str("AE:");
        stat.push_str(local.format("%I:%M %p %d-%m-%Y").to_string().as_str());
        stat.push('|');


        stat.push_str("humaid's system");
        status.set_status(stat.as_str());
        thread::sleep(time::Duration::from_secs(1));
    }
    status.close();
}

fn get_battery() -> Option<string::String> {
    let mut present = File::open("/sys/class/power_supply/BAT0/present").expect("something");
    let mut present_contents = String::new();
    present.read_to_string(&mut present_contents).expect("something");
    assert_eq!(present_contents, "1\n", "Battery not present");

    let mut full = File::open("/sys/class/power_supply/BAT0/energy_full_design").expect("something");
    let mut full_contents = String::new();
    full.read_to_string(&mut full_contents).expect("something");

    let full_design: i32 = full_contents.replace("\n","").parse().unwrap();

    let mut now = File::open("/sys/class/power_supply/BAT0/energy_now").expect("something");
    let mut now_contents = String::new();
    now.read_to_string(&mut now_contents).expect("something");

    let now_cap: i32 = now_contents.replace("\n","").parse().unwrap();

    let mut status = File::open("/sys/class/power_supply/BAT0/status").expect("something");
    let mut status_contents = String::new();
    status.read_to_string(&mut status_contents).expect("something");
    let stat = match status_contents.replace("\n","").as_ref() {
        "Discharging" =>  "-",
        "Charging" =>"+",
        _ =>"/",
    };

    Some(format!("{}%{}",((now_cap as f64/full_design as f64)*100_f64) as i32, stat))
}
