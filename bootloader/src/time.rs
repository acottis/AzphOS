//! This crate gets the time from the CMOS on the motherboard, currently can
//! just just capture current time with pretty print
//! #TODO
//! * Epoch Time
use crate::cpu;

/// Stores the current time in its raw parts
pub struct DateTime {
    // 0x00
    sec: u8,
    // 0x02
    min: u8,
    // 0x04
    hour: u8,
    // 0x07
    day: u8,
    // 0x08
    month: u8,
    // 0x09
    year: u8,
    // 0x32
    centuary: u8,
}

impl DateTime {
    /// Captures the current time
    pub fn now() -> Self {
        Self {
            sec: cpu::rtc_register(0x00),      // Seconds
            min: cpu::rtc_register(0x02),      // Minutes
            hour: cpu::rtc_register(0x04),     // Hours
            day: cpu::rtc_register(0x07),      // Days
            month: cpu::rtc_register(0x08),    // Months
            year: cpu::rtc_register(0x09),     // Years
            centuary: cpu::rtc_register(0x32), // Centuaries
        }
    }
}
/// Implement display for DateTime
impl core::fmt::Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02X}{:02X}-{:02X}-{:02X} {:02X}:{:02X}:{:02X}",
            self.centuary,
            self.year,
            self.month,
            self.day,
            self.hour,
            self.min,
            self.sec,
        )
    }
}

// Bugged as it doesnt use Epoch time yet, hacked it by adding minutes
// pub fn sleep(seconds: usize) {
//     let dt = DateTime::now();
//     let start = dt.sec as usize + (dt.min as usize * 60) as usize;
//     //serial_print!("Starting to sleep... for {} seconds, Currently:
//     // {:X}",seconds, start);
//     while (DateTime::now().sec as usize
//         + ((DateTime::now().min as usize) * 60) as usize)
//         < (start + seconds)
//     {}
//     //serial_print!(" Awake now at: {:X}\n", DateTime::now().sec);
// }
