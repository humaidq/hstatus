extern crate x11;
use chrono::prelude::*;
use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;
use std::{string, thread, time};
use libc::{getloadavg, c_int};

#[derive(Debug, Clone, Copy)]
pub struct DesktopStatus {
    disp: *mut x11::xlib::Display,
}

impl DesktopStatus {
    pub fn new() -> Self {
        DesktopStatus {
            disp: unsafe { x11::xlib::XOpenDisplay(std::ptr::null()) },
        }
    }
    pub fn set_status(self, stat: &str) {
        unsafe {
            let s = CString::new(stat).expect("CString::new failed at set_status");
            x11::xlib::XStoreName(
                self.disp,
                x11::xlib::XDefaultRootWindow(self.disp),
                s.as_ptr(),
            );
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
        // load
        let load_res = get_load();
        match load_res {
            Ok(load) => {
                stat.push_str("L:");
                stat.push_str(load.as_str());
                stat.push('|');
            }
            Err(why) => println!("Cannot get load: {}", why),
        }

        // bat
        let bat = get_battery();
        match bat {
            Ok(perc) => {
                stat.push_str("B:");
                stat.push_str(perc.as_str());
                stat.push('|');
            }
            Err(why) => println!("Cannot get battery percentage: {}", why),
        }
        // time
        let local: DateTime<Local> = Local::now();
        stat.push_str("UK:");
        stat.push_str(
            local
                .with_timezone(&chrono::FixedOffset::east(3600))
                .format("%I:%M")
                .to_string()
                .as_str(),
        );
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

fn read_file(file: &str) -> std::io::Result<string::String> {
    let mut file = File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.replace("\n", ""))
}

fn get_load() -> Result<string::String, &'static str> {
    let mut avgs = Vec::with_capacity(3_usize);
    match unsafe {
        getloadavg(avgs.as_mut_ptr(), 3 as c_int)
    } {
        -1 => Err("returned -1"),
        3 => {
            unsafe {
                avgs.set_len(3_usize);
            }
            Ok(format!("{} {} {}", avgs[0], avgs[1], avgs[2]))
        },
        _ => Err("unknown value"),
    }
}

fn get_battery() -> std::io::Result<string::String> {
    let present = read_file("/sys/class/power_supply/BAT0/present")?;
    assert_eq!(present, "1", "Battery not present");

    let full = read_file("/sys/class/power_supply/BAT0/energy_full_design")?;
    let full_design: i32 = full.parse().unwrap();

    let now = read_file("/sys/class/power_supply/BAT0/energy_now")?;
    let now_cap: i32 = now.parse().unwrap();

    let status = read_file("/sys/class/power_supply/BAT0/status")?;
    let stat = match status.as_ref() {
        "Discharging" => "-",
        "Charging" => "+",
        _ => "/",
    };

    Ok(format!(
        "{}%{}",
        ((now_cap as f64 / full_design as f64) * 100_f64) as i32,
        stat
    ))
}
