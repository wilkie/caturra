//! `java.time` values: the calendar arithmetic behind `LocalDate` and the
//! two enums it answers with.
//!
//! Everything here is PURE — the proleptic Gregorian calendar, no clock, no
//! locale and no timezone database — which is what makes it exactly
//! comparable with a JDK. The parts of `java.time` that need a zone (a
//! `ZonedDateTime`, or what "today" is) ask the HOST, because the browser
//! already has the IANA database and vendoring a second copy would only add a
//! version to disagree with.

use std::fmt::Write as _;

/// A `java.time.LocalDate`: a date on the proleptic Gregorian calendar, with
/// no time and no zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    /// 1..=12.
    pub month: u8,
    /// 1..=31, valid for the month.
    pub day: u8,
}

/// The year range `LocalDate` accepts (JLS-independent; `java.time`'s own).
const MIN_YEAR: i32 = -999_999_999;
const MAX_YEAR: i32 = 999_999_999;

const MONTH_NAMES: [&str; 12] = [
    "JANUARY",
    "FEBRUARY",
    "MARCH",
    "APRIL",
    "MAY",
    "JUNE",
    "JULY",
    "AUGUST",
    "SEPTEMBER",
    "OCTOBER",
    "NOVEMBER",
    "DECEMBER",
];

const DAY_NAMES: [&str; 7] = [
    "MONDAY",
    "TUESDAY",
    "WEDNESDAY",
    "THURSDAY",
    "FRIDAY",
    "SATURDAY",
    "SUNDAY",
];

/// The name of month `1..=12`, as `Month.toString()` gives it.
#[must_use]
pub fn month_name(month: u8) -> &'static str {
    MONTH_NAMES[usize::from(month.clamp(1, 12)) - 1]
}

/// The name of day-of-week `1..=7` (Monday is 1), as `DayOfWeek.toString()`.
#[must_use]
pub fn day_name(day: u8) -> &'static str {
    DAY_NAMES[usize::from(day.clamp(1, 7)) - 1]
}

#[must_use]
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[must_use]
pub fn length_of_month(year: i32, month: u8) -> u8 {
    match month {
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Date {
    /// `LocalDate.of(year, month, day)`, with `java.time`'s own messages for
    /// each way it can be wrong. `Err` is the `DateTimeException`'s text.
    pub fn of(year: i32, month: i32, day: i32) -> Result<Self, String> {
        if !(MIN_YEAR..=MAX_YEAR).contains(&year) {
            return Err(format!(
                "Invalid value for Year (valid values -999999999 - 999999999): {year}"
            ));
        }
        if !(1..=12).contains(&month) {
            return Err(format!(
                "Invalid value for MonthOfYear (valid values 1 - 12): {month}"
            ));
        }
        if !(1..=31).contains(&day) {
            return Err(format!(
                "Invalid value for DayOfMonth (valid values 1 - 28/31): {day}"
            ));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (month, day) = (month as u8, day as u8);
        let length = length_of_month(year, month);
        if day > length {
            // The two spellings are the JDK's, not a slip: the leap-year case
            // says "February 29" in title case and every other says
            // "FEBRUARY 30" in upper. Only February can reach the first, which
            // is why the month is written out there.
            return if month == 2 && day == 29 {
                Err(format!(
                    "Invalid date 'February 29' as '{year}' is not a leap year"
                ))
            } else {
                Err(format!("Invalid date '{} {day}'", month_name(month)))
            };
        }
        Ok(Self { year, month, day })
    }

    /// Days since 1970-01-01. The civil-from-days algorithm, which is exact
    /// over the whole proleptic Gregorian range.
    #[must_use]
    pub fn to_epoch_day(self) -> i64 {
        let year = i64::from(self.year);
        let month = i64::from(self.month);
        let day = i64::from(self.day);
        let mut total = 365 * year;
        if year >= 0 {
            total += (year + 3) / 4 - (year + 99) / 100 + (year + 399) / 400;
        } else {
            total -= year / -4 - year / -100 + year / -400;
        }
        total += (367 * month - 362) / 12;
        total += day - 1;
        if month > 2 {
            total -= 1;
            if !is_leap_year(self.year) {
                total -= 1;
            }
        }
        total - 719_528
    }

    /// The inverse: `LocalDate.ofEpochDay`.
    #[must_use]
    pub fn from_epoch_day(epoch_day: i64) -> Self {
        let mut zero_day = epoch_day + 719_528 - 60;
        let mut adjust = 0i64;
        if zero_day < 0 {
            let cycles = (zero_day + 1) / 146_097 - 1;
            adjust = cycles * 400;
            zero_day += -cycles * 146_097;
        }
        let mut year_estimate = (400 * zero_day + 591) / 146_097;
        let mut day_estimate = zero_day
            - (365 * year_estimate + year_estimate / 4 - year_estimate / 100 + year_estimate / 400);
        if day_estimate < 0 {
            year_estimate -= 1;
            day_estimate = zero_day
                - (365 * year_estimate + year_estimate / 4 - year_estimate / 100
                    + year_estimate / 400);
        }
        year_estimate += adjust;
        let march_doy = day_estimate;
        let march_month = (march_doy * 5 + 2) / 153;
        let month = (march_month + 2) % 12 + 1;
        let day = march_doy - (march_month * 306 + 5) / 10 + 1;
        year_estimate += march_month / 10;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Self {
            year: year_estimate as i32,
            month: month as u8,
            day: day as u8,
        }
    }

    #[must_use]
    pub fn plus_days(self, days: i64) -> Self {
        Self::from_epoch_day(self.to_epoch_day().saturating_add(days))
    }

    /// `plusMonths`, with `java.time`'s clamping: the day is kept if the new
    /// month has it and pulled back to that month's last day if not, so
    /// `2024-01-31 plus one month` is `2024-02-29`.
    #[must_use]
    pub fn plus_months(self, months: i64) -> Self {
        let total = i64::from(self.year) * 12 + i64::from(self.month - 1) + months;
        let year = total.div_euclid(12);
        #[allow(clippy::cast_possible_truncation)]
        let month = (total.rem_euclid(12) + 1) as u8;
        let year = year.clamp(i64::from(MIN_YEAR), i64::from(MAX_YEAR));
        #[allow(clippy::cast_possible_truncation)]
        let year = year as i32;
        Self {
            year,
            month,
            day: self.day.min(length_of_month(year, month)),
        }
    }

    #[must_use]
    pub fn plus_years(self, years: i64) -> Self {
        self.plus_months(years.saturating_mul(12))
    }

    /// 1 = Monday, matching `DayOfWeek.getValue()`.
    #[must_use]
    pub fn day_of_week(self) -> u8 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let index = (self.to_epoch_day() + 3).rem_euclid(7) as u8;
        index + 1
    }

    #[must_use]
    pub fn day_of_year(self) -> i32 {
        let start = Self {
            year: self.year,
            month: 1,
            day: 1,
        };
        #[allow(clippy::cast_possible_truncation)]
        let days = (self.to_epoch_day() - start.to_epoch_day() + 1) as i32;
        days
    }

    #[must_use]
    pub fn length_of_year(self) -> i32 {
        if is_leap_year(self.year) { 366 } else { 365 }
    }

    /// `withDayOfMonth` / `withMonth` / `withYear` all validate the way `of`
    /// does, and say the same things when they fail.
    pub fn with(self, year: i32, month: i32, day: i32) -> Result<Self, String> {
        Self::of(year, month, day)
    }
}

impl std::fmt::Display for Date {
    /// ISO-8601, with `java.time`'s year rule: four digits, a `+` above 9999
    /// and a `-` below zero.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let year = self.year;
        if (0..=9999).contains(&year) {
            write!(f, "{year:04}")?;
        } else if year > 9999 {
            write!(f, "+{year}")?;
        } else if year > -10000 {
            write!(f, "-{:04}", -year)?;
        } else {
            write!(f, "{year}")?;
        }
        write!(f, "-{:02}-{:02}", self.month, self.day)
    }
}

/// `LocalDate.parse` — the ISO form only (`2024-01-15`), which is what
/// `toString` writes. The message is the JDK's `DateTimeParseException` text,
/// including the index it blames.
pub fn parse_date(text: &str) -> Result<Date, String> {
    let invalid = |index: usize| format!("Text '{text}' could not be parsed at index {index}");
    let bytes = text.as_bytes();
    // An optional sign, then at least four year digits.
    let (negative, start) = match bytes.first() {
        Some(b'-') => (true, 1),
        Some(b'+') => (false, 1),
        _ => (false, 0),
    };
    let mut at = start;
    while at < bytes.len() && bytes[at].is_ascii_digit() {
        at += 1;
    }
    if at - start < 4 {
        return Err(invalid(0));
    }
    let year: i64 = text[start..at].parse().map_err(|_| invalid(0))?;
    let year = if negative { -year } else { year };
    let field = |at: &mut usize| -> Result<i32, String> {
        if bytes.get(*at) != Some(&b'-') {
            return Err(invalid(*at));
        }
        *at += 1;
        let from = *at;
        while *at < bytes.len() && bytes[*at].is_ascii_digit() {
            *at += 1;
        }
        if *at - from != 2 {
            return Err(invalid(from));
        }
        text[from..*at].parse().map_err(|_| invalid(from))
    };
    let month = field(&mut at)?;
    let day = field(&mut at)?;
    if at != bytes.len() {
        return Err(invalid(at));
    }
    #[allow(clippy::cast_possible_truncation)]
    Date::of(year as i32, month, day)
}

#[cfg(test)]
mod tests {
    use super::{Date, parse_date};

    /// Every answer here was read off a real JDK 11 (`java.time.LocalDate`),
    /// not derived: the point of the table is that it is not our arithmetic
    /// checking our arithmetic.
    #[test]
    fn matches_the_jdk() {
        let d = Date::of(2024, 1, 15).expect("valid");
        assert_eq!(d.to_string(), "2024-01-15");
        assert_eq!(d.to_epoch_day(), 19737);
        assert_eq!(d.day_of_week(), 1); // MONDAY
        assert_eq!(d.day_of_year(), 15);
        assert_eq!(d.plus_days(30).to_string(), "2024-02-14");
        assert_eq!(d.plus_months(1).to_string(), "2024-02-15");
        assert_eq!(d.plus_years(1).to_string(), "2025-01-15");
        assert_eq!(d.plus_days(-21).to_string(), "2023-12-25");
        // The clamping rule, both ways round.
        assert_eq!(
            Date::of(2024, 1, 31)
                .expect("valid")
                .plus_months(1)
                .to_string(),
            "2024-02-29"
        );
        assert_eq!(
            Date::of(2024, 2, 29)
                .expect("valid")
                .plus_years(1)
                .to_string(),
            "2025-02-28"
        );
        assert_eq!(
            Date::of(2023, 3, 31)
                .expect("valid")
                .plus_months(-1)
                .to_string(),
            "2023-02-28"
        );
        assert_eq!(
            Date::of(2024, 12, 31)
                .expect("valid")
                .plus_days(1)
                .to_string(),
            "2025-01-01"
        );
        assert_eq!(Date::from_epoch_day(0).to_string(), "1970-01-01");
        assert_eq!(Date::from_epoch_day(-1).to_string(), "1969-12-31");
        assert_eq!(Date::from_epoch_day(19737), d);
        // The year is four digits, with a sign outside 0..=9999.
        assert_eq!(Date::of(1, 1, 1).expect("valid").to_string(), "0001-01-01");
        assert_eq!(
            Date::of(-1, 1, 1).expect("valid").to_string(),
            "-0001-01-01"
        );
        assert_eq!(
            Date::of(10000, 1, 1).expect("valid").to_string(),
            "+10000-01-01"
        );
        assert_eq!(Date::of(0, 1, 1).expect("valid").to_string(), "0000-01-01");
    }

    #[test]
    fn says_what_the_jdk_says_when_it_is_wrong() {
        assert_eq!(
            Date::of(2024, 2, 30).unwrap_err(),
            "Invalid date 'FEBRUARY 30'"
        );
        assert_eq!(
            Date::of(2024, 13, 1).unwrap_err(),
            "Invalid value for MonthOfYear (valid values 1 - 12): 13"
        );
        assert_eq!(
            Date::of(2024, 1, 0).unwrap_err(),
            "Invalid value for DayOfMonth (valid values 1 - 28/31): 0"
        );
        assert_eq!(
            Date::of(2023, 2, 29).unwrap_err(),
            "Invalid date 'February 29' as '2023' is not a leap year"
        );
        assert_eq!(
            parse_date("15/01/2024").unwrap_err(),
            "Text '15/01/2024' could not be parsed at index 0"
        );
        assert_eq!(
            parse_date("2024-01-15").expect("valid").to_string(),
            "2024-01-15"
        );
    }

    /// Round-tripping every day over four centuries — the leap-year rule, the
    /// 400-year cycle and the negative side of the epoch all at once.
    #[test]
    fn epoch_day_round_trips() {
        let mut day = Date::of(1800, 1, 1).expect("valid").to_epoch_day();
        let end = Date::of(2200, 1, 1).expect("valid").to_epoch_day();
        while day < end {
            let date = Date::from_epoch_day(day);
            assert_eq!(date.to_epoch_day(), day, "{date}");
            assert_eq!(
                Date::of(date.year, i32::from(date.month), i32::from(date.day)),
                Ok(date)
            );
            day += 1;
        }
    }
}

/// A `java.time.LocalTime`: a time of day with no date and no zone, held the
/// way `java.time` holds it — as nanoseconds since midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub nano_of_day: i64,
}

