//! Calendar dates in UTC, written `YYYY-MM-DD`. A file expires when its `expires` day ends, UTC.

use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    /// A real calendar date between 2000-01-01 and 9999-12-31.
    pub fn new(year: u16, month: u8, day: u8) -> Option<Date> {
        if !(2000..=9999).contains(&year) || !(1..=12).contains(&month) {
            return None;
        }
        if day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Date { year, month, day })
    }

    /// Reads exactly `YYYY-MM-DD`: ten ASCII characters, digits and two hyphens.
    pub fn parse(s: &str) -> Option<Date> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let num = |from: usize, to: usize| -> Option<u16> {
            let mut n: u16 = 0;
            for &c in &b[from..to] {
                if !c.is_ascii_digit() {
                    return None;
                }
                n = n * 10 + u16::from(c - b'0');
            }
            Some(n)
        };
        Date::new(num(0, 4)?, num(5, 7)? as u8, num(8, 10)? as u8)
    }

    /// Days since 1970-01-01.
    pub fn days_since_epoch(self) -> i64 {
        days_from_civil(i64::from(self.year), i64::from(self.month), i64::from(self.day))
    }

    pub fn from_days_since_epoch(days: i64) -> Option<Date> {
        let (y, m, d) = civil_from_days(days);
        if !(2000..=9999).contains(&y) {
            return None;
        }
        Date::new(y as u16, m as u8, d as u8)
    }

    /// The UTC date that contains this Unix time (seconds since 1970-01-01 00:00 UTC).
    pub fn from_unix(secs: i64) -> Option<Date> {
        Date::from_days_since_epoch(secs.div_euclid(86_400))
    }

    /// The first second after this day, UTC. A file whose `expires` is this date is expired from then on.
    pub fn end_unix(self) -> i64 {
        (self.days_since_epoch() + 1) * 86_400
    }

    pub fn add_days(self, n: i64) -> Option<Date> {
        Date::from_days_since_epoch(self.days_since_epoch() + n)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn is_leap(y: u16) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: u16, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if is_leap(y) => 29,
        _ => 28,
    }
}

// Howard Hinnant's day-count algorithms (public domain): http://howardhinnant.github.io/date_algorithms.html
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}
