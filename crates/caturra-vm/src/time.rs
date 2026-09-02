//! `java.time` values: the calendar arithmetic behind `LocalDate` and the
//! two enums it answers with.
//!
//! Everything here is PURE — the proleptic Gregorian calendar, no clock, no
//! locale and no timezone database — which is what makes it exactly
//! comparable with a JDK. The parts of `java.time` that need a zone (a
//! `ZonedDateTime`, or what "today" is) ask the HOST, because the browser
//! already has the IANA database and vendoring a second copy would only add a
//! version to disagree with.

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