pub const NANOS_PER_SECOND: i64 = 1_000_000_000;
pub const NANOS_PER_MINUTE: i64 = 60 * NANOS_PER_SECOND;
pub const NANOS_PER_HOUR: i64 = 60 * NANOS_PER_MINUTE;
pub const NANOS_PER_DAY: i64 = 24 * NANOS_PER_HOUR;

impl Time {
    /// `LocalTime.of(...)`, with `java.time`'s message for each field.
    pub fn of(hour: i32, minute: i32, second: i32, nano: i32) -> Result<Self, String> {
        for (value, name, high) in [
            (hour, "HourOfDay", 23),
            (minute, "MinuteOfHour", 59),
            (second, "SecondOfMinute", 59),
            (nano, "NanoOfSecond", 999_999_999),
        ] {
            if !(0..=high).contains(&value) {
                return Err(format!(
                    "Invalid value for {name} (valid values 0 - {high}): {value}"
                ));
            }
        }
        Ok(Self {
            nano_of_day: i64::from(hour) * NANOS_PER_HOUR
                + i64::from(minute) * NANOS_PER_MINUTE
                + i64::from(second) * NANOS_PER_SECOND
                + i64::from(nano),
        })
    }

    /// Wrapping to the day, which is what `plusHours` and friends do.
    #[must_use]
    pub fn plus_nanos(self, nanos: i64) -> Self {
        Self {
            nano_of_day: (self.nano_of_day + nanos.rem_euclid(NANOS_PER_DAY))
                .rem_euclid(NANOS_PER_DAY),
        }
    }

    /// How many whole days `plus_nanos` would carry — what a `LocalDateTime`
    /// needs when its time crosses midnight.
    #[must_use]
    pub fn overflow_days(self, nanos: i64) -> i64 {
        (self.nano_of_day + nanos).div_euclid(NANOS_PER_DAY)
    }

    #[must_use]
    pub fn hour(self) -> i32 {
        #[allow(clippy::cast_possible_truncation)]
        let hour = (self.nano_of_day / NANOS_PER_HOUR) as i32;
        hour
    }

    #[must_use]
    pub fn minute(self) -> i32 {
        #[allow(clippy::cast_possible_truncation)]
        let minute = (self.nano_of_day / NANOS_PER_MINUTE % 60) as i32;
        minute
    }

    #[must_use]
    pub fn second(self) -> i32 {
        #[allow(clippy::cast_possible_truncation)]
        let second = (self.nano_of_day / NANOS_PER_SECOND % 60) as i32;
        second
    }

    #[must_use]
    pub fn nano(self) -> i32 {
        #[allow(clippy::cast_possible_truncation)]
        let nano = (self.nano_of_day % NANOS_PER_SECOND) as i32;
        nano
    }
}

impl std::fmt::Display for Time {
    /// `java.time`'s own rule: `HH:mm`, with `:ss` only when there is a second
    /// or a fraction to show, and the fraction in whole groups of three —
    /// `10:15`, `10:15:30`, `10:15:30.500`, `01:02:03.000000004`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.hour(), self.minute())?;
        let (second, nano) = (self.second(), self.nano());
        if second == 0 && nano == 0 {
            return Ok(());
        }
        write!(f, ":{second:02}")?;
        if nano == 0 {
            return Ok(());
        }
        if nano % 1_000_000 == 0 {
            write!(f, ".{:03}", nano / 1_000_000)
        } else if nano % 1000 == 0 {
            write!(f, ".{:06}", nano / 1000)
        } else {
            write!(f, ".{nano:09}")
        }
    }
}

/// `LocalTime.parse` — the ISO forms `HH:mm`, `HH:mm:ss` and `HH:mm:ss.fff`.
pub fn parse_time(text: &str) -> Result<Time, String> {
    let invalid = |index: usize| format!("Text '{text}' could not be parsed at index {index}");
    let bytes = text.as_bytes();
    let two = |at: usize| -> Result<i32, String> {
        if at + 2 > bytes.len() || !bytes[at].is_ascii_digit() || !bytes[at + 1].is_ascii_digit() {
            return Err(invalid(at));
        }
        text[at..at + 2].parse().map_err(|_| invalid(at))
    };
    let hour = two(0)?;
    if bytes.get(2) != Some(&b':') {
        return Err(invalid(2));
    }
    let minute = two(3)?;
    let (mut second, mut nano) = (0, 0);
    let mut at = 5;
    if bytes.get(at) == Some(&b':') {
        second = two(at + 1)?;
        at += 3;
        if bytes.get(at) == Some(&b'.') {
            let from = at + 1;
            let mut end = from;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end == from || end - from > 9 {
                return Err(invalid(from));
            }
            let digits = &text[from..end];
            let scaled: i64 = digits.parse().map_err(|_| invalid(from))?;
            #[allow(clippy::cast_possible_truncation)]
            let scaled = (scaled * 10_i64.pow(9 - u32::try_from(digits.len()).unwrap_or(9))) as i32;
            nano = scaled;
            at = end;
        }
    }
    if at != bytes.len() {
        return Err(invalid(at));
    }
    Time::of(hour, minute, second, nano)
}

/// A `java.time.LocalDateTime`: a date and a time, with no zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

