extern crate chrono;
extern crate libc;
extern crate x11;
extern crate reqwest;
extern crate select;
extern crate cached;

use cached::proc_macro::cached;

use select::document::Document;
use select::predicate::{Class, Predicate};

use chrono::prelude::*;
use libc::{c_int, getloadavg};
use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;
use std::{string, thread, time};

const SYSTEM_NAME: &str = "humaid's system";

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

type StatusItem = fn() -> String;

fn load_item() -> String {
    match get_load() {
        Ok(load) => return format!("L:{}|", load.as_str()),
        Err(why) => println!("Cannot get load: {}", why),
    }
    "".to_string()
}

fn battery_item() -> String {
    match get_battery_with_status() {
        Ok(perc) => return format!("B:{}|", perc.as_str()),
        Err(why) => println!("Cannot get battery percentage: {}", why),
    }
    "".to_string()
}

fn time_item() -> String {
    let mut res = String::new();
    let local: DateTime<Local> = Local::now();
    res.push_str("UK:");
    res.push_str(
        local
            //.with_timezone(&chrono::FixedOffset::east(3600))
            .format("%I:%M")
            .to_string()
            .as_str(),
    );
    res.push(' ');
    res.push_str("AE:");
    res.push_str(local
        .with_timezone(&chrono::FixedOffset::east(4 * 3600))
        .format("%I:%M %p %d-%m-%Y").to_string().as_str());
    res.push('|');
    res
}

#[cached(time=21600)]
fn get_covid_stats() -> String {
    let resp = reqwest::blocking::get("http://covid19.ncema.gov.ae/").unwrap();
    if !resp.status().is_success() {
        println!("covid fail: {}", resp.status());
        return "fail".to_string();
    }
    let doc = Document::from_read(resp).unwrap();

    let active = doc
        .select(Class("active").descendant(Class("counter"))).next().unwrap()
        .text();
    
    let recovered = doc
        .select(Class("recovered").descendant(Class("new-cases").descendant(Class("recovered")))).next().unwrap()
        .text();

    let rec = recovered.split(' ').collect::<Vec<&str>>()[2].to_string();

    return format!("A:{} R:{}|", active, rec);
}

fn main() {
    let mut stat_items: Vec<StatusItem> = Vec::new();
    stat_items.push(get_covid_stats);
    stat_items.push(load_item);
    stat_items.push(battery_item);
    stat_items.push(time_item);

    let status: DesktopStatus = DesktopStatus::new();
    loop {
        println!("Updating status");
        // Run the low battery flair
        let bat_num = get_battery_perc();
        if bat_num < 20 {
            let st_res = read_file("/sys/class/power_supply/BAT0/status");
            if let Ok(s) = st_res {
                if s.trim() == "Discharging" {
                    let mut bat_notice = String::new();
                    bat_notice.push_str("==============================");
                    bat_notice.push_str(" !!! Low Battery !!! (");
                    bat_notice.push_str(bat_num.to_string().as_str());
                    bat_notice.push_str("%) ==============================");

                    for i in 0..4 {
                        if i % 2 == 0 {
                            status.set_status(bat_notice.as_str());
                        } else {
                            status.set_status("hey!");
                        }
                        thread::sleep(time::Duration::from_secs(1));
                    }
                }
            }
        }

        let mut stat = String::new();

        for i in &stat_items {
            stat.push_str(i().as_str());
        }

        stat.push_str(SYSTEM_NAME);
        status.set_status(stat.as_str());
        thread::sleep(time::Duration::from_secs(3));
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
    match unsafe { getloadavg(avgs.as_mut_ptr(), 3 as c_int) } {
        -1 => Err("returned -1"),
        3 => {
            unsafe {
                avgs.set_len(3_usize);
            }
            Ok(format!("{:.2} {:.2} {:.2}", avgs[0], avgs[1], avgs[2]))
        }
        _ => Err("unknown value"),
    }
}

fn get_battery_perc() -> i32 {
    let present = read_file("/sys/class/power_supply/BAT0/present").unwrap();
    assert_eq!(present, "1", "Battery not present");

    let full:i32 = read_file("/sys/class/power_supply/BAT0/energy_full_design")
        .unwrap().parse().unwrap();
    let now: i32 = read_file("/sys/class/power_supply/BAT0/energy_now")
        .unwrap().parse().unwrap();

    return ((now as f64 / full as f64) * 100_f64) as i32
}

fn get_battery_with_status() -> std::io::Result<string::String> {
    let status = read_file("/sys/class/power_supply/BAT0/status")?;
    let stat = match status.as_ref() {
        "Discharging" => "-",
        "Charging" => "+",
        _ => "/",
    };

    Ok(format!("{}%{}", get_battery_perc(), stat))
}
