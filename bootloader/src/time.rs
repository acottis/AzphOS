//! This crate gets the time from the CMOS on the motherboard, currently can just just capture current time with
//! pretty print
use crate::cpu;

/// Stores the current time in its raw parts
/// 
pub struct DateTime{
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

impl DateTime{
    /// Captures the current time
    /// 
    pub fn now() -> Self {
        Self{
            sec: cpu::get_rtc_register(0x00), // Seconds
            min: cpu::get_rtc_register(0x02), // Minutes
            hour: cpu::get_rtc_register(0x04), // Hours
            day: cpu::get_rtc_register(0x07), // Days
            month: cpu::get_rtc_register(0x08), // Months
            year: cpu::get_rtc_register(0x09), // Years
            centuary: cpu::get_rtc_register(0x32), // Centuaries
        }
    }
}
/// Implement display for DateTime
impl core::fmt::Display for DateTime{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:X}{:X}-{:X}-{:X} {:X}:{:X}:{:X}", 
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