impl DateTime {
    /// Add nanoseconds, carrying whole days into the DATE — which is the only
    /// thing a `LocalDateTime` does that its two halves do not.
    #[must_use]
    pub fn plus_nanos(self, nanos: i64) -> Self {
        Self {
            date: self.date.plus_days(self.time.overflow_days(nanos)),
            time: self.time.plus_nanos(nanos),
        }
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}T{}", self.date, self.time)
    }
}

/// `LocalDateTime.parse` — the ISO form, a date and a time joined by `T`.
pub fn parse_date_time(text: &str) -> Result<DateTime, String> {
    let invalid = |index: usize| format!("Text '{text}' could not be parsed at index {index}");
    let Some(split) = text.find('T') else {
        return Err(invalid(text.len().min(10)));
    };
    let date = parse_date(&text[..split]).map_err(|_| invalid(0))?;
    let time = parse_time(&text[split + 1..]).map_err(|_| invalid(split + 1))?;
    Ok(DateTime { date, time })
}

/// A `java.time.Duration`: an amount of time, held as `java.time` holds it —
/// seconds plus a nanosecond part that is never negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    pub seconds: i64,
    pub nanos: i32,
}

impl Duration {
    #[must_use]
    pub fn of_nanos(total: i64) -> Self {
        let seconds = total.div_euclid(NANOS_PER_SECOND);
        #[allow(clippy::cast_possible_truncation)]
        let nanos = total.rem_euclid(NANOS_PER_SECOND) as i32;
        Self { seconds, nanos }
    }

    #[must_use]
    pub fn total_nanos(self) -> i64 {
        self.seconds
            .saturating_mul(NANOS_PER_SECOND)
            .saturating_add(i64::from(self.nanos))
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.seconds == 0 && self.nanos == 0
    }

    #[must_use]
    pub fn is_negative(self) -> bool {
        self.seconds < 0
    }

    /// The PART of each unit, as `java.time`'s `toXPart` reads them: the
    /// hours of a duration that also has minutes in it, not the whole of it
    /// in hours.
    #[must_use]
    pub fn part(self, unit: char) -> i64 {
        let total = self.total_nanos();
        match unit {
            // The PART within a day — `toHours` is the whole span, and
            // `toHoursPart` is what is left after the days are taken out.
            'H' => total / NANOS_PER_HOUR % 24,
            'M' => total / NANOS_PER_MINUTE % 60,
            'S' => total / NANOS_PER_SECOND % 60,
            'm' => total / 1_000_000 % 1_000,
            _ => total % NANOS_PER_SECOND,
        }
    }
}

impl std::fmt::Display for Duration {
    /// ISO-8601, exactly as `java.time` writes it: `PT2H`, `PT1H30M`,
    /// `PT48H` (a duration has no days in its text), `PT1.5S`, `PT-1M-30S`,
    /// and `PT0S` for nothing at all.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            return write!(f, "PT0S");
        }
        // A NEGATIVE duration with a fraction is written as one number per
        // part: -43199.999999999s is `PT-11H-59M-59.999999999S`, so the whole
        // seconds are counted from `seconds + 1` and the fraction is the
        // complement. (`java.time` does exactly this, and the shape is what a
        // program prints.)
        let effective = if self.seconds < 0 && self.nanos > 0 {
            self.seconds + 1
        } else {
            self.seconds
        };
        let hours = effective / 3600;
        let minutes = (effective % 3600) / 60;
        let secs = effective % 60;
        let mut out = String::from("PT");
        if hours != 0 {
            let _ = write!(out, "{hours}H");
        }
        if minutes != 0 {
            let _ = write!(out, "{minutes}M");
        }
        if secs == 0 && self.nanos == 0 && out.len() > 2 {
            return write!(f, "{out}");
        }
        // The sign lives on the whole part, and a value that rounds to zero
        // still shows it: -0.000000001s is `PT-0.000000001S`.
        if secs == 0 && self.nanos > 0 && self.seconds < 0 {
            out.push_str("-0");
        } else {
            let _ = write!(out, "{secs}");
        }
        if self.nanos > 0 {
            let at = out.len();
            let fraction = if self.seconds < 0 {
                2 * NANOS_PER_SECOND - i64::from(self.nanos)
            } else {
                i64::from(self.nanos) + NANOS_PER_SECOND
            };
            let _ = write!(out, "{fraction}");
            // The leading 1 (or 2) was there to keep the zeros; it becomes the
            // decimal point, and trailing zeros go.
            out.replace_range(at..=at, ".");
            while out.ends_with('0') {
                out.pop();
            }
        }
        out.push('S');
        write!(f, "{out}")
    }
}

/// A `java.time.Period`: a number of years, months and days, each kept as
/// written — `P1Y2M5D` is not 425 days, and does not become them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    pub years: i32,
    pub months: i32,
    pub days: i32,
}

impl Period {
    /// `Period.between(start, end)` — `java.time`'s own algorithm, which
    /// counts whole months first and takes the days from what is left, so
    /// `2024-01-15` to `2025-03-20` is `P1Y2M5D` and not a day count.
    #[must_use]
    pub fn between(start: Date, end: Date) -> Self {
        let proleptic = |date: Date| i64::from(date.year) * 12 + i64::from(date.month) - 1;
        let mut total_months = proleptic(end) - proleptic(start);
        let mut days = i32::from(end.day) - i32::from(start.day);
        if total_months > 0 && days < 0 {
            total_months -= 1;
            let moved = start.plus_months(total_months);
            #[allow(clippy::cast_possible_truncation)]
            let difference = (end.to_epoch_day() - moved.to_epoch_day()) as i32;
            days = difference;
        } else if total_months < 0 && days > 0 {
            total_months += 1;
            days -= i32::from(length_of_month(end.year, end.month));
        }
        #[allow(clippy::cast_possible_truncation)]
        Self {
            years: (total_months / 12) as i32,
            months: (total_months % 12) as i32,
            days,
        }
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.years == 0 && self.months == 0 && self.days == 0
    }

    /// `normalized()` folds the MONTHS into years — and only those: a period
    /// keeps its days as written, because a day is not a fixed part of a
    /// month.
    #[must_use]
    pub fn normalized(self) -> Self {
        let total = self.total_months();
        #[allow(clippy::cast_possible_truncation)]
        Self {
            years: (total / 12) as i32,
            months: (total % 12) as i32,
            days: self.days,
        }
    }

    #[must_use]
    pub fn total_months(self) -> i64 {
        i64::from(self.years) * 12 + i64::from(self.months)
    }
}

impl std::fmt::Display for Period {
    /// `P1Y2M3D`, with each part left out when it is zero — and `P0D` when
    /// they all are.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            return write!(f, "P0D");
        }
        write!(f, "P")?;
        if self.years != 0 {
            write!(f, "{}Y", self.years)?;
        }
        if self.months != 0 {
            write!(f, "{}M", self.months)?;
        }
        if self.days != 0 {
            write!(f, "{}D", self.days)?;
        }
        Ok(())
    }
}

/// The `ChronoUnit` constants caturra models, stored under the JDK's OWN
/// ordinal so that `ordinal()` answers what a JDK answers — the two the
/// calendar cannot count in nanoseconds (months and years) are at the end,
/// where `java.time` puts them. Its `toString` is TITLE case ("Days"), which
/// is not its `name()` ("DAYS"): a unit carries a description of its own.
const UNIT_NAMES: [&str; 16] = [
    "Nanos",
    "Micros",
    "Millis",
    "Seconds",
    "Minutes",
    "Hours",
    "HalfDays",
    "Days",
    "Weeks",
    "Months",
    "Years",
    "Decades",
    "Centuries",
    "Millennia",
    "Eras",
    "Forever",
];

/// The `ChronoUnit` names in Java's own spelling, which is not the enum's:
/// `ChronoUnit.HALF_DAYS.toString()` is `HalfDays`.
pub const UNIT_CONSTANTS: [&str; 16] = [
    "NANOS",
    "MICROS",
    "MILLIS",
    "SECONDS",
    "MINUTES",
    "HOURS",
    "HALF_DAYS",
    "DAYS",
    "WEEKS",
    "MONTHS",
    "YEARS",
    "DECADES",
    "CENTURIES",
    "MILLENNIA",
    "ERAS",
    "FOREVER",
];

#[must_use]
pub fn unit_name(unit: u8) -> &'static str {
    UNIT_NAMES[usize::from(unit).min(UNIT_NAMES.len() - 1)]
}

/// How long one unit is ESTIMATED to be — what `ChronoUnit.getDuration`
/// answers. A month is 31556952 seconds divided by twelve (the average
/// Gregorian year), and FOREVER is as long as a `Duration` goes.
#[must_use]
pub fn unit_duration(unit: u8) -> Duration {
    const YEAR_SECONDS: i64 = 31_556_952;
    if let Some(nanos) = unit_nanos(unit) {
        return Duration {
            seconds: nanos / NANOS_PER_SECOND,
            nanos: i32::try_from(nanos % NANOS_PER_SECOND).unwrap_or(0),
        };
    }
    let seconds = match unit {
        9 => YEAR_SECONDS / 12,
        10 => YEAR_SECONDS,
        11 => YEAR_SECONDS * 10,
        12 => YEAR_SECONDS * 100,
        13 => YEAR_SECONDS * 1000,
        14 => YEAR_SECONDS * 1_000_000_000,
        _ => i64::MAX,
    };
    Duration {
        seconds,
        nanos: if unit >= 15 { 999_999_999 } else { 0 },
    }
}

/// Whether a unit measures the CALENDAR (days and up) or the clock.
/// `FOREVER` is neither.
#[must_use]
pub fn unit_is_date_based(unit: u8) -> bool {
    (7..=14).contains(&unit)
}

#[must_use]
pub fn unit_is_time_based(unit: u8) -> bool {
    unit < 7
}

/// Whether a unit's length is an ESTIMATE: every calendar unit is, because a
/// month and a year vary, and so is `FOREVER`. `DAYS` is estimated too — a
/// day is not always 24 hours where there is a time zone.
#[must_use]
pub fn unit_is_estimated(unit: u8) -> bool {
    unit >= 7
}

/// How many nanoseconds one unit is EXACTLY — `None` for the units a
/// CALENDAR defines (a month is not a fixed number of anything).
#[must_use]
pub fn unit_nanos(unit: u8) -> Option<i64> {
    Some(match unit {
        0 => 1,
        1 => 1_000,
        2 => 1_000_000,
        3 => NANOS_PER_SECOND,
        4 => NANOS_PER_MINUTE,
        5 => NANOS_PER_HOUR,
        6 => 12 * NANOS_PER_HOUR,
        7 => NANOS_PER_DAY,
        8 => 7 * NANOS_PER_DAY,
        _ => return None,
    })
}

/// `Duration.parse` — the ISO-8601 form `PnDTnHnMnS`, where every part is
/// optional, each may be signed, and the seconds may have a fraction.
///
/// # Errors
/// The JDK's one message for anything it cannot read.
pub fn parse_duration(text: &str) -> Result<Duration, String> {
    const BAD: &str = "Text cannot be parsed to a Duration";
    let units: Vec<char> = text.chars().collect();
    let mut at = 0;
    let overall: i128 = match units.first() {
        Some('-') => {
            at = 1;
            -1
        }
        Some('+') => {
            at = 1;
            1
        }
        _ => 1,
    };
    if units.get(at).is_none_or(|c| !c.eq_ignore_ascii_case(&'P')) {
        return Err(String::from(BAD));
    }
    at += 1;
    let mut nanos: i128 = 0;
    let mut after_t = false;
    let mut any = false;
    while at < units.len() {
        if units[at].eq_ignore_ascii_case(&'T') {
            if after_t {
                return Err(String::from(BAD));
            }
            after_t = true;
            at += 1;
            continue;
        }
        let start = at;
        if matches!(units.get(at), Some('-' | '+')) {
            at += 1;
        }
        let digits_from = at;
        while units.get(at).is_some_and(char::is_ascii_digit) {
            at += 1;
        }
        if at == digits_from {
            return Err(String::from(BAD));
        }
        let whole: String = units[start..at].iter().collect();
        // A FRACTION belongs to the seconds and to nothing else.
        let mut fraction = String::new();
        if units.get(at) == Some(&'.') {
            at += 1;
            while units.get(at).is_some_and(char::is_ascii_digit) {
                fraction.push(units[at]);
                at += 1;
            }
        }
        let Some(letter) = units.get(at).copied() else {
            return Err(String::from(BAD));
        };
        at += 1;
        let value: i128 = whole.parse().map_err(|_| String::from(BAD))?;
        let per: i128 = match (letter.to_ascii_uppercase(), after_t) {
            ('D', false) => 86_400,
            ('H', true) => 3_600,
            ('M', true) => 60,
            ('S', true) => 1,
            _ => return Err(String::from(BAD)),
        };
        nanos += value * per * 1_000_000_000;
        if !fraction.is_empty() {
            if !letter.eq_ignore_ascii_case(&'S') {
                return Err(String::from(BAD));
            }
            let padded = format!("{fraction:0<9}");
            let part: i128 = padded[..9].parse().unwrap_or(0);
            // The sign of the WHOLE part carries into its fraction.
            nanos += if whole.starts_with('-') { -part } else { part };
        }
        any = true;
    }
    if !any {
        return Err(String::from(BAD));
    }
    let total = nanos * overall;
    Ok(Duration {
        seconds: i64::try_from(total.div_euclid(1_000_000_000)).map_err(|_| String::from(BAD))?,
        nanos: i32::try_from(total.rem_euclid(1_000_000_000)).unwrap_or(0),
    })
}

/// `Period.parse` — `PnYnMnWnD`, where a week is seven days.
///
/// # Errors
/// The JDK's one message for anything it cannot read.
pub fn parse_period(text: &str) -> Result<Period, String> {
    const BAD: &str = "Text cannot be parsed to a Period";
    let units: Vec<char> = text.chars().collect();
    let mut at = 0;
    let overall: i64 = match units.first() {
        Some('-') => {
            at = 1;
            -1
        }
        Some('+') => {
            at = 1;
            1
        }
        _ => 1,
    };
    if units.get(at).is_none_or(|c| !c.eq_ignore_ascii_case(&'P')) {
        return Err(String::from(BAD));
    }
    at += 1;
    let (mut years, mut months, mut days) = (0i64, 0i64, 0i64);
    let mut any = false;
    while at < units.len() {
        let start = at;
        if matches!(units.get(at), Some('-' | '+')) {
            at += 1;
        }
        let digits_from = at;
        while units.get(at).is_some_and(char::is_ascii_digit) {
            at += 1;
        }
        if at == digits_from {
            return Err(String::from(BAD));
        }
        let value: i64 = units[start..at]
            .iter()
            .collect::<String>()
            .parse()
            .map_err(|_| String::from(BAD))?;
        let Some(letter) = units.get(at).copied() else {
            return Err(String::from(BAD));
        };
        at += 1;
        match letter.to_ascii_uppercase() {
            'Y' => years += value,
            'M' => months += value,
            'W' => days += value * 7,
            'D' => days += value,
            _ => return Err(String::from(BAD)),
        }
        any = true;
    }
    if !any {
        return Err(String::from(BAD));
    }
    Ok(Period {
        years: i32::try_from(years * overall).unwrap_or(0),
        months: i32::try_from(months * overall).unwrap_or(0),
        days: i32::try_from(days * overall).unwrap_or(0),
    })
}

/// A `java.time.temporal.TemporalAdjusters` adjuster: a rule for moving a
/// date, remembered as which rule and what it was given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Adjuster {
    pub kind: AdjusterKind,
    /// The day of week `firstInMonth` and its relatives were given, 1..=7.
    pub day: u8,
    /// The count `dayOfWeekInMonth` was given, which may be negative.
    pub ordinal: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdjusterKind {
    FirstDayOfMonth,
    LastDayOfMonth,
    FirstDayOfNextMonth,
    FirstDayOfYear,
    LastDayOfYear,
    FirstDayOfNextYear,
    FirstInMonth,
    LastInMonth,
    DayOfWeekInMonth,
    Next,
    NextOrSame,
    Previous,
    PreviousOrSame,
}

impl Adjuster {
    /// The date this adjuster makes of `date`.
    #[must_use]
    pub fn apply(self, date: Date) -> Date {
        let wanted = i32::from(self.day);
        let current = i32::from(date.day_of_week());
        match self.kind {
            AdjusterKind::FirstDayOfMonth => {
                Date::from_epoch_day(date.to_epoch_day() - i64::from(date.day) + 1)
            }
            AdjusterKind::LastDayOfMonth => Date::from_epoch_day(
                date.to_epoch_day() - i64::from(date.day)
                    + i64::from(length_of_month(date.year, date.month)),
            ),
            AdjusterKind::FirstDayOfNextMonth => {
                let last = Adjuster {
                    kind: AdjusterKind::LastDayOfMonth,
                    ..self
                }
                .apply(date);
                Date::from_epoch_day(last.to_epoch_day() + 1)
            }
            AdjusterKind::FirstDayOfYear => {
                Date::from_epoch_day(date.to_epoch_day() - i64::from(date.day_of_year()) + 1)
            }
            AdjusterKind::LastDayOfYear => Date::from_epoch_day(
                date.to_epoch_day() - i64::from(date.day_of_year())
                    + if is_leap_year(date.year) { 366 } else { 365 },
            ),
            AdjusterKind::FirstDayOfNextYear => {
                let last = Adjuster {
                    kind: AdjusterKind::LastDayOfYear,
                    ..self
                }
                .apply(date);
                Date::from_epoch_day(last.to_epoch_day() + 1)
            }
            // The first of a weekday in the month: go to the first of the
            // month and then forwards to the day wanted.
            AdjusterKind::FirstInMonth => Adjuster {
                kind: AdjusterKind::DayOfWeekInMonth,
                ordinal: 1,
                ..self
            }
            .apply(date),
            AdjusterKind::LastInMonth => Adjuster {
                kind: AdjusterKind::DayOfWeekInMonth,
                ordinal: -1,
                ..self
            }
            .apply(date),
            AdjusterKind::DayOfWeekInMonth => {
                if self.ordinal >= 0 {
                    let first = Adjuster {
                        kind: AdjusterKind::FirstDayOfMonth,
                        ..self
                    }
                    .apply(date);
                    let ahead = (wanted - i32::from(first.day_of_week())).rem_euclid(7);
                    let weeks = i64::from(self.ordinal.max(1) - 1);
                    Date::from_epoch_day(first.to_epoch_day() + i64::from(ahead) + weeks * 7)
                } else {
                    let last = Adjuster {
                        kind: AdjusterKind::LastDayOfMonth,
                        ..self
                    }
                    .apply(date);
                    let behind = (i32::from(last.day_of_week()) - wanted).rem_euclid(7);
                    let weeks = i64::from(-self.ordinal - 1);
                    Date::from_epoch_day(last.to_epoch_day() - i64::from(behind) - weeks * 7)
                }
            }
            // NEXT is strictly forwards; `nextOrSame` stays put when it is
            // already the day wanted.
            AdjusterKind::Next | AdjusterKind::NextOrSame => {
                let ahead = (wanted - current).rem_euclid(7);
                let ahead = if ahead == 0 && self.kind == AdjusterKind::Next {
                    7
                } else {
                    ahead
                };
                Date::from_epoch_day(date.to_epoch_day() + i64::from(ahead))
            }
            AdjusterKind::Previous | AdjusterKind::PreviousOrSame => {
                let behind = (current - wanted).rem_euclid(7);
                let behind = if behind == 0 && self.kind == AdjusterKind::Previous {
                    7
                } else {
                    behind
                };
                Date::from_epoch_day(date.to_epoch_day() - i64::from(behind))
            }
        }
    }
}

/// `java.time.temporal.ChronoField`, in the JDK's own order — a program names
/// one to ask a date or a time for a single field, and the answers below are
/// the JDK's, recorded field by field.
///
/// Each entry is the enum CONSTANT, the `toString` spelling, the base unit,
/// the range unit, and the field's own `range()` as
/// (minimum, largest minimum, smallest maximum, maximum).
#[derive(Debug)]
pub struct FieldInfo {
    pub constant: &'static str,
    pub text: &'static str,
    pub base_unit: u8,
    pub range_unit: u8,
    pub range: (i64, i64, i64, i64),
}

const fn field(
    constant: &'static str,
    text: &'static str,
    base_unit: u8,
    range_unit: u8,
    range: (i64, i64, i64, i64),
) -> FieldInfo {
    FieldInfo {
        constant,
        text,
        base_unit,
        range_unit,
        range,
    }
}

pub static FIELDS: &[FieldInfo] = &[
    field(
        "NANO_OF_SECOND",
        "NanoOfSecond",
        0,
        3,
        (0, 0, 999_999_999, 999_999_999),
    ),
    field(
        "NANO_OF_DAY",
        "NanoOfDay",
        0,
        7,
        (0, 0, 86_399_999_999_999, 86_399_999_999_999),
    ),
    field(
        "MICRO_OF_SECOND",
        "MicroOfSecond",
        1,
        3,
        (0, 0, 999_999, 999_999),
    ),
    field(
        "MICRO_OF_DAY",
        "MicroOfDay",
        1,
        7,
        (0, 0, 86_399_999_999, 86_399_999_999),
    ),
    field("MILLI_OF_SECOND", "MilliOfSecond", 2, 3, (0, 0, 999, 999)),
    field(
        "MILLI_OF_DAY",
        "MilliOfDay",
        2,
        7,
        (0, 0, 86_399_999, 86_399_999),
    ),
    field("SECOND_OF_MINUTE", "SecondOfMinute", 3, 4, (0, 0, 59, 59)),
    field("SECOND_OF_DAY", "SecondOfDay", 3, 7, (0, 0, 86_399, 86_399)),
    field("MINUTE_OF_HOUR", "MinuteOfHour", 4, 5, (0, 0, 59, 59)),
    field("MINUTE_OF_DAY", "MinuteOfDay", 4, 7, (0, 0, 1439, 1439)),
    field("HOUR_OF_AMPM", "HourOfAmPm", 5, 6, (0, 0, 11, 11)),
    field(
        "CLOCK_HOUR_OF_AMPM",
        "ClockHourOfAmPm",
        5,
        6,
        (1, 1, 12, 12),
    ),
    field("HOUR_OF_DAY", "HourOfDay", 5, 7, (0, 0, 23, 23)),
    field("CLOCK_HOUR_OF_DAY", "ClockHourOfDay", 5, 7, (1, 1, 24, 24)),
    field("AMPM_OF_DAY", "AmPmOfDay", 6, 7, (0, 0, 1, 1)),
    field("DAY_OF_WEEK", "DayOfWeek", 7, 8, (1, 1, 7, 7)),
    field(
        "ALIGNED_DAY_OF_WEEK_IN_MONTH",
        "AlignedDayOfWeekInMonth",
        7,
        8,
        (1, 1, 7, 7),
    ),
    field(
        "ALIGNED_DAY_OF_WEEK_IN_YEAR",
        "AlignedDayOfWeekInYear",
        7,
        8,
        (1, 1, 7, 7),
    ),
    field("DAY_OF_MONTH", "DayOfMonth", 7, 9, (1, 1, 28, 31)),
    field("DAY_OF_YEAR", "DayOfYear", 7, 10, (1, 1, 365, 366)),
    field(
        "EPOCH_DAY",
        "EpochDay",
        7,
        15,
        (
            -365_243_219_162,
            -365_243_219_162,
            365_241_780_471,
            365_241_780_471,
        ),
    ),
    field(
        "ALIGNED_WEEK_OF_MONTH",
        "AlignedWeekOfMonth",
        8,
        9,
        (1, 1, 4, 5),
    ),
    field(
        "ALIGNED_WEEK_OF_YEAR",
        "AlignedWeekOfYear",
        8,
        10,
        (1, 1, 53, 53),
    ),
    field("MONTH_OF_YEAR", "MonthOfYear", 9, 10, (1, 1, 12, 12)),
    field(
        "PROLEPTIC_MONTH",
        "ProlepticMonth",
        9,
        15,
        (
            -11_999_999_988,
            -11_999_999_988,
            11_999_999_999,
            11_999_999_999,
        ),
    ),
    field(
        "YEAR_OF_ERA",
        "YearOfEra",
        10,
        15,
        (1, 1, 999_999_999, 1_000_000_000),
    ),
    field(
        "YEAR",
        "Year",
        10,
        15,
        (-999_999_999, -999_999_999, 999_999_999, 999_999_999),
    ),
    field("ERA", "Era", 14, 15, (0, 0, 1, 1)),
    field(
        "INSTANT_SECONDS",
        "InstantSeconds",
        3,
        15,
        (i64::MIN, i64::MIN, i64::MAX, i64::MAX),
    ),
    field(
        "OFFSET_SECONDS",
        "OffsetSeconds",
        3,
        15,
        (-64800, -64800, 64800, 64800),
    ),
];

/// A field by its `ChronoField` constant name.
#[must_use]
pub fn field_by_constant(name: &str) -> Option<u8> {
    FIELDS
        .iter()
        .position(|info| info.constant == name)
        .and_then(|at| u8::try_from(at).ok())
}

#[must_use]
pub fn field_info(field: u8) -> &'static FieldInfo {
    &FIELDS[usize::from(field).min(FIELDS.len() - 1)]
}

/// A field's own `range()`, before any date narrows it.
#[must_use]
pub fn field_range(field: u8) -> ValueRange {
    let (min, largest_min, smallest_max, max) = field_info(field).range;
    ValueRange {
        min,
        largest_min,
        smallest_max,
        max,
    }
}

/// Whether a field measures the CALENDAR. `INSTANT_SECONDS` and
/// `OFFSET_SECONDS` are neither date- nor time-based: they need a zone.
#[must_use]
pub fn field_is_date_based(field: u8) -> bool {
    (15..=27).contains(&field)
}

#[must_use]
pub fn field_is_time_based(field: u8) -> bool {
    field < 15
}

/// `java.time.temporal.ValueRange` — what a field can hold, which for a few
/// fields depends on the date: February has 29 days this year and 28 the
/// next, so `DAY_OF_MONTH` has a smallest maximum and a maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ValueRange {
    pub min: i64,
    pub largest_min: i64,
    pub smallest_max: i64,
    pub max: i64,
}

impl ValueRange {
    #[must_use]
    pub fn fixed(min: i64, max: i64) -> ValueRange {
        ValueRange {
            min,
            largest_min: min,
            smallest_max: max,
            max,
        }
    }

    #[must_use]
    pub fn is_fixed(&self) -> bool {
        self.min == self.largest_min && self.smallest_max == self.max
    }

    #[must_use]
    pub fn contains(&self, value: i64) -> bool {
        self.min <= value && value <= self.max
    }

    /// The JDK's own wording: "1 - 28/31" where the maximum varies, and
    /// "1/2 - 28/31" where the minimum does too.
    #[must_use]
    pub fn text(&self) -> String {
        let mut out = self.min.to_string();
        if self.min != self.largest_min {
            out.push('/');
            out.push_str(&self.largest_min.to_string());
        }
        out.push_str(" - ");
        out.push_str(&self.smallest_max.to_string());
        if self.smallest_max != self.max {
            out.push('/');
            out.push_str(&self.max.to_string());
        }
        out
    }
}

/// One piece of a `DateTimeFormatter` pattern: a field to print, or text to
/// print as it stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece {
    /// A field and how many letters were written for it — the count decides
    /// the width, and for a month or a day-of-week it decides text vs number.
    Field(char, usize),
    Literal(String),
    /// `[...]` — printed only when the value has every field inside it, and
    /// optional when parsing.
    Optional(Vec<Piece>),
    /// `ppppX` — the piece after the pad letters, right-justified in that
    /// many columns.
    Pad(usize, Box<Piece>),
}

/// Why a pattern could not be printed: a field the value has not got, or a
/// ZONE, which no `LocalDate`, `LocalTime` or `LocalDateTime` carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatFail {
    /// `UnsupportedTemporalTypeException: Unsupported field: X`.
    Field(String),
    /// `DateTimeException: Unable to extract ZoneId from temporal X`.
    Zone,
}

/// Parse a `DateTimeFormatter.ofPattern` pattern. `Err` is the
/// `IllegalArgumentException`'s text, which the JDK spells out.
pub fn parse_pattern(pattern: &str) -> Result<Vec<Piece>, String> {
    let letters: Vec<char> = pattern.chars().collect();
    let mut at = 0;
    let pieces = parse_section(pattern, &letters, &mut at, false)?;
    if at < letters.len() {
        // Only a `]` with no `[` in front of it stops the top-level scan.
        return Err(String::from("Pattern invalid: ], using Locale.US"));
    }
    Ok(pieces)
}

/// Every letter `java.time` knows. Anything else is "Unknown pattern letter",
/// which is a different complaint from a letter it knows and cannot use here.
const PATTERN_LETTERS: &str = "GuyDMLdQqYwWEecFaBhKkHmsSAnNVzOXxZp";

/// How many of a letter may be written in a row.
fn most_letters(letter: char) -> usize {
    match letter {
        'a' | 'W' | 'F' => 1,
        'd' | 'H' | 'h' | 'K' | 'k' | 'm' | 's' | 'w' => 2,
        'D' => 3,
        'z' => 4,
        'G' | 'M' | 'L' | 'E' | 'e' | 'c' | 'Q' | 'q' | 'X' | 'x' | 'Z' => 5,
        'S' => 9,
        _ => 19,
    }
}

/// One run of pattern pieces, stopping at the `]` that closes an optional
/// section when `nested` says there is one to close.
#[allow(clippy::too_many_lines)] // one arm per pattern shape
fn parse_section(
    pattern: &str,
    letters: &[char],
    at: &mut usize,
    nested: bool,
) -> Result<Vec<Piece>, String> {
    let mut pieces: Vec<Piece> = Vec::new();
    while *at < letters.len() {
        let letter = letters[*at];
        if letter == ']' {
            if nested {
                *at += 1;
                return Ok(pieces);
            }
            return Ok(pieces);
        }
        if letter == '[' {
            *at += 1;
            let start = *at;
            let inner = parse_section(pattern, letters, at, true)?;
            if *at == start && letters.get(start) != Some(&']') {
                return Err(String::from(
                    "Pattern ends with an incomplete optional section",
                ));
            }
            pieces.push(Piece::Optional(inner));
            continue;
        }
        // A QUOTED run is one piece; `''` inside or outside one is a quote.
        if letter == '\'' {
            *at += 1;
            if letters.get(*at) == Some(&'\'') {
                pieces.push(Piece::Literal(String::from("'")));
                *at += 1;
                continue;
            }
            let mut text = String::new();
            loop {
                let Some(next) = letters.get(*at).copied() else {
                    return Err(format!(
                        "Pattern ends with an incomplete string literal: {}",
                        pattern
                            .chars()
                            .skip(*at - text.chars().count() - 1)
                            .collect::<String>()
                    ));
                };
                *at += 1;
                if next == '\'' {
                    // `''` inside a quoted run is one quote, not the end.
                    if letters.get(*at) == Some(&'\'') {
                        text.push('\'');
                        *at += 1;
                        continue;
                    }
                    break;
                }
                text.push(next);
            }
            pieces.push(Piece::Literal(text));
            continue;
        }
        if !letter.is_ascii_alphabetic() {
            // `{`, `}` and `#` are held back for a future release, and a JDK
            // refuses them rather than printing them.
            if matches!(letter, '{' | '}' | '#') {
                return Err(format!("Pattern includes reserved character: '{letter}'"));
            }
            pieces.push(Piece::Literal(letter.to_string()));
            *at += 1;
            continue;
        }
        let mut count = 0;
        while at.saturating_add(count) < letters.len() && letters[*at + count] == letter {
            count += 1;
        }
        if !PATTERN_LETTERS.contains(letter) {
            return Err(format!("Unknown pattern letter: {letter}"));
        }
        // `B` is a Java 16 letter; 11 does not know it.
        if letter == 'B' {
            return Err(format!("Unknown pattern letter: {letter}"));
        }
        if letter == 'V' && count != 2 {
            return Err(String::from("Pattern letter count must be 2: V"));
        }
        if letter == 'O' && count != 1 && count != 4 {
            return Err(String::from("Pattern letter count must be 1 or 4: O"));
        }
        if letter == 'c' && count == 2 {
            return Err(String::from("Invalid pattern \"cc\""));
        }
        if count > most_letters(letter) {
            return Err(format!("Too many pattern letters: {letter}"));
        }
        *at += count;
        // `p` pads whatever comes NEXT into `count` columns.
        if letter == 'p' {
            let mut padded = parse_section(pattern, letters, at, nested)?;
            if padded.is_empty() {
                return Err(format!(
                    "Pad letter 'p' must be followed by valid pad pattern: {}",
                    "p".repeat(count)
                ));
            }
            let first = padded.remove(0);
            pieces.push(Piece::Pad(count, Box::new(first)));
            pieces.extend(padded);
            continue;
        }
        pieces.push(Piece::Field(letter, count));
    }
    if nested {
        return Err(String::from(
            "Pattern ends with an incomplete optional section",
        ));
    }
    Ok(pieces)
}

/// The `ChronoField` a pattern letter reads, as the JDK NAMES it — which is
/// what its "Unsupported field" message says when the value has no such part.
fn field_name(letter: char) -> &'static str {
    match letter {
        // `u` is the proleptic year and `y` is the year OF THE ERA. They are
        // the same number for every date a student writes, and different
        // names in every message.
        'u' => "Year",
        'H' => "HourOfDay",
        'h' => "ClockHourOfAmPm",
        'm' => "MinuteOfHour",
        's' => "SecondOfMinute",
        'S' | 'n' => "NanoOfSecond",
        'a' => "AmPmOfDay",
        'y' => "YearOfEra",
        'M' | 'L' => "MonthOfYear",
        'd' => "DayOfMonth",
        'D' => "DayOfYear",
        'K' => "HourOfAmPm",
        'k' => "ClockHourOfDay",
        'A' => "MilliOfDay",
        'N' => "NanoOfDay",
        'G' => "Era",
        'Q' | 'q' => "QuarterOfYear",
        'F' => "AlignedDayOfWeekInMonth",
        // `E`, and the week fields too: those are resolved through the
        // locale's week rules, and the first piece those want is the day of
        // the week — which is what a JDK names when the value has not got
        // one.
        _ => "DayOfWeek",
    }
}

/// The month's name in title case — "February" — and its short form.
#[must_use]
pub fn month_text(month: u8, short: bool) -> String {
    if short {
        SHORT_MONTHS[usize::from(month.clamp(1, 12)) - 1].to_owned()
    } else {
        title(month_name(month))
    }
}

/// The day's name in title case — "Thursday" — and its short form.
#[must_use]
pub fn day_text(day: u8, short: bool) -> String {
    if short {
        SHORT_DAYS[usize::from(day.clamp(1, 7)) - 1].to_owned()
    } else {
        title(day_name(day))
    }
}

const SHORT_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const SHORT_DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// Title case, as a formatter's text fields print: "May", "Saturday".
fn title(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..1]);
    out.push_str(&text[1..].to_lowercase());
    out
}

/// Print `date`/`time` through a parsed pattern. `Err` is the name of a field
/// the value does not have — a `LocalDate` has no `HourOfDay`.
#[allow(clippy::too_many_lines)] // one arm per pattern letter
pub fn format_pieces(
    pieces: &[Piece],
    date: Option<Date>,
    time: Option<Time>,
) -> Result<String, FormatFail> {
    let mut out = String::new();
    for piece in pieces {
        let (letter, count) = match piece {
            Piece::Literal(text) => {
                out.push_str(text);
                continue;
            }
            // An OPTIONAL section prints only if the value has everything in
            // it, and prints nothing at all otherwise.
            Piece::Optional(inner) => {
                if let Ok(text) = format_pieces(inner, date, time) {
                    out.push_str(&text);
                }
                continue;
            }
            Piece::Pad(width, inner) => {
                let text = format_pieces(std::slice::from_ref(inner.as_ref()), date, time)?;
                for _ in text.chars().count()..*width {
                    out.push(' ');
                }
                out.push_str(&text);
                continue;
            }
            Piece::Field(letter, count) => (*letter, *count),
        };
        // A ZONE is not a field: no local value carries one, and a JDK says
        // so in different words.
        if matches!(letter, 'z' | 'V') {
            return Err(FormatFail::Zone);
        }
        // An OFFSET is a field, and no local value has it — whatever else
        // the value is missing.
        if matches!(letter, 'O' | 'X' | 'x' | 'Z') {
            return Err(FormatFail::Field(String::from("OffsetSeconds")));
        }
        let needs_time = matches!(
            letter,
            'H' | 'h' | 'K' | 'k' | 'm' | 's' | 'S' | 'a' | 'A' | 'n' | 'N'
        );
        if needs_time && time.is_none() || !needs_time && date.is_none() {
            return Err(FormatFail::Field(field_name(letter).to_owned()));
        }
        let (date, time) = (
            date.unwrap_or(Date {
                year: 0,
                month: 1,
                day: 1,
            }),
            time.unwrap_or(Time { nano_of_day: 0 }),
        );
        match letter {
            // `y` is the year OF THE ERA — always positive, so year -1 is
            // the year 2 (of the era before this one) — while `u` is the
            // proleptic year, which carries its sign. Two letters of either
            // is the last two digits.
            'y' | 'u' => {
                let value = if letter == 'y' && date.year <= 0 {
                    1 - date.year
                } else {
                    date.year
                };
                if count == 2 {
                    // The last two digits of the MAGNITUDE: `uu` of year -44
                    // is 44, not the 56 a modulo of a negative would give.
                    let _ = write!(out, "{:02}", value.abs() % 100);
                } else {
                    // The width counts DIGITS; the sign sits outside it, and
                    // a number too wide for the pattern is marked with `+`.
                    let digits = format!("{:0width$}", value.abs(), width = count);
                    if value < 0 {
                        out.push('-');
                    } else if digits.len() > count && count >= 4 {
                        out.push('+');
                    }
                    out.push_str(&digits);
                }
            }
            // Five letters is the NARROW form, which is the first letter of
            // the name — "J" for January, "M" for Monday.
            'M' => match count {
                1 | 2 => {
                    let _ = write!(out, "{:0width$}", date.month, width = count);
                }
                3 => out.push_str(SHORT_MONTHS[usize::from(date.month) - 1]),
                5 => out.push_str(&title(month_name(date.month))[..1]),
                _ => out.push_str(&title(month_name(date.month))),
            },
            'd' => {
                let _ = write!(out, "{:0width$}", date.day, width = count);
            }
            'D' => {
                let _ = write!(out, "{:0width$}", date.day_of_year(), width = count);
            }
            // One to three letters of `E` are all the short name; four is the
            // long one.
            'E' => {
                let day = usize::from(date.day_of_week()) - 1;
                let long = title(day_name(date.day_of_week()));
                match count {
                    1..=3 => out.push_str(SHORT_DAYS[day]),
                    5 => out.push_str(&long[..1]),
                    _ => out.push_str(&long),
                }
            }
            'H' => {
                let _ = write!(out, "{:0width$}", time.hour(), width = count);
            }
            'h' => {
                let hour = match time.hour() % 12 {
                    0 => 12,
                    other => other,
                };
                let _ = write!(out, "{hour:0count$}");
            }
            'm' => {
                let _ = write!(out, "{:0width$}", time.minute(), width = count);
            }
            's' => {
                let _ = write!(out, "{:0width$}", time.second(), width = count);
            }
            'S' => {
                let digits = time.nano() / 10_i32.pow(u32::try_from(9 - count.min(9)).unwrap_or(0));
                let _ = write!(out, "{digits:0width$}", width = count.min(9));
            }
            'a' => out.push_str(if time.hour() < 12 { "AM" } else { "PM" }),
            // The ERA, whose text is the only thing the count changes.
            'G' => {
                let era = date.year >= 1;
                out.push_str(match (count, era) {
                    (4, true) => "Anno Domini",
                    (4, false) => "Before Christ",
                    (5, true) => "A",
                    (5, false) => "B",
                    (_, true) => "AD",
                    (_, false) => "BC",
                });
            }
            // `K` is the hour WITHIN the half-day (0-11); `k` counts the day
            // from one to twenty-four.
            'K' => {
                let _ = write!(out, "{:0width$}", time.hour() % 12, width = count);
            }
            'k' => {
                let hour = if time.hour() == 0 { 24 } else { time.hour() };
                let _ = write!(out, "{hour:0count$}");
            }
            // The three whole-count fractions. The letter count is a MINIMUM
            // width, and every one of these is wider than five.
            'A' => {
                let _ = write!(
                    out,
                    "{:0width$}",
                    time.nano_of_day / 1_000_000,
                    width = count
                );
            }
            'n' => {
                let _ = write!(out, "{:0width$}", time.nano(), width = count);
            }
            'N' => {
                let _ = write!(out, "{:0width$}", time.nano_of_day, width = count);
            }
            // `L` is the STANDALONE month, which in English reads the same as
            // `M`; `q` is the standalone quarter.
            'L' => match count {
                1 | 2 => {
                    let _ = write!(out, "{:0width$}", date.month, width = count);
                }
                3 => out.push_str(SHORT_MONTHS[usize::from(date.month) - 1]),
                5 => out.push_str(&title(month_name(date.month))[..1]),
                _ => out.push_str(&title(month_name(date.month))),
            },
            'Q' | 'q' => {
                let quarter = (date.month - 1) / 3 + 1;
                match count {
                    1 | 2 => {
                        let _ = write!(out, "{quarter:0count$}");
                    }
                    3 => {
                        let _ = write!(out, "Q{quarter}");
                    }
                    4 => {
                        let _ = write!(out, "{}{} quarter", quarter, ordinal_suffix(quarter));
                    }
                    _ => {
                        let _ = write!(out, "{quarter}");
                    }
                }
            }
            // The two ALIGNED fields: which week of the month a day is in,
            // and which of that weekday it is.
            'W' => {
                let _ = write!(out, "{}", (date.day - 1) / 7 + 1);
            }
            'F' => {
                let _ = write!(out, "{}", (date.day - 1) % 7 + 1);
            }
            // `e` and `c` are the day of week counted the way the LOCALE
            // counts it — this formatter is en-US throughout, where the week
            // begins on Sunday.
            'e' | 'c' => {
                let localized = date.day_of_week() % 7 + 1;
                let long = title(day_name(date.day_of_week()));
                match count {
                    1 | 2 => {
                        let _ = write!(out, "{localized:0count$}");
                    }
                    3 => out.push_str(SHORT_DAYS[usize::from(date.day_of_week()) - 1]),
                    5 => out.push_str(&long[..1]),
                    _ => out.push_str(&long),
                }
            }
            // The WEEK-BASED year and its week number, under the same en-US
            // rules: a week begins on Sunday, and week one is whichever
            // holds the first of January.
            'Y' => {
                let value = date.year;
                if count == 2 {
                    let _ = write!(out, "{:02}", value.abs() % 100);
                } else {
                    // The width counts DIGITS, and the sign sits outside it.
                    if value < 0 {
                        out.push('-');
                    }
                    let _ = write!(out, "{:0width$}", value.abs(), width = count);
                }
            }
            'w' => {
                let _ = write!(out, "{:0width$}", week_of_year(date), width = count);
            }
            // The OFFSET letters. A local value has no offset either, and a
            // JDK calls that an unsupported FIELD rather than a missing zone.
            _ => return Err(FormatFail::Field(String::from("OffsetSeconds"))),
        }
    }
    Ok(out)
}

/// "1st", "2nd", "3rd", "4th" — what `QQQQ` writes.
fn ordinal_suffix(value: u8) -> &'static str {
    match value {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}

/// The week of the year a date falls in, counted the way en-US counts it:
/// the week begins on SUNDAY, and week one is whichever holds January 1.
fn week_of_year(date: Date) -> i32 {
    let first = Date {
        year: date.year,
        month: 1,
        day: 1,
    };
    // Sunday is 7 in `java.time` and 0 here, which is where the week starts.
    let offset = i32::from(first.day_of_week() % 7);
    (date.day_of_year() + offset - 1) / 7 + 1
}

/// The month or day-of-week NAME at the start of `text` — long form first, so
/// "May" is not read as a short "May" when the pattern asked for the long one.
/// Answers the index and how many bytes it took.
fn match_name(letter: char, text: &str) -> Option<(usize, usize)> {
    let names: &[&str] = if letter == 'M' {
        &SHORT_MONTHS
    } else {
        &SHORT_DAYS
    };
    (0..names.len()).find_map(|index| {
        #[allow(clippy::cast_possible_truncation)]
        let ordinal = (index + 1) as u8;
        let full = if letter == 'M' {
            title(month_name(ordinal))
        } else {
            title(day_name(ordinal))
        };
        if text.starts_with(full.as_str()) {
            Some((index, full.len()))
        } else if text.starts_with(names[index]) {
            Some((index, names[index].len()))
        } else {
            None
        }
    })
}

/// What a pattern's fields resolve TO. `ofPattern` is SMART about it: a day
/// the month does not have is pulled back to the last one it does (February
/// 30th is the 29th in a leap year), while a field outside its range at all is
/// refused — and refused with a REASON rather than an index.
#[allow(clippy::too_many_arguments)]
fn resolve_smartly(
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
    hour: Option<i32>,
    minute: Option<i32>,
    second: Option<i32>,
    afternoon: Option<bool>,
) -> Result<(Option<Date>, Option<Time>), String> {
    let date = match (year, month, day) {
        (Some(year), Some(month), Some(day)) => {
            if !(1..=12).contains(&month) {
                return Err(format!(
                    ": Invalid value for MonthOfYear (valid values 1 - 12): {month}"
                ));
            }
            if !(1..=31).contains(&day) {
                return Err(format!(
                    ": Invalid value for DayOfMonth (valid values 1 - 28/31): {day}"
                ));
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let length = i32::from(length_of_month(year, month as u8));
            Some(Date::of(year, month, day.min(length)).map_err(|reason| format!(": {reason}"))?)
        }
        _ => None,
    };
    let time = match hour {
        Some(hour) => {
            let hour = match afternoon {
                Some(true) if hour < 12 => hour + 12,
                Some(false) if hour == 12 => 0,
                _ => hour,
            };
            Some(
                Time::of(hour, minute.unwrap_or(0), second.unwrap_or(0), 0)
                    .map_err(|reason| format!(": {reason}"))?,
            )
        }
        None => None,
    };
    Ok((date, time))
}

/// Read `text` through a parsed pattern, answering whichever of a date and a
/// time it could build. `Err` is the index the JDK blames.
#[allow(clippy::too_many_lines)] // one arm per pattern letter
pub fn parse_pieces(pieces: &[Piece], text: &str) -> Result<(Option<Date>, Option<Time>), String> {
    let at_index = |index: usize| format!(" at index {index}");
    let bytes = text.as_bytes();
    let mut at = 0usize;
    let (mut year, mut month, mut day) = (None, None, None);
    let (mut hour, mut minute, mut second) = (None, None, None);
    let mut afternoon = None;
    let mut day_of_year: Option<i32> = None;
    for piece in pieces {
        let (letter, count) = match piece {
            Piece::Literal(literal) => {
                if !text[at.min(text.len())..].starts_with(literal.as_str()) {
                    return Err(at_index(at));
                }
                at += literal.len();
                continue;
            }
            // An OPTIONAL section is read if it is there and skipped if it
            // is not; a PAD is the piece it wraps, after its spaces.
            Piece::Optional(inner) => {
                if let Ok((made_date, made_time)) = parse_pieces(inner, &text[at.min(text.len())..])
                {
                    let _ = (made_date, made_time);
                }
                continue;
            }
            Piece::Pad(width, inner) => {
                let mut spaces = 0;
                while spaces + 1 < *width && text[at.min(text.len())..].starts_with(' ') {
                    at += 1;
                    spaces += 1;
                }
                match inner.as_ref() {
                    Piece::Field(letter, count) => (*letter, *count),
                    _ => continue,
                }
            }
            Piece::Field(letter, count) => (*letter, *count),
        };
        if letter == 'a' {
            let rest = &text[at.min(text.len())..];
            if rest.starts_with("AM") {
                afternoon = Some(false);
            } else if rest.starts_with("PM") {
                afternoon = Some(true);
            } else {
                return Err(at_index(at));
            }
            at += 2;
            continue;
        }
        if matches!(letter, 'M' | 'E') && count >= 3 {
            let Some((index, width)) = match_name(letter, &text[at.min(text.len())..]) else {
                return Err(at_index(at));
            };
            if letter == 'M' {
                month = i32::try_from(index + 1).ok();
            }
            at += width;
            continue;
        }
        // A number: as many digits as the pattern asks for, or as many as
        // there are when the pattern wrote one letter. A YEAR may carry a
        // sign, and a signed one is read greedily — which is how a year
        // outside four digits is written back (`+10000-01-01`).
        let from = at;
        let signed = matches!(letter, 'y' | 'u') && matches!(bytes.get(at), Some(b'+' | b'-'));
        if signed {
            at += 1;
        }
        let wanted = if count == 1 || signed {
            usize::MAX
        } else {
            count
        };
        let digits_from = at;
        while at < bytes.len() && bytes[at].is_ascii_digit() && at - digits_from < wanted {
            at += 1;
        }
        if at == digits_from {
            return Err(at_index(from));
        }
        // `parse` reads the sign too, and a leading `+` is one Rust does not
        // take, so it is dropped here.
        let digits = text[from..at].trim_start_matches('+');
        // A four-letter year is written with a SIGN when it needs more room
        // than that (`+10000`), so an unsigned run of more digits is not a
        // wide year — it is a bad one, and the field is what failed.
        if matches!(letter, 'y' | 'u')
            && !signed
            && count >= 4
            && bytes.get(at).is_some_and(u8::is_ascii_digit)
        {
            return Err(at_index(from));
        }
        let value: i64 = digits.parse().map_err(|_| at_index(from))?;
        #[allow(clippy::cast_possible_truncation)]
        let value = value as i32;
        match letter {
            // Two digits of a year are 2000-based, as `ofPattern("yy")` reads
            // them.
            'y' | 'u' => year = Some(if count == 2 { 2000 + value } else { value }),
            'M' => month = Some(value),
            'd' => day = Some(value),
            'H' | 'h' => hour = Some(value),
            'm' => minute = Some(value),
            's' => second = Some(value),
            // The day OF THE YEAR names a date on its own, once the year is
            // known — which it always is by the time the pattern reaches it.
            'D' => day_of_year = Some(value),
            _ => {}
        }
    }
    if at != text.len() {
        return Err(at_index(at));
    }
    if let (Some(day_of_year), Some(year), None) = (day_of_year, year, day) {
        let start = Date::of(year, 1, 1).map_err(|_| at_index(0))?;
        let made = Date::from_epoch_day(start.to_epoch_day() + i64::from(day_of_year) - 1);
        return Ok((Some(made), None));
    }
    resolve_smartly(year, month, day, hour, minute, second, afternoon)
}

/// How a JDK DESCRIBES a formatter — `DateTimeFormatter.toString` prints the
/// printer it built, not the pattern it was given, and a program that prints
/// one sees this. Every shape here was read off a real JDK.
#[must_use]
pub fn describe_pieces(pieces: &[Piece]) -> String {
    let mut out = String::new();
    for piece in pieces {
        match piece {
            // A literal that IS a quote prints as two of them, not as a
            // quote inside quotes.
            Piece::Literal(text) if text == "'" => out.push_str("''"),
            Piece::Literal(text) => {
                let _ = write!(out, "'{text}'");
            }
            // A pattern's DESCRIPTION spells an optional section in
            // brackets and a pad as the pad it is.
            Piece::Optional(inner) => {
                let _ = write!(out, "[{}]", describe_pieces(inner));
            }
            Piece::Pad(width, inner) => {
                let _ = write!(
                    out,
                    "Pad({},{width})",
                    describe_pieces(std::slice::from_ref(inner.as_ref()))
                );
            }
            Piece::Field(letter, count) => {
                let field = field_name(*letter);
                let count = *count;
                match letter {
                    // A month or a day-of-week written three times or more is
                    // TEXT; up to two is a number.
                    // A day-of-week is ALWAYS text; a month is text from
                    // three letters. Five letters is the narrow form.
                    'E' | 'M' if *letter == 'E' || count >= 3 => {
                        let form = match count {
                            1..=3 => ",SHORT",
                            5 => ",NARROW",
                            _ => "",
                        };
                        let _ = write!(out, "Text({field}{form})");
                    }
                    'a' => {
                        let _ = write!(out, "Text({field},SHORT)");
                    }
                    'S' => {
                        let _ = write!(out, "Fraction(NanoOfSecond,{count},{count})");
                    }
                    // A two-letter year is the last two digits of a
                    // 2000-based century; more is the year, padded.
                    'y' | 'u' if count == 2 => {
                        let _ = write!(out, "ReducedValue({field},2,2,2000-01-01)");
                    }
                    // Four or more letters PADS the year; three is plain.
                    'y' | 'u' if count >= 3 => {
                        let style = if count >= 4 { "EXCEEDS_PAD" } else { "NORMAL" };
                        let _ = write!(out, "Value({field},{count},19,{style})");
                    }
                    // Two letters of a day-of-year is a RANGE (2 to 3); three
                    // is an exact width.
                    'D' if count == 2 => {
                        let _ = write!(out, "Value({field},2,3,NOT_NEGATIVE)");
                    }
                    _ if count == 1 => {
                        let _ = write!(out, "Value({field})");
                    }
                    _ => {
                        let _ = write!(out, "Value({field},{count})");
                    }
                }
            }
        }
    }
    out
}

/// The ISO formatters, which are NOT a value's `toString`: an ISO time always
/// writes its seconds (`14:05:00`), and its fraction carries only as many
/// digits as it needs (`14:05:09.5`, where `toString` writes `.500`).
#[must_use]
pub fn iso_time_text(time: Time) -> String {
    let mut out = format!(
        "{:02}:{:02}:{:02}",
        time.hour(),
        time.minute(),
        time.second()
    );
    let nano = time.nano();
    if nano != 0 {
        let mut digits = format!("{nano:09}");
        while digits.ends_with('0') {
            digits.pop();
        }
        let _ = write!(out, ".{digits}");
    }
    out
}

/// How a JDK describes each ISO formatter — read off a real one, brackets and
/// all (an optional section prints in brackets).
#[must_use]
pub fn iso_description(kind: u8) -> &'static str {
    const DATE: &str = "Value(Year,4,10,EXCEEDS_PAD)'-'Value(MonthOfYear,2)'-'Value(DayOfMonth,2)";
    const TIME: &str = "Value(HourOfDay,2)':'Value(MinuteOfHour,2)[':'Value(SecondOfMinute,2)\
                        [Fraction(NanoOfSecond,0,9,DecimalPoint)]]";
    match kind {
        0 => DATE,
        1 => TIME,
        _ => concat!(
            "Value(Year,4,10,EXCEEDS_PAD)'-'Value(MonthOfYear,2)'-'Value(DayOfMonth,2)",
            "'T'Value(HourOfDay,2)':'Value(MinuteOfHour,2)[':'Value(SecondOfMinute,2)",
            "[Fraction(NanoOfSecond,0,9,DecimalPoint)]]"
        ),
    }
}
