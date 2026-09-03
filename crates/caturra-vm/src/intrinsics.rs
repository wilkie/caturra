//! Intrinsic ("native") classes: core library members whose semantics
//! live in Rust rather than in interpreted bytecode.
//!
//! Resolution order per `specs/SCOPE.md`: intrinsics → baked-in classlib
//! → user classes. v0 covers `java.lang.System.out`/`err` and
//! `java.io.PrintStream.print`/`println`; the surface grows with
//! `specs/LANGUAGE.md` staging.

use crate::io::ConsoleIo;
use crate::map::JavaHashMap;
use crate::unicode;
use crate::value::{
    Heap, HeapObject, HeapRef, IntKind, IteratorWrites, JValue, MapViewKind, PrintSink, StdStream,
    Temporal,
};
use crate::vfs::VirtualFileSystem;
use crate::vm::VmError;

/// Lazily allocated intrinsic singletons (`System.out`, `System.err`).
#[derive(Debug, Default)]
pub struct IntrinsicStatics {
    /// `System.out`. Public because `System.setOut` REPLACES it: the compiler
    /// routes every `System.out.println` to whatever object this holds.
    pub stdout: Option<HeapRef>,
    pub stderr: Option<HeapRef>,
    stdin: Option<HeapRef>,
}

impl IntrinsicStatics {
    /// The singletons behind `System.out`/`err`/`in`, which live as long as
    /// the run does — roots for the collector.
    pub fn roots(&self) -> impl Iterator<Item = HeapRef> + '_ {
        [self.stdout, self.stderr, self.stdin].into_iter().flatten()
    }

    /// Resolve an intrinsic static field like `java/lang/System.out`,
    /// allocating its singleton on first use.
    pub fn static_field(&mut self, heap: &mut Heap, class: &str, field: &str) -> Option<JValue> {
        match (class, field) {
            ("java/lang/System", "out") => {
                let reference = *self.stdout.get_or_insert_with(|| {
                    heap.alloc(HeapObject::PrintStream(PrintSink::Std(StdStream::Out)))
                });
                Some(JValue::Ref(Some(reference)))
            }
            ("java/lang/System", "err") => {
                let reference = *self.stderr.get_or_insert_with(|| {
                    heap.alloc(HeapObject::PrintStream(PrintSink::Std(StdStream::Err)))
                });
                Some(JValue::Ref(Some(reference)))
            }
            ("java/lang/System", "in") => {
                let reference = *self
                    .stdin
                    .get_or_insert_with(|| heap.alloc(HeapObject::InputStream));
                Some(JValue::Ref(Some(reference)))
            }
            _ => None,
        }
    }
}

/// The year range `java.time` accepts, repeated here so the clamping in
/// `withYear`/`withMonth` does not have to guess at a month length for a year
/// that will be refused anyway.
const MIN_TIME_YEAR: i32 = -999_999_999;
const MAX_TIME_YEAR: i32 = 999_999_999;

/// `java.time.DateTimeException` with this text — what every invalid date
/// operation throws.
fn date_time_exception(message: &str) -> VmError {
    VmError::UncaughtException(format!("java.time.DateTimeException: {message}"))
}

/// A `java.time` value's `hashCode`, as the JDK computes it — two equal
/// values must hash alike wherever the program looks, and a `LocalDateTime`
/// is its two halves `XOR`ed, so this has to be one function.
fn temporal_hash(value: Temporal) -> i32 {
    match value {
        Temporal::Date(date) => {
            let year = date.year;
            #[allow(clippy::cast_possible_wrap)]
            let mask = 0xFFFF_F800_u32 as i32;
            (year & mask)
                ^ (year.wrapping_shl(11) + (i32::from(date.month) << 6) + i32::from(date.day))
        }
        Temporal::Time(time) => {
            #[allow(clippy::cast_possible_truncation)]
            let folded = (time.nano_of_day ^ (time.nano_of_day >> 32)) as i32;
            folded
        }
        Temporal::DateTime(when) => {
            temporal_hash(Temporal::Date(when.date)) ^ temporal_hash(Temporal::Time(when.time))
        }
        // A `Duration` hashes its two fields the way the JDK folds them; a
        // `Period` and a unit are small enough to hash as themselves.
        Temporal::Duration(amount) => {
            #[allow(clippy::cast_possible_truncation)]
            let folded = (amount.seconds ^ (amount.seconds >> 32)) as i32;
            folded + (51 * amount.nanos)
        }
        Temporal::Period(period) => period
            .years
            .wrapping_add(period.months.wrapping_shl(8))
            .wrapping_add(period.days.wrapping_shl(16)),
        Temporal::Unit(unit) => i32::from(unit),
        Temporal::DayOfWeek(day) => i32::from(day),
        Temporal::Month(month) => i32::from(month),
        Temporal::Field(field) => i32::from(field),
        Temporal::Adjuster(adjuster) => i32::from(adjuster.day),
        Temporal::Era(era) => i32::from(era),
        // `ValueRange` folds its four numbers the way the JDK does.
        Temporal::Range(range) => {
            let fold = range.min + (range.largest_min << 16) + (range.largest_min >> 48)
                - (range.smallest_max << 32)
                - (range.smallest_max >> 32)
                + (range.max << 48)
                + (range.max >> 16);
            #[allow(clippy::cast_possible_truncation)]
            let hash = ((fold ^ (fold >> 32)) & 0xFFFF_FFFF) as i32;
            hash
        }
    }
}

/// What all three answer alike: `toString`, `name`, `equals` and `hashCode`.
/// `None` means "not one of these", so the caller carries on.
fn temporal_object_method(
    value: Temporal,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Option<JValue> {
    let other = match args.first() {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
            Some(HeapObject::Temporal(other)) => Some(*other),
            _ => None,
        },
        _ => None,
    };
    let answer = match method {
        "toString" => JValue::Ref(Some(heap.alloc_string(&value.text()))),
        // An enum's `name()` is the CONSTANT — which for a `ChronoUnit` is
        // not its `toString`: `ChronoUnit.DAYS.name()` is "DAYS" and its text
        // is "Days".
        "name" if matches!(value, Temporal::DayOfWeek(_) | Temporal::Month(_)) => {
            JValue::Ref(Some(heap.alloc_string(&value.text())))
        }
        // A `ChronoUnit`'s and a `ChronoField`'s name is the CONSTANT, which
        // their text is not: `HALF_DAYS` prints as "HalfDays", and
        // `NANO_OF_SECOND` as "NanoOfSecond".
        "name" => match value {
            Temporal::Era(_) => JValue::Ref(Some(heap.alloc_string(&value.text()))),
            Temporal::Unit(unit) => JValue::Ref(Some(
                heap.alloc_string(crate::time::UNIT_CONSTANTS[usize::from(unit)]),
            )),
            Temporal::Field(field) => JValue::Ref(Some(
                heap.alloc_string(crate::time::field_info(field).constant),
            )),
            _ => return None,
        },
        "equals" => JValue::Int(i32::from(other == Some(value))),
        "hashCode" => JValue::Int(temporal_hash(value)),
        _ => return None,
    };
    Some(answer)
}

/// `DayOfWeek` and `Month`: enums, so a value, a name, an ordinal and an
/// ordering. Their shared `equals`/`hashCode`/`toString` are handled with the
/// date's, above.
fn temporal_enum_method(
    value: Temporal,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let int = |value: i64| Ok(Some(JValue::Int(i32::try_from(value).unwrap_or(i32::MAX))));
    let other = match args.first() {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
            Some(HeapObject::Temporal(other)) => Some(*other),
            _ => None,
        },
        _ => None,
    };
    match (value, method) {
        (Temporal::DayOfWeek(day), "getValue") => int(i64::from(day)),
        (Temporal::Month(month), "getValue") => int(i64::from(month)),
        (Temporal::DayOfWeek(day), "ordinal") => int(i64::from(day) - 1),
        (Temporal::Month(month), "ordinal") => int(i64::from(month) - 1),
        (Temporal::DayOfWeek(day), "compareTo") => match other {
            Some(Temporal::DayOfWeek(against)) => int(i64::from(day) - i64::from(against)),
            _ => Err(throw("java.lang.ClassCastException: not a DayOfWeek")),
        },
        (Temporal::Month(month), "compareTo") => match other {
            Some(Temporal::Month(against)) => int(i64::from(month) - i64::from(against)),
            _ => Err(throw("java.lang.ClassCastException: not a Month")),
        },
        // An enum ROTATES: Saturday plus three days is Tuesday.
        (_, "plus" | "minus") => {
            let by = match args.first() {
                Some(JValue::Long(value)) => *value,
                Some(JValue::Int(value)) => i64::from(*value),
                _ => 0,
            };
            let by = if method == "minus" { -by } else { by };
            let (current, size) = match value {
                Temporal::DayOfWeek(day) => (i64::from(day), 7),
                Temporal::Month(month) => (i64::from(month), 12),
                _ => {
                    return Err(VmError::UnknownIntrinsic(format!(
                        "{}.{method}",
                        value.class_name()
                    )));
                }
            };
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let rotated = ((current - 1 + by).rem_euclid(size) + 1) as u8;
            let made = match value {
                Temporal::DayOfWeek(_) => Temporal::DayOfWeek(rotated),
                _ => Temporal::Month(rotated),
            };
            Ok(Some(JValue::Ref(Some(heap.intern_temporal(made)))))
        }
        // The quarter a month is in starts at January, April, July or
        // October.
        (Temporal::Month(month), "firstMonthOfQuarter") => {
            let first = (month - 1) / 3 * 3 + 1;
            Ok(Some(JValue::Ref(Some(
                heap.intern_temporal(Temporal::Month(first)),
            ))))
        }
        (Temporal::Month(month), "maxLength") => {
            int(i64::from(crate::time::length_of_month(2024, month)))
        }
        (Temporal::Month(month), "minLength") => {
            int(i64::from(crate::time::length_of_month(2023, month)))
        }
        // `firstDayOfYear(leapYear)` — the day-of-year this month starts on.
        (Temporal::Month(month), "firstDayOfYear") => {
            let leap = matches!(args.first(), Some(JValue::Int(1)));
            let year = if leap { 2024 } else { 2023 };
            let start = crate::time::Date {
                year,
                month,
                day: 1,
            };
            int(i64::from(start.day_of_year()))
        }
        // `Month.length(boolean leapYear)`.
        (Temporal::Month(month), "length") => {
            let leap = matches!(args.first(), Some(JValue::Int(1)));
            let year = if leap { 2024 } else { 2023 };
            int(i64::from(crate::time::length_of_month(year, month)))
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{}.{method}",
            value.class_name()
        ))),
    }
}

/// `LocalDateTime`'s factories: from fields, from a date and a time, from
/// text, and from the host's clock.
fn local_date_time_static(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let made = |heap: &mut Heap, when: crate::time::DateTime| {
        Ok(Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::DateTime(when)),
        ))))
    };
    match method {
        "of" => {
            // Either five-to-seven numbers, or a `LocalDate` and a
            // `LocalTime` — the two shapes a program writes.
            if let [JValue::Ref(Some(date_ref)), JValue::Ref(Some(time_ref))] = args {
                let (
                    Some(HeapObject::Temporal(Temporal::Date(date))),
                    Some(HeapObject::Temporal(Temporal::Time(time))),
                ) = (heap.get(*date_ref), heap.get(*time_ref))
                else {
                    return Err(throw("java.lang.ClassCastException: not a date and a time"));
                };
                let when = crate::time::DateTime {
                    date: *date,
                    time: *time,
                };
                return made(heap, when);
            }
            let field = |at: usize| match args.get(at) {
                Some(JValue::Int(value)) => *value,
                _ => 0,
            };
            let date = match crate::time::Date::of(field(0), field(1), field(2)) {
                Ok(date) => date,
                Err(message) => return Err(date_time_exception(&message)),
            };
            let time = match crate::time::Time::of(field(3), field(4), field(5), field(6)) {
                Ok(time) => time,
                Err(message) => return Err(date_time_exception(&message)),
            };
            made(heap, crate::time::DateTime { date, time })
        }
        "parse" => {
            let text = match args.first() {
                Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                Some(JValue::Ref(None)) => {
                    return Err(throw("java.lang.NullPointerException: text"));
                }
                _ => None,
            }
            .unwrap_or_default();
            if let Some(JValue::Ref(Some(formatter))) = args.get(1) {
                let formatter = *formatter;
                return parse_with_formatter(heap, "java/time/LocalDateTime", &text, formatter);
            }
            match crate::time::parse_date_time(&text) {
                Ok(when) => made(heap, when),
                Err(message) => Err(VmError::UncaughtException(format!(
                    "java.time.format.DateTimeParseException: {message}"
                ))),
            }
        }
        "now" => {
            let offset = i64::from(console.zone_offset_seconds()) * 1000;
            let local = console.now_millis().saturating_add(offset);
            let when = crate::time::DateTime {
                date: crate::time::Date::from_epoch_day(local.div_euclid(86_400_000)),
                time: crate::time::Time {
                    nano_of_day: local.rem_euclid(86_400_000) * 1_000_000,
                },
            };
            made(heap, when)
        }
        "from" => temporal_from(heap, args, true),
        _ => Err(VmError::UnknownIntrinsic(format!(
            "java/time/LocalDateTime.{method}"
        ))),
    }
}

/// The unit a `plusX`/`minusX` name asks for, as nanoseconds — `None` when
/// the unit is one only a DATE has (days and up are handled by the date).
fn nanos_per_unit(unit: &str) -> Option<i64> {
    Some(match unit {
        "Nanos" => 1,
        "Seconds" => crate::time::NANOS_PER_SECOND,
        "Minutes" => crate::time::NANOS_PER_MINUTE,
        "Hours" => crate::time::NANOS_PER_HOUR,
        _ => return None,
    })
}

/// The amount a one-argument `plusX`/`minusX` was given, signed by which of
/// the two it was, with the unit's name.
fn shift_by<'a>(method: &'a str, args: &[JValue]) -> Option<(&'a str, i64)> {
    let plus = method.starts_with("plus");
    if !plus && !method.starts_with("minus") {
        return None;
    }
    let amount = match args.first() {
        Some(JValue::Int(n)) => i64::from(*n),
        Some(JValue::Long(n)) => *n,
        _ => 0,
    };
    let unit = if plus {
        method.trim_start_matches("plus")
    } else {
        method.trim_start_matches("minus")
    };
    Some((unit, if plus { amount } else { -amount }))
}

/// `truncatedTo(unit)` — everything below the unit becomes zero. A unit
/// bigger than a day cannot truncate a time of day, and `java.time` says so.
fn truncate_time(
    time: crate::time::Time,
    heap: &mut Heap,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let Some(JValue::Ref(Some(reference))) = args.first() else {
        return Err(throw("java.lang.NullPointerException: unit"));
    };
    let Some(HeapObject::Temporal(Temporal::Unit(unit))) = heap.get(*reference) else {
        return Err(throw("java.lang.ClassCastException: not a ChronoUnit"));
    };
    // A unit larger than a day cannot truncate a time of day, and neither
    // can one that does not divide a day evenly.
    let Some(step) =
        crate::time::unit_nanos(*unit).filter(|step| *step <= crate::time::NANOS_PER_DAY)
    else {
        // The JDK names no unit here — the sentence is the whole message.
        return Err(VmError::UncaughtException(String::from(
            "java.time.temporal.UnsupportedTemporalTypeException: Unit is too large to be \
             used for truncation",
        )));
    };
    let truncated = crate::time::Time {
        nano_of_day: time.nano_of_day - time.nano_of_day % step,
    };
    Ok(Some(JValue::Ref(Some(
        heap.intern_temporal(Temporal::Time(truncated)),
    ))))
}

/// `java.time.LocalTime`: a time of day. Everything it answers is derived
/// from the one number it holds.
fn time_method(
    time: crate::time::Time,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let int = |value: i64| Ok(Some(JValue::Int(i32::try_from(value).unwrap_or(i32::MAX))));
    let made = |heap: &mut Heap, time: crate::time::Time| {
        Ok(Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::Time(time)),
        ))))
    };
    let other = match args.first() {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
            Some(HeapObject::Temporal(Temporal::Time(other))) => Some(*other),
            _ => None,
        },
        _ => None,
    };
    match method {
        "getHour" => int(i64::from(time.hour())),
        "getMinute" => int(i64::from(time.minute())),
        "getSecond" => int(i64::from(time.second())),
        "getNano" => int(i64::from(time.nano())),
        "toSecondOfDay" => int(time.nano_of_day / crate::time::NANOS_PER_SECOND),
        "toNanoOfDay" => Ok(Some(JValue::Long(time.nano_of_day))),
        "withHour" | "withMinute" | "withSecond" | "withNano" => {
            #[allow(clippy::cast_possible_truncation)]
            let with = match args.first() {
                Some(JValue::Int(n)) => *n,
                Some(JValue::Long(n)) => *n as i32,
                _ => 0,
            };
            let (hour, minute, second, nano) = match method {
                "withHour" => (with, time.minute(), time.second(), time.nano()),
                "withMinute" => (time.hour(), with, time.second(), time.nano()),
                "withSecond" => (time.hour(), time.minute(), with, time.nano()),
                _ => (time.hour(), time.minute(), time.second(), with),
            };
            match crate::time::Time::of(hour, minute, second, nano) {
                Ok(made_time) => made(heap, made_time),
                Err(message) => Err(date_time_exception(&message)),
            }
        }
        // `atDate(date)` — the other way of building a date-time.
        "atDate" => match args.first() {
            Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                Some(HeapObject::Temporal(Temporal::Date(date))) => {
                    let when = crate::time::DateTime { date: *date, time };
                    Ok(Some(JValue::Ref(Some(
                        heap.intern_temporal(Temporal::DateTime(when)),
                    ))))
                }
                _ => Err(throw("java.lang.ClassCastException: not a LocalDate")),
            },
            _ => Err(throw("java.lang.NullPointerException: date")),
        },
        "truncatedTo" => truncate_time(time, heap, args),
        "isBefore" | "isAfter" | "compareTo" => {
            let Some(against) = other else {
                return Err(throw("java.lang.ClassCastException: not a LocalTime"));
            };
            // `LocalTime.compareTo` is `Integer.compare` field by field, so
            // -1/0/1 — a `LocalDate`'s is the raw difference, and the two
            // really do differ.
            if method == "compareTo" {
                return int(i64::from(
                    match time.nano_of_day.cmp(&against.nano_of_day) {
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    },
                ));
            }
            let answer = if method == "isBefore" {
                time < against
            } else {
                time > against
            };
            Ok(Some(JValue::Int(i32::from(answer))))
        }
        _ => match shift_by(method, args) {
            // A time WRAPS at midnight rather than carrying: `23:00` plus two
            // hours is `01:00`, and nothing about the day is remembered.
            Some((unit, amount)) => match nanos_per_unit(unit) {
                Some(nanos) => made(heap, time.plus_nanos(amount.saturating_mul(nanos))),
                None => Err(VmError::UnknownIntrinsic(format!(
                    "java/time/LocalTime.{method}"
                ))),
            },
            None => Err(VmError::UnknownIntrinsic(format!(
                "java/time/LocalTime.{method}"
            ))),
        },
    }
}

/// `LocalDate.atTime` and the `withX` family — everything that builds a NEW
/// date (or date-time) out of one, and validates as `of` does.
fn date_builder(
    date: crate::time::Date,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let number = || match args.first() {
        Some(JValue::Int(n)) => i64::from(*n),
        Some(JValue::Long(n)) => *n,
        _ => 0,
    };
    match method {
        // `withDayOfYear(n)` — the n-th day of the same year.
        "withDayOfYear" => {
            let wanted = number();
            let length = i64::from(date.length_of_year());
            if !(1..=length).contains(&wanted) {
                return Err(date_time_exception(&format!(
                    "Invalid value for DayOfYear (valid values 1 - 365/366): {wanted}"
                )));
            }
            let start = crate::time::Date {
                year: date.year,
                month: 1,
                day: 1,
            };
            Ok(Some(JValue::Ref(Some(heap.intern_temporal(
                Temporal::Date(start.plus_days(wanted - 1)),
            )))))
        }
        "atTime" => {
            let field = |at: usize| match args.get(at) {
                Some(JValue::Int(value)) => *value,
                _ => 0,
            };
            match crate::time::Time::of(field(0), field(1), field(2), field(3)) {
                Ok(time) => Ok(Some(JValue::Ref(Some(heap.intern_temporal(
                    Temporal::DateTime(crate::time::DateTime { date, time }),
                ))))),
                Err(message) => Err(date_time_exception(&message)),
            }
        }
        "withYear" | "withMonth" | "withDayOfMonth" => {
            #[allow(clippy::cast_possible_truncation)]
            let with = number() as i32;
            // `withYear`/`withMonth` CLAMP the day (the JDK's
            // `resolvePreviousValid`, so `Jan 31` with month February is
            // `Feb 29`); only `withDayOfMonth` is strict about it.
            let (year, month, day) = match method {
                "withYear" => (with, i32::from(date.month), i32::from(date.day)),
                "withMonth" => (date.year, with, i32::from(date.day)),
                _ => (date.year, i32::from(date.month), with),
            };
            let day = if method == "withDayOfMonth" {
                day
            } else if (1..=12).contains(&month) && (MIN_TIME_YEAR..=MAX_TIME_YEAR).contains(&year) {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let length = i32::from(crate::time::length_of_month(year, month as u8));
                day.min(length)
            } else {
                day
            };
            match date.with(year, month, day) {
                Ok(made) => Ok(Some(JValue::Ref(Some(
                    heap.intern_temporal(Temporal::Date(made)),
                )))),
                Err(message) => Err(date_time_exception(&message)),
            }
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "java/time/LocalDate.{method}"
        ))),
    }
}

/// `LocalDateTime.withX` — the half that owns the field answers, and the
/// other half is carried over unchanged.
fn date_time_with(
    when: crate::time::DateTime,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let unknown = || VmError::UnknownIntrinsic(format!("java/time/LocalDateTime.{method}"));
    let half = if matches!(
        method,
        "withHour" | "withMinute" | "withSecond" | "withNano"
    ) {
        time_method(when.time, heap, method, args)?
    } else {
        temporal_method(Temporal::Date(when.date), heap, method, args)?
    };
    let Some(JValue::Ref(Some(reference))) = half else {
        return Err(unknown());
    };
    let made = match heap.get(reference) {
        Some(HeapObject::Temporal(Temporal::Time(time))) => crate::time::DateTime {
            date: when.date,
            time: *time,
        },
        Some(HeapObject::Temporal(Temporal::Date(date))) => crate::time::DateTime {
            date: *date,
            time: when.time,
        },
        _ => return Err(unknown()),
    };
    Ok(Some(JValue::Ref(Some(
        heap.intern_temporal(Temporal::DateTime(made)),
    ))))
}

/// How two date-times compare. `compareTo` answers the DATE's difference and
/// only reaches the time on a tie, which is how `java.time` writes it.
fn compare_date_times(
    when: crate::time::DateTime,
    against: crate::time::DateTime,
    method: &str,
) -> JValue {
    if method == "compareTo" {
        let date_order = date_difference(when.date, against.date);
        let order = if date_order == 0 {
            i64::from(when.time.nano_of_day.cmp(&against.time.nano_of_day) as i8)
        } else {
            date_order
        };
        return JValue::Int(i32::try_from(order).unwrap_or(i32::MAX));
    }
    let answer = match method {
        "isBefore" => when < against,
        "isAfter" => when > against,
        _ => when == against,
    };
    JValue::Int(i32::from(answer))
}

/// `java.time.LocalDateTime`: a date and a time. Its readers are its halves'
/// readers, and its arithmetic carries whole days from one into the other.
fn date_time_method(
    when: crate::time::DateTime,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let made = |heap: &mut Heap, when: crate::time::DateTime| {
        Ok(Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::DateTime(when)),
        ))))
    };
    let other = match args.first() {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
            Some(HeapObject::Temporal(Temporal::DateTime(other))) => Some(*other),
            _ => None,
        },
        _ => None,
    };
    match method {
        // A date-time truncates its TIME half, and `DAYS` empties it.
        "truncatedTo" => {
            let truncated = time_method(when.time, heap, method, args)?;
            let Some(JValue::Ref(Some(reference))) = truncated else {
                return Err(VmError::UnknownIntrinsic(String::from(
                    "java/time/LocalDateTime.truncatedTo",
                )));
            };
            let Some(HeapObject::Temporal(Temporal::Time(time))) = heap.get(reference) else {
                return Err(VmError::UnknownIntrinsic(String::from(
                    "java/time/LocalDateTime.truncatedTo",
                )));
            };
            let time = *time;
            made(
                heap,
                crate::time::DateTime {
                    date: when.date,
                    time,
                },
            )
        }
        "toLocalDate" => Ok(Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::Date(when.date)),
        )))),
        "toLocalTime" => Ok(Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::Time(when.time)),
        )))),
        "isBefore" | "isAfter" | "isEqual" | "compareTo" => {
            let Some(against) = other else {
                return Err(throw("java.lang.ClassCastException: not a LocalDateTime"));
            };
            Ok(Some(compare_date_times(when, against, method)))
        }
        _ => {
            // A reader the DATE or the TIME answers: ask them, on the halves.
            if let Some(answer) = date_reader(Temporal::Date(when.date), heap, method) {
                return Ok(Some(answer));
            }
            if matches!(
                method,
                "getHour" | "getMinute" | "getSecond" | "getNano" | "toSecondOfDay"
            ) {
                return time_method(when.time, heap, method, args);
            }
            if method.starts_with("with") {
                return date_time_with(when, heap, method, args);
            }
            let Some((unit, amount)) = shift_by(method, args) else {
                return Err(VmError::UnknownIntrinsic(format!(
                    "java/time/LocalDateTime.{method}"
                )));
            };
            // A time unit carries into the date; a date unit moves the date
            // and leaves the time alone.
            if let Some(nanos) = nanos_per_unit(unit) {
                return made(heap, when.plus_nanos(amount.saturating_mul(nanos)));
            }
            let moved = match unit {
                "Days" => when.date.plus_days(amount),
                "Weeks" => when.date.plus_days(amount.saturating_mul(7)),
                "Months" => when.date.plus_months(amount),
                "Years" => when.date.plus_years(amount),
                _ => {
                    return Err(VmError::UnknownIntrinsic(format!(
                        "java/time/LocalDateTime.{method}"
                    )));
                }
            };
            made(
                heap,
                crate::time::DateTime {
                    date: moved,
                    time: when.time,
                },
            )
        }
    }
}

/// The pattern a formatter reference holds, and the pieces it parses to.
/// `Ok(None)` is one of the ISO constants, which format as the value's own
/// `toString` does.
fn formatter_pieces(
    heap: &Heap,
    reference: HeapRef,
) -> Result<Result<Vec<crate::time::Piece>, u8>, VmError> {
    let Some(HeapObject::DateFormat(kind)) = heap.get(reference) else {
        return Err(throw(
            "java.lang.ClassCastException: not a DateTimeFormatter",
        ));
    };
    let pattern = match kind {
        crate::value::DateFormatKind::Iso(which) => return Ok(Err(*which)),
        crate::value::DateFormatKind::Pattern(pattern) => pattern,
    };
    match crate::time::parse_pattern(pattern) {
        Ok(pieces) => Ok(Ok(pieces)),
        Err(message) => Err(throw(&*format!(
            "java.lang.IllegalArgumentException: {message}"
        ))),
    }
}

/// `value.format(formatter)` — and `formatter.format(value)`, which is the
/// same thing written the other way round.
fn format_temporal(
    value: Temporal,
    heap: &mut Heap,
    formatter: HeapRef,
) -> Result<Option<JValue>, VmError> {
    let (date, time) = match value {
        Temporal::Date(date) => (Some(date), None),
        Temporal::Time(time) => (None, Some(time)),
        Temporal::DateTime(when) => (Some(when.date), Some(when.time)),
        _ => return Err(throw("java.lang.ClassCastException: not a date or a time")),
    };
    let text = match formatter_pieces(heap, formatter)? {
        // An ISO constant is NOT the value's `toString`: its time half
        // always writes the seconds, and its fraction carries only as many
        // digits as it needs.
        Err(_) => match (date, time) {
            (Some(date), None) => date.to_string(),
            (None, Some(time)) => crate::time::iso_time_text(time),
            (Some(date), Some(time)) => format!("{date}T{}", crate::time::iso_time_text(time)),
            _ => value.text(),
        },
        Ok(pieces) => match crate::time::format_pieces(&pieces, date, time) {
            Ok(text) => text,
            Err(crate::time::FormatFail::Field(field)) => {
                return Err(VmError::UncaughtException(format!(
                    "java.time.temporal.UnsupportedTemporalTypeException: Unsupported field: \
                     {field}"
                )));
            }
            // A ZONE is not a missing field but a missing zone, and a JDK
            // names the value it could not find one in.
            Err(crate::time::FormatFail::Zone) => {
                return Err(VmError::UncaughtException(format!(
                    "java.time.DateTimeException: Unable to extract ZoneId from temporal {}",
                    value.text()
                )));
            }
        },
    };
    Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
}

/// `LocalDate.parse(text, formatter)` and its two siblings. `want` is which
/// of the three asked, so that a missing field is that type's own complaint.
fn parse_with_formatter(
    heap: &mut Heap,
    want: &str,
    text: &str,
    formatter: HeapRef,
) -> Result<Option<JValue>, VmError> {
    // The JDK's two shapes: "could not be parsed at index 4" when the text
    // does not fit the pattern, and "could not be parsed: Invalid value for
    // MonthOfYear …" when it fits but says something impossible.
    let failed = |why: &str| {
        VmError::UncaughtException(format!(
            "java.time.format.DateTimeParseException: Text '{text}' could not be parsed{why}"
        ))
    };
    let built = match formatter_pieces(heap, formatter)? {
        Err(_) => match want {
            "java/time/LocalDate" => crate::time::parse_date(text)
                .map(|date| (Some(date), None))
                .map_err(|_| failed(" at index 0"))?,
            "java/time/LocalTime" => crate::time::parse_time(text)
                .map(|time| (None, Some(time)))
                .map_err(|_| failed(" at index 0"))?,
            _ => crate::time::parse_date_time(text)
                .map(|when| (Some(when.date), Some(when.time)))
                .map_err(|_| failed(" at index 0"))?,
        },
        Ok(pieces) => {
            crate::time::parse_pieces(&pieces, text).map_err(|why| failed(why.as_str()))?
        }
    };
    let value = match (want, built) {
        ("java/time/LocalDate", (Some(date), _)) => Temporal::Date(date),
        ("java/time/LocalTime", (_, Some(time))) => Temporal::Time(time),
        ("java/time/LocalDateTime", (Some(date), Some(time))) => {
            Temporal::DateTime(crate::time::DateTime { date, time })
        }
        // The text PARSED, but not into what was asked for — a date pattern
        // read into a `LocalTime`. The JDK says which type it wanted and what
        // it resolved instead, which is a different sentence from "the text
        // does not fit".
        (_, (date, time)) => {
            // The value's OWN text, not the ISO formatter's: a time with no
            // seconds prints as "13:45" here, where ISO writes "13:45:00".
            let resolved = match (date, time) {
                (Some(date), Some(time)) => format!("{date}T{time}"),
                (Some(date), None) => date.to_string(),
                (None, Some(time)) => time.to_string(),
                (None, None) => String::new(),
            };
            return Err(failed(&format!(
                ": Unable to obtain {} from TemporalAccessor: {{}},ISO resolved to {resolved} \
                 of type java.time.format.Parsed",
                want.rsplit('/').next().unwrap_or(want)
            )));
        }
    };
    Ok(Some(JValue::Ref(Some(heap.intern_temporal(value)))))
}

/// What a `Duration` answers about itself: the whole of it in one unit, the
/// PART of it in one unit, and whether it is nothing or less than nothing.
fn duration_reader(amount: crate::time::Duration, method: &str) -> Option<JValue> {
    let total = amount.total_nanos();
    let part = |unit: char| {
        Some(JValue::Int(
            i32::try_from(amount.part(unit)).unwrap_or(i32::MAX),
        ))
    };
    match method {
        // Every `toX` TRUNCATES toward zero, as `java.time`'s do.
        "toDays" => Some(JValue::Long(total / crate::time::NANOS_PER_DAY)),
        "toHours" => Some(JValue::Long(total / crate::time::NANOS_PER_HOUR)),
        "toMinutes" => Some(JValue::Long(total / crate::time::NANOS_PER_MINUTE)),
        "getSeconds" | "toSeconds" => Some(JValue::Long(amount.seconds)),
        "toMillis" => Some(JValue::Long(total / 1_000_000)),
        "toNanos" => Some(JValue::Long(total)),
        "getNano" => Some(JValue::Int(amount.nanos)),
        "isZero" => Some(JValue::Int(i32::from(amount.is_zero()))),
        "isNegative" => Some(JValue::Int(i32::from(amount.is_negative()))),
        "toHoursPart" => part('H'),
        "toMinutesPart" => part('M'),
        "toSecondsPart" => part('S'),
        "toMillisPart" => part('m'),
        "toNanosPart" => part('n'),
        _ => None,
    }
}

/// `multipliedBy`, `dividedBy` and the two `withX` — the operations that
/// answer a NEW duration built from this one's numbers.
fn duration_rebuilt(
    amount: crate::time::Duration,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let made = |heap: &mut Heap, amount: crate::time::Duration| {
        Ok(Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::Duration(amount)),
        ))))
    };
    let total = amount.total_nanos();
    match method {
        "multipliedBy" | "dividedBy" => {
            let by = match args.first() {
                Some(JValue::Int(value)) => i64::from(*value),
                Some(JValue::Long(value)) => *value,
                _ => 1,
            };
            if method == "dividedBy" && by == 0 {
                return Err(throw("java.lang.ArithmeticException: / by zero"));
            }
            let moved = if method == "multipliedBy" {
                total.saturating_mul(by)
            } else {
                total / by
            };
            made(heap, crate::time::Duration::of_nanos(moved))
        }
        "withSeconds" => {
            let seconds = match args.first() {
                Some(JValue::Long(value)) => *value,
                Some(JValue::Int(value)) => i64::from(*value),
                _ => 0,
            };
            made(
                heap,
                crate::time::Duration {
                    seconds,
                    nanos: amount.nanos,
                },
            )
        }
        "withNanos" => {
            let nanos = match args.first() {
                Some(JValue::Int(value)) => *value,
                _ => 0,
            };
            made(
                heap,
                crate::time::Duration {
                    seconds: amount.seconds,
                    nanos,
                },
            )
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "java/time/Duration.{method}"
        ))),
    }
}

/// `java.time.Duration`: an amount of time, and the units it can be read in.
#[allow(clippy::too_many_lines)] // one arm per method
fn duration_method(
    amount: crate::time::Duration,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let made = |heap: &mut Heap, amount: crate::time::Duration| {
        Ok(Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::Duration(amount)),
        ))))
    };
    let other = match args.first() {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
            Some(HeapObject::Temporal(Temporal::Duration(other))) => Some(*other),
            _ => None,
        },
        _ => None,
    };
    let total = amount.total_nanos();
    match method {
        _ if duration_reader(amount, method).is_some() => Ok(duration_reader(amount, method)),
        "compareTo" => match other {
            Some(against) => Ok(Some(JValue::Int(match total.cmp(&against.total_nanos()) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            }))),
            None => Err(throw("java.lang.ClassCastException: not a Duration")),
        },
        "negated" => made(heap, crate::time::Duration::of_nanos(-total)),
        "toDaysPart" => Ok(Some(JValue::Long(total / crate::time::NANOS_PER_DAY))),
        // `truncatedTo` on a Duration is truncation TOWARDS ZERO, not the
        // floor a time of day gets.
        "truncatedTo" => {
            let Some(unit) = unit_argument(heap, args.first()) else {
                return Err(throw("java.lang.ClassCastException: not a ChronoUnit"));
            };
            let Some(step) =
                crate::time::unit_nanos(unit).filter(|step| *step <= crate::time::NANOS_PER_DAY)
            else {
                return Err(VmError::UncaughtException(String::from(
                    "java.time.temporal.UnsupportedTemporalTypeException: Unit is too large \
                     to be used for truncation",
                )));
            };
            made(heap, crate::time::Duration::of_nanos(total - total % step))
        }
        // A `Duration` holds exactly two units, and answers only those.
        "get" => {
            let Some(unit) = unit_argument(heap, args.first()) else {
                return Err(throw("java.lang.ClassCastException: not a ChronoUnit"));
            };
            match unit {
                3 => Ok(Some(JValue::Long(amount.seconds))),
                0 => Ok(Some(JValue::Long(i64::from(amount.nanos)))),
                _ => Err(VmError::UncaughtException(format!(
                    "java.time.temporal.UnsupportedTemporalTypeException: Unsupported unit: {}",
                    crate::time::unit_name(unit)
                ))),
            }
        }
        "getUnits" => {
            let units: Vec<JValue> = [3u8, 0]
                .into_iter()
                .map(|unit| JValue::Ref(Some(heap.intern_temporal(Temporal::Unit(unit)))))
                .collect();
            let list = heap.alloc(HeapObject::ArrayList(units));
            Ok(Some(JValue::Ref(Some(list))))
        }
        "toHoursPart" => Ok(Some(JValue::Int(
            i32::try_from(amount.part('H')).unwrap_or(i32::MAX),
        ))),
        "toMinutesPart" => Ok(Some(JValue::Int(
            i32::try_from(amount.part('M')).unwrap_or(i32::MAX),
        ))),
        "toSecondsPart" => Ok(Some(JValue::Int(
            i32::try_from(amount.part('S')).unwrap_or(i32::MAX),
        ))),
        "toMillisPart" => Ok(Some(JValue::Int(
            i32::try_from(amount.part('m')).unwrap_or(i32::MAX),
        ))),
        "toNanosPart" => Ok(Some(JValue::Int(
            i32::try_from(amount.part('n')).unwrap_or(i32::MAX),
        ))),
        "multipliedBy" | "dividedBy" | "withSeconds" | "withNanos" => {
            duration_rebuilt(amount, heap, method, args)
        }
        _ => {
            let Some((unit, count)) = shift_by(method, args) else {
                return Err(VmError::UnknownIntrinsic(format!(
                    "java/time/Duration.{method}"
                )));
            };
            let nanos = match unit {
                "Days" => crate::time::NANOS_PER_DAY,
                "Hours" => crate::time::NANOS_PER_HOUR,
                "Minutes" => crate::time::NANOS_PER_MINUTE,
                "Seconds" => crate::time::NANOS_PER_SECOND,
                "Millis" => 1_000_000,
                "Nanos" => 1,
                _ => {
                    return Err(VmError::UnknownIntrinsic(format!(
                        "java/time/Duration.{method}"
                    )));
                }
            };
            made(
                heap,
                crate::time::Duration::of_nanos(total + count.saturating_mul(nanos)),
            )
        }
    }
}

/// `java.time.Period`: years, months and days, each as written.
#[allow(clippy::too_many_lines)] // one arm per method
fn period_method(
    period: crate::time::Period,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let count = match args.first() {
        Some(JValue::Int(value)) => *value,
        Some(JValue::Long(value)) => i32::try_from(*value).unwrap_or(i32::MAX),
        _ => 0,
    };
    let made = |heap: &mut Heap, period: crate::time::Period| {
        Ok(Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::Period(period)),
        ))))
    };
    // `plus(period)` and `minus(period)` add the three fields separately —
    // a period is not a length, so there is nothing to normalize.
    if matches!(method, "plus" | "minus")
        && let Some(JValue::Ref(Some(reference))) = args.first()
        && let Some(HeapObject::Temporal(Temporal::Period(other))) = heap.get(*reference)
    {
        let sign = if method == "minus" { -1 } else { 1 };
        return made(
            heap,
            crate::time::Period {
                years: period.years + other.years * sign,
                months: period.months + other.months * sign,
                days: period.days + other.days * sign,
            },
        );
    }
    // A `Period` holds exactly three units, and answers only those.
    if method == "get" {
        let Some(unit) = unit_argument(heap, args.first()) else {
            return Err(throw("java.lang.ClassCastException: not a ChronoUnit"));
        };
        return match unit {
            10 => Ok(Some(JValue::Long(i64::from(period.years)))),
            9 => Ok(Some(JValue::Long(i64::from(period.months)))),
            7 => Ok(Some(JValue::Long(i64::from(period.days)))),
            _ => Err(VmError::UncaughtException(format!(
                "java.time.temporal.UnsupportedTemporalTypeException: Unsupported unit: {}",
                crate::time::unit_name(unit)
            ))),
        };
    }
    if method == "getUnits" {
        let units: Vec<JValue> = [10u8, 9, 7]
            .into_iter()
            .map(|unit| JValue::Ref(Some(heap.intern_temporal(Temporal::Unit(unit)))))
            .collect();
        let list = heap.alloc(HeapObject::ArrayList(units));
        return Ok(Some(JValue::Ref(Some(list))));
    }
    // The whole plus/minus/with family, each touching one field.
    if let Some(field) = method
        .strip_prefix("plus")
        .or_else(|| method.strip_prefix("minus"))
        .or_else(|| method.strip_prefix("with"))
    {
        let signed = if method.starts_with("minus") {
            -count
        } else {
            count
        };
        let replace = method.starts_with("with");
        let mut built = period;
        match field {
            "Years" => {
                built.years = if replace {
                    count
                } else {
                    period.years + signed
                }
            }
            "Months" => {
                built.months = if replace {
                    count
                } else {
                    period.months + signed
                }
            }
            "Days" => built.days = if replace { count } else { period.days + signed },
            _ => {
                return Err(VmError::UnknownIntrinsic(format!(
                    "java/time/Period.{method}"
                )));
            }
        }
        return made(heap, built);
    }
    match method {
        "multipliedBy" => made(
            heap,
            crate::time::Period {
                years: period.years.saturating_mul(count),
                months: period.months.saturating_mul(count),
                days: period.days.saturating_mul(count),
            },
        ),
        "negated" => made(
            heap,
            crate::time::Period {
                years: -period.years,
                months: -period.months,
                days: -period.days,
            },
        ),
        "normalized" => made(heap, period.normalized()),
        "getYears" => Ok(Some(JValue::Int(period.years))),
        "getMonths" => Ok(Some(JValue::Int(period.months))),
        "getDays" => Ok(Some(JValue::Int(period.days))),
        "toTotalMonths" => Ok(Some(JValue::Long(period.total_months()))),
        "isZero" => Ok(Some(JValue::Int(i32::from(period.is_zero())))),
        // A period is negative when ANY part is: `P1Y-2D` is.
        "isNegative" => Ok(Some(JValue::Int(i32::from(
            period.years < 0 || period.months < 0 || period.days < 0,
        )))),
        _ => Err(VmError::UnknownIntrinsic(format!(
            "java/time/Period.{method}"
        ))),
    }
}

/// `ChronoUnit.X.between(start, end)` — the only thing a unit is asked, and
/// the reason a program names one at all.
/// `java.time.temporal.ValueRange` — four numbers and the questions asked of
/// them.
fn range_method(
    range: crate::time::ValueRange,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let number = || match args.first() {
        Some(JValue::Long(v)) => *v,
        Some(JValue::Int(v)) => i64::from(*v),
        _ => 0,
    };
    Ok(Some(match method {
        "getMinimum" => JValue::Long(range.min),
        "getLargestMinimum" => JValue::Long(range.largest_min),
        "getSmallestMaximum" => JValue::Long(range.smallest_max),
        "getMaximum" => JValue::Long(range.max),
        "isFixed" => JValue::Int(i32::from(range.is_fixed())),
        "isValidValue" => JValue::Int(i32::from(range.contains(number()))),
        "isIntValue" => JValue::Int(i32::from(
            i32::try_from(range.min).is_ok() && i32::try_from(range.max).is_ok(),
        )),
        "isValidIntValue" => JValue::Int(i32::from(
            i32::try_from(range.min).is_ok()
                && i32::try_from(range.max).is_ok()
                && range.contains(number()),
        )),
        _ => {
            return Err(VmError::UnknownIntrinsic(format!(
                "java/time/temporal/ValueRange.{method}"
            )));
        }
    }))
}

/// `java.time.temporal.ChronoField` — what a field IS, before any date is
/// asked about it.
fn field_method(
    field: u8,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let info = crate::time::field_info(field);
    Ok(Some(match method {
        "ordinal" => JValue::Int(i32::from(field)),
        // An enum's order is its ordinal's.
        "compareTo" => match args.first() {
            Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                Some(HeapObject::Temporal(Temporal::Field(other))) => {
                    JValue::Int(i32::from(field) - i32::from(*other))
                }
                _ => return Err(throw("java.lang.ClassCastException: not a ChronoField")),
            },
            _ => return Err(throw("java.lang.NullPointerException")),
        },
        "range" => JValue::Ref(Some(
            heap.intern_temporal(Temporal::Range(crate::time::field_range(field))),
        )),
        "isDateBased" => JValue::Int(i32::from(crate::time::field_is_date_based(field))),
        "isTimeBased" => JValue::Int(i32::from(crate::time::field_is_time_based(field))),
        "getBaseUnit" => JValue::Ref(Some(heap.intern_temporal(Temporal::Unit(info.base_unit)))),
        "getRangeUnit" => JValue::Ref(Some(heap.intern_temporal(Temporal::Unit(info.range_unit)))),
        "getDisplayName" => {
            let reference = heap.alloc_string(info.text);
            JValue::Ref(Some(reference))
        }
        // `isSupportedBy(temporal)` and `getFrom(temporal)` ask the value,
        // which is the same question `isSupported`/`getLong` ask the other
        // way round.
        "isSupportedBy" | "getFrom" | "rangeRefinedBy" => {
            let Some(JValue::Ref(Some(reference))) = args.first() else {
                return Err(throw("java.lang.NullPointerException"));
            };
            let Some(HeapObject::Temporal(value)) = heap.get(*reference) else {
                return Err(throw("java.lang.ClassCastException: not a temporal"));
            };
            let value = *value;
            match method {
                "isSupportedBy" => JValue::Int(i32::from(supports_field(value, field))),
                "getFrom" => JValue::Long(field_value(value, field)?),
                _ => JValue::Ref(Some(
                    heap.intern_temporal(Temporal::Range(field_range_of(value, field)?)),
                )),
            }
        }
        "checkValidValue" | "checkValidIntValue" => {
            let value = match args.first() {
                Some(JValue::Long(v)) => *v,
                Some(JValue::Int(v)) => i64::from(*v),
                _ => 0,
            };
            let range = crate::time::field_range(field);
            if !range.contains(value) {
                return Err(date_time_exception(&format!(
                    "Invalid value for {} (valid values {}): {value}",
                    info.text,
                    range.text()
                )));
            }
            if method == "checkValidIntValue" {
                JValue::Int(i32::try_from(value).unwrap_or(0))
            } else {
                JValue::Long(value)
            }
        }
        _ => {
            return Err(VmError::UnknownIntrinsic(format!(
                "java/time/temporal/ChronoField.{method}"
            )));
        }
    }))
}

/// The `ChronoField` constants, in the order the compiler emits them — which
/// is `crate::time::FIELDS`' order, spelled as the enum names.
static FIELD_CONSTANTS: [&str; 30] = [
    "NANO_OF_SECOND",
    "NANO_OF_DAY",
    "MICRO_OF_SECOND",
    "MICRO_OF_DAY",
    "MILLI_OF_SECOND",
    "MILLI_OF_DAY",
    "SECOND_OF_MINUTE",
    "SECOND_OF_DAY",
    "MINUTE_OF_HOUR",
    "MINUTE_OF_DAY",
    "HOUR_OF_AMPM",
    "CLOCK_HOUR_OF_AMPM",
    "HOUR_OF_DAY",
    "CLOCK_HOUR_OF_DAY",
    "AMPM_OF_DAY",
    "DAY_OF_WEEK",
    "ALIGNED_DAY_OF_WEEK_IN_MONTH",
    "ALIGNED_DAY_OF_WEEK_IN_YEAR",
    "DAY_OF_MONTH",
    "DAY_OF_YEAR",
    "EPOCH_DAY",
    "ALIGNED_WEEK_OF_MONTH",
    "ALIGNED_WEEK_OF_YEAR",
    "MONTH_OF_YEAR",
    "PROLEPTIC_MONTH",
    "YEAR_OF_ERA",
    "YEAR",
    "ERA",
    "INSTANT_SECONDS",
    "OFFSET_SECONDS",
];

/// The `ChronoField` an argument names, if it is one.
fn field_argument(heap: &Heap, value: Option<&JValue>) -> Option<u8> {
    match value {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference)? {
            HeapObject::Temporal(Temporal::Field(field)) => Some(*field),
            _ => None,
        },
        _ => None,
    }
}

/// `isSupported(field)`, `get(field)`, `getLong(field)`, `range(field)` and
/// `with(field, value)` — the `TemporalAccessor` surface, in one place.
fn field_surface(
    value: Temporal,
    field: u8,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    match method {
        "isSupported" => Ok(Some(JValue::Int(i32::from(supports_field(value, field))))),
        "range" => {
            let range = field_range_of(value, field)?;
            Ok(Some(JValue::Ref(Some(
                heap.intern_temporal(Temporal::Range(range)),
            ))))
        }
        "getLong" => Ok(Some(JValue::Long(field_value(value, field)?))),
        "get" => {
            // A field the value does not have at all is reported as that
            // FIRST — the width complaint below is for a field it does have.
            if !supports_field(value, field) {
                return Err(unsupported_field(field));
            }
            // Four fields are too wide for an `int`, and a JDK says so rather
            // than truncating — even for a value that would have fitted.
            if matches!(field, 1 | 3 | 20 | 24) {
                return Err(VmError::UncaughtException(format!(
                    "java.time.temporal.UnsupportedTemporalTypeException: Invalid field '{}' \
                     for get() method, use getLong() instead",
                    crate::time::field_info(field).text
                )));
            }
            let answer = field_value(value, field)?;
            Ok(Some(JValue::Int(i32::try_from(answer).unwrap_or(0))))
        }
        _ => {
            let wanted = match args.get(1) {
                Some(JValue::Long(v)) => *v,
                Some(JValue::Int(v)) => i64::from(*v),
                _ => 0,
            };
            with_field(value, field, wanted, heap)
        }
    }
}

/// `with(field, value)` — the same value with ONE field changed.
fn with_field(
    value: Temporal,
    field: u8,
    wanted: i64,
    heap: &mut Heap,
) -> Result<Option<JValue>, VmError> {
    // The VALUE is checked against the field's OWN range first, before the
    // receiver is asked whether it has the field at all — so
    // `LocalDate.with(HOUR_OF_AMPM, 30)` complains about the 30 rather than
    // about a date having no hour.
    let range = crate::time::field_range(field);
    if !range.contains(wanted) {
        return Err(date_time_exception(&format!(
            "Invalid value for {} (valid values {}): {wanted}",
            crate::time::field_info(field).text,
            range.text()
        )));
    }
    if !supports_field(value, field) {
        return Err(unsupported_field(field));
    }
    let current = field_value(value, field)?;
    let (date, time) = match value {
        Temporal::Date(date) => (Some(date), None),
        Temporal::Time(time) => (None, Some(time)),
        Temporal::DateTime(when) => (Some(when.date), Some(when.time)),
        _ => return Err(unsupported_field(field)),
    };
    // Every field is either a NUMBER of some unit away from where it is —
    // which is exactly `plus` — or a piece of the date to write directly.
    let rebuilt = if crate::time::field_is_time_based(field) {
        let Some(time) = time else {
            return Err(unsupported_field(field));
        };
        // A "-of-second" field REPLACES the whole nano-of-second, so setting
        // the microsecond clears the nanoseconds under it; a "-of-day" field
        // replaces the whole day. The rest move the clock by a difference.
        let second_start = time.nano_of_day - time.nano_of_day % crate::time::NANOS_PER_SECOND;
        let nano = match field {
            0 => second_start + wanted,
            1 => wanted,
            2 => second_start + wanted * 1_000,
            3 => wanted * 1_000,
            4 => second_start + wanted * 1_000_000,
            5 => wanted * 1_000_000,
            6 | 7 => time.nano_of_day + (wanted - current) * crate::time::NANOS_PER_SECOND,
            8 | 9 => time.nano_of_day + (wanted - current) * crate::time::NANOS_PER_MINUTE,
            10 | 12 => time.nano_of_day + (wanted - current) * crate::time::NANOS_PER_HOUR,
            11 | 13 => {
                let midnight = if field == 11 { 12 } else { 24 };
                let hours = if wanted == midnight { 0 } else { wanted };
                let now = if current == midnight { 0 } else { current };
                time.nano_of_day + (hours - now) * crate::time::NANOS_PER_HOUR
            }
            _ => time.nano_of_day + (wanted - current) * 12 * crate::time::NANOS_PER_HOUR,
        };
        let made = crate::time::Time {
            nano_of_day: nano.rem_euclid(crate::time::NANOS_PER_DAY),
        };
        match value {
            Temporal::Time(_) => Temporal::Time(made),
            _ => Temporal::DateTime(crate::time::DateTime {
                date: date.unwrap_or_else(|| crate::time::Date::from_epoch_day(0)),
                time: made,
            }),
        }
    } else {
        let Some(date) = date else {
            return Err(unsupported_field(field));
        };
        let made = date_with_field(date, field, wanted, current)?;
        match value {
            Temporal::Date(_) => Temporal::Date(made),
            _ => Temporal::DateTime(crate::time::DateTime {
                date: made,
                time: time.unwrap_or(crate::time::Time { nano_of_day: 0 }),
            }),
        }
    };
    Ok(Some(JValue::Ref(Some(heap.intern_temporal(rebuilt)))))
}

/// A date with one calendar field changed.
fn date_with_field(
    date: crate::time::Date,
    field: u8,
    wanted: i64,
    current: i64,
) -> Result<crate::time::Date, VmError> {
    let days = |count: i64| {
        Ok(crate::time::Date::from_epoch_day(
            date.to_epoch_day() + count,
        ))
    };
    let year = |value: i64| {
        let year = i32::try_from(value).unwrap_or(0);
        crate::time::Date::of(
            year,
            i32::from(date.month),
            i32::from(date.day.min(crate::time::length_of_month(year, date.month))),
        )
        .map_err(|message| date_time_exception(&message))
    };
    match field {
        // A day field moves by days and a WEEK field by weeks.
        15..=17 | 19 => days(wanted - current),
        21 | 22 => days((wanted - current) * 7),
        18 => crate::time::Date::of(
            date.year,
            i32::from(date.month),
            i32::try_from(wanted).unwrap_or(1),
        )
        .map_err(|message| date_time_exception(&message)),
        20 => Ok(crate::time::Date::from_epoch_day(wanted)),
        23 => {
            let month = u8::try_from(wanted).unwrap_or(1);
            crate::time::Date::of(
                date.year,
                i32::from(month),
                i32::from(date.day.min(crate::time::length_of_month(date.year, month))),
            )
            .map_err(|message| date_time_exception(&message))
        }
        24 => {
            let months = wanted - current;
            let total = i64::from(date.year) * 12 + i64::from(date.month) - 1 + months;
            let made_year = i32::try_from(total.div_euclid(12)).unwrap_or(0);
            let made_month = u8::try_from(total.rem_euclid(12) + 1).unwrap_or(1);
            crate::time::Date::of(
                made_year,
                i32::from(made_month),
                i32::from(
                    date.day
                        .min(crate::time::length_of_month(made_year, made_month)),
                ),
            )
            .map_err(|message| date_time_exception(&message))
        }
        25 => year(if date.year >= 1 { wanted } else { 1 - wanted }),
        26 => year(wanted),
        // The ERA: staying in the same era changes nothing, and swapping it
        // reflects the year about 1.
        _ => {
            if wanted == i64::from(u8::from(date.year >= 1)) {
                Ok(date)
            } else {
                year(1 - i64::from(date.year))
            }
        }
    }
}

/// The message a JDK gives for a field a value does not have.
fn unsupported_field(field: u8) -> VmError {
    VmError::UncaughtException(format!(
        "java.time.temporal.UnsupportedTemporalTypeException: Unsupported field: {}",
        crate::time::field_info(field).text
    ))
}

/// One field of a `java.time` value, for the formatter — `None` where the
/// value does not have it, which is what a date-time conversion reports.
#[must_use]
pub(crate) fn temporal_field(value: Temporal, field: u8) -> Option<i64> {
    field_value(value, field).ok()
}

/// One field's value. This is the whole of `getLong(field)`, and everything
/// else that reads a field goes through it.
#[allow(clippy::too_many_lines)] // one arm per field
fn field_value(value: Temporal, field: u8) -> Result<i64, VmError> {
    // `INSTANT_SECONDS` and `OFFSET_SECONDS` are neither date- nor
    // time-based: they need a zone, which none of these values has.
    if !supports_field(value, field) {
        return Err(unsupported_field(field));
    }
    let (date, time) = match value {
        Temporal::Date(date) => (Some(date), None),
        Temporal::Time(time) => (None, Some(time)),
        Temporal::DateTime(when) => (Some(when.date), Some(when.time)),
        Temporal::DayOfWeek(day) if field == 15 => return Ok(i64::from(day)),
        Temporal::Month(month) if field == 23 => return Ok(i64::from(month)),
        _ => return Err(unsupported_field(field)),
    };
    if crate::time::field_is_time_based(field) {
        let Some(time) = time else {
            return Err(unsupported_field(field));
        };
        let nano = time.nano_of_day;
        let hour = nano / crate::time::NANOS_PER_HOUR;
        let minute = (nano / crate::time::NANOS_PER_MINUTE) % 60;
        let second = (nano / crate::time::NANOS_PER_SECOND) % 60;
        return Ok(match field {
            0 => nano % 1_000_000_000,
            1 => nano,
            2 => (nano % 1_000_000_000) / 1_000,
            3 => nano / 1_000,
            4 => (nano % 1_000_000_000) / 1_000_000,
            5 => nano / 1_000_000,
            6 => second,
            7 => nano / 1_000_000_000,
            8 => minute,
            9 => hour * 60 + minute,
            10 => hour % 12,
            11 => {
                let of_am_pm = hour % 12;
                if of_am_pm == 0 { 12 } else { of_am_pm }
            }
            12 => hour,
            13 => {
                if hour == 0 {
                    24
                } else {
                    hour
                }
            }
            _ => hour / 12,
        });
    }
    let Some(date) = date else {
        return Err(unsupported_field(field));
    };
    let day_of_year = i64::from(date.day_of_year());
    let year = i64::from(date.year);
    Ok(match field {
        15 => i64::from(date.day_of_week()),
        16 => (i64::from(date.day) - 1) % 7 + 1,
        17 => (day_of_year - 1) % 7 + 1,
        18 => i64::from(date.day),
        19 => day_of_year,
        20 => date.to_epoch_day(),
        21 => (i64::from(date.day) - 1) / 7 + 1,
        22 => (day_of_year - 1) / 7 + 1,
        23 => i64::from(date.month),
        24 => year * 12 + i64::from(date.month) - 1,
        25 => {
            if year >= 1 {
                year
            } else {
                1 - year
            }
        }
        26 => year,
        _ => i64::from(u8::from(year >= 1)),
    })
}

/// A field's range NARROWED by the value: February 2024 has 29 days, and a
/// year before the era has fewer years in it.
fn field_range_of(value: Temporal, field: u8) -> Result<crate::time::ValueRange, VmError> {
    if !supports_field(value, field) {
        return Err(unsupported_field(field));
    }
    let date = match value {
        Temporal::Date(date) => Some(date),
        Temporal::DateTime(when) => Some(when.date),
        _ => None,
    };
    let Some(date) = date else {
        return Ok(crate::time::field_range(field));
    };
    Ok(match field {
        18 => crate::time::ValueRange::fixed(
            1,
            i64::from(crate::time::length_of_month(date.year, date.month)),
        ),
        19 => crate::time::ValueRange::fixed(
            1,
            if crate::time::is_leap_year(date.year) {
                366
            } else {
                365
            },
        ),
        // The aligned week of a month runs to 4 or 5 depending on where the
        // first of the month falls.
        21 => crate::time::ValueRange::fixed(
            1,
            if date.day <= 2 && date.day_of_week() > 4 {
                4
            } else {
                5
            },
        ),
        25 => crate::time::ValueRange::fixed(
            1,
            if date.year <= 0 {
                1_000_000_000
            } else {
                999_999_999
            },
        ),
        _ => crate::time::field_range(field),
    })
}

/// What a `ChronoUnit` says about ITSELF — how long it is, what it measures,
/// and whether a given value can be moved by it. `None` means "not one of
/// these", so the caller carries on to `between`.
fn unit_facts(
    unit: u8,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let answer = match method {
        "getDuration" => {
            let made = Temporal::Duration(crate::time::unit_duration(unit));
            JValue::Ref(Some(heap.intern_temporal(made)))
        }
        "isDateBased" => JValue::Int(i32::from(crate::time::unit_is_date_based(unit))),
        "isTimeBased" => JValue::Int(i32::from(crate::time::unit_is_time_based(unit))),
        "isDurationEstimated" => JValue::Int(i32::from(crate::time::unit_is_estimated(unit))),
        "isSupportedBy" => {
            let supported = match args.first() {
                Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::Temporal(value)) => supports_unit(*value, unit),
                    _ => false,
                },
                _ => false,
            };
            JValue::Int(i32::from(supported))
        }
        "compareTo" => {
            let against = match args.first() {
                Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::Temporal(Temporal::Unit(other))) => i32::from(*other),
                    _ => return Err(throw("java.lang.ClassCastException: not a ChronoUnit")),
                },
                _ => return Err(throw("java.lang.NullPointerException")),
            };
            JValue::Int(i32::from(unit) - against)
        }
        _ => return Ok(None),
    };
    Ok(Some(answer))
}

/// `with(x)` where `x` is a value rather than a field: an adjuster, a month,
/// a day of week, or the date or time half of a stamp.
fn adjust_with(
    value: Temporal,
    adjuster: Temporal,
    heap: &mut Heap,
) -> Result<Option<JValue>, VmError> {
    let (date, time) = match value {
        Temporal::Date(date) => (Some(date), None),
        Temporal::Time(time) => (None, Some(time)),
        Temporal::DateTime(when) => (Some(when.date), Some(when.time)),
        _ => return Err(throw("java.lang.ClassCastException: not a temporal")),
    };
    let rebuild = |heap: &mut Heap, date: Option<crate::time::Date>, time| {
        let made = match (date, time) {
            (Some(date), Some(time)) => Temporal::DateTime(crate::time::DateTime { date, time }),
            (Some(date), None) => Temporal::Date(date),
            (None, Some(time)) => Temporal::Time(time),
            (None, None) => return Err(throw("java.lang.ClassCastException: not a temporal")),
        };
        Ok(Some(JValue::Ref(Some(heap.intern_temporal(made)))))
    };
    match adjuster {
        Temporal::Adjuster(rule) => {
            let Some(date) = date else {
                return Err(throw("java.lang.ClassCastException: not a date"));
            };
            rebuild(heap, Some(rule.apply(date)), time)
        }
        // A month or a day of week SETS its own field.
        Temporal::Month(month) => with_field(value, 23, i64::from(month), heap),
        Temporal::DayOfWeek(day) => with_field(value, 15, i64::from(day), heap),
        Temporal::Date(replacement) => rebuild(heap, Some(replacement), time),
        Temporal::Time(replacement) => rebuild(heap, date, Some(replacement)),
        Temporal::DateTime(replacement) => {
            rebuild(heap, Some(replacement.date), Some(replacement.time))
        }
        _ => Err(throw("java.lang.ClassCastException: not an adjuster")),
    }
}

/// `plus(amount)` — a whole `Period` or `Duration` at once.
fn shift_by_amount(
    value: Temporal,
    amount: Temporal,
    negate: bool,
    heap: &mut Heap,
) -> Result<Option<JValue>, VmError> {
    let sign = if negate { -1 } else { 1 };
    match amount {
        // A period is years, then months, then days — in that order, because
        // each may land on a shorter month than the last.
        Temporal::Period(period) => {
            // Years and months go on TOGETHER, as one number of months: a
            // JDK adds `years * 12 + months` and then the days, and adding
            // the years first would clamp February 29 to the 28th on the way
            // through.
            let mut moved = value;
            for (unit, count) in [
                (9, i64::from(period.years) * 12 + i64::from(period.months)),
                (7, i64::from(period.days)),
            ] {
                if count == 0 {
                    continue;
                }
                let Some(JValue::Ref(Some(reference))) =
                    shift_by_unit(moved, count * sign, unit, heap)?
                else {
                    return Err(throw("java.lang.ClassCastException: not a temporal"));
                };
                let Some(HeapObject::Temporal(next)) = heap.get(reference) else {
                    return Err(throw("java.lang.ClassCastException: not a temporal"));
                };
                moved = *next;
            }
            Ok(Some(JValue::Ref(Some(heap.intern_temporal(moved)))))
        }
        Temporal::Duration(duration) => {
            let Some(JValue::Ref(Some(reference))) =
                shift_by_unit(value, duration.seconds * sign, 3, heap)?
            else {
                return Err(throw("java.lang.ClassCastException: not a temporal"));
            };
            let Some(HeapObject::Temporal(moved)) = heap.get(reference) else {
                return Err(throw("java.lang.ClassCastException: not a temporal"));
            };
            shift_by_unit(*moved, i64::from(duration.nanos) * sign, 0, heap)
        }
        _ => Err(throw("java.lang.ClassCastException: not an amount")),
    }
}

/// `LocalTime.from(temporal)` and `LocalDateTime.from(temporal)` — the half
/// of a value that is asked for.
fn temporal_from(heap: &mut Heap, args: &[JValue], whole: bool) -> Result<Option<JValue>, VmError> {
    let Some(JValue::Ref(Some(reference))) = args.first() else {
        return Err(throw("java.lang.NullPointerException"));
    };
    let Some(HeapObject::Temporal(value)) = heap.get(*reference) else {
        return Err(throw("java.lang.ClassCastException: not a temporal"));
    };
    let made = match (*value, whole) {
        (Temporal::Time(time), false) => Temporal::Time(time),
        (Temporal::DateTime(when), false) => Temporal::Time(when.time),
        (Temporal::DateTime(when), true) => Temporal::DateTime(when),
        (other, _) => {
            return Err(date_time_exception(&format!(
                "Unable to obtain {} from TemporalAccessor: {}",
                if whole { "LocalDateTime" } else { "LocalTime" },
                other.text()
            )));
        }
    };
    Ok(Some(JValue::Ref(Some(heap.intern_temporal(made)))))
}

/// The `ChronoUnit` an argument names, if it is one.
fn unit_argument(heap: &Heap, value: Option<&JValue>) -> Option<u8> {
    match value {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference)? {
            HeapObject::Temporal(Temporal::Unit(unit)) => Some(*unit),
            _ => None,
        },
        _ => None,
    }
}

/// `plus(amount, unit)` — the same value moved by a number of any unit it
/// knows.
fn shift_by_unit(
    value: Temporal,
    amount: i64,
    unit: u8,
    heap: &mut Heap,
) -> Result<Option<JValue>, VmError> {
    if !supports_unit(value, unit) {
        return Err(VmError::UncaughtException(format!(
            "java.time.temporal.UnsupportedTemporalTypeException: Unsupported unit: {}",
            crate::time::unit_name(unit)
        )));
    }
    let (date, time) = match value {
        Temporal::Date(date) => (Some(date), None),
        Temporal::Time(time) => (None, Some(time)),
        Temporal::DateTime(when) => (Some(when.date), Some(when.time)),
        _ => return Err(throw("java.lang.ClassCastException: not a temporal")),
    };
    // A clock unit moves the nano-of-day, carrying whole days into the date;
    // a calendar unit moves the date and leaves the clock alone.
    let made = if let Some(nanos) = crate::time::unit_nanos(unit).filter(|_| unit < 7) {
        let Some(time) = time else {
            return Err(throw("java.lang.ClassCastException: not a time"));
        };
        let total = i128::from(time.nano_of_day) + i128::from(amount) * i128::from(nanos);
        let day_nanos = i128::from(crate::time::NANOS_PER_DAY);
        let carried = total.div_euclid(day_nanos);
        let moved = crate::time::Time {
            nano_of_day: i64::try_from(total.rem_euclid(day_nanos)).unwrap_or(0),
        };
        match date {
            None => Temporal::Time(moved),
            Some(date) => Temporal::DateTime(crate::time::DateTime {
                date: date.plus_days(i64::try_from(carried).unwrap_or(0)),
                time: moved,
            }),
        }
    } else {
        let Some(date) = date else {
            return Err(throw("java.lang.ClassCastException: not a date"));
        };
        let moved = match unit {
            7 => date.plus_days(amount),
            8 => date.plus_days(amount.saturating_mul(7)),
            9 => date.plus_months(amount),
            10 => date.plus_years(amount),
            11 => date.plus_years(amount.saturating_mul(10)),
            12 => date.plus_years(amount.saturating_mul(100)),
            13 => date.plus_years(amount.saturating_mul(1000)),
            // An ERA away is the SAME date in the other era, which is what
            // setting the era field does.
            _ => {
                let era = i64::from(u8::from(date.year >= 1));
                return with_field(Temporal::Date(date), 27, era.saturating_add(amount), heap).map(
                    |answer| match (answer, time) {
                        (Some(JValue::Ref(Some(made))), Some(time)) => {
                            let Some(HeapObject::Temporal(Temporal::Date(date))) = heap.get(made)
                            else {
                                return Some(JValue::Ref(Some(made)));
                            };
                            let when = crate::time::DateTime { date: *date, time };
                            Some(JValue::Ref(Some(
                                heap.intern_temporal(Temporal::DateTime(when)),
                            )))
                        }
                        (other, _) => other,
                    },
                );
            }
        };
        match time {
            None => Temporal::Date(moved),
            Some(time) => Temporal::DateTime(crate::time::DateTime { date: moved, time }),
        }
    };
    Ok(Some(JValue::Ref(Some(heap.intern_temporal(made)))))
}

/// Whether a value can be moved by a unit: a date knows nothing smaller than
/// a day, a time nothing larger than a half-day, and nothing at all measures
/// FOREVER.
fn supports_unit(value: Temporal, unit: u8) -> bool {
    match value {
        Temporal::Date(_) => (7..=14).contains(&unit),
        Temporal::Time(_) => unit <= 6,
        Temporal::DateTime(_) => unit <= 14,
        _ => false,
    }
}

/// Whether a value answers a field at all — `LocalDate` knows nothing of an
/// hour, and `LocalTime` nothing of a month.
fn supports_field(value: Temporal, field: u8) -> bool {
    match value {
        Temporal::Date(_) => crate::time::field_is_date_based(field),
        Temporal::Time(_) => crate::time::field_is_time_based(field),
        Temporal::DateTime(_) => {
            crate::time::field_is_date_based(field) || crate::time::field_is_time_based(field)
        }
        Temporal::DayOfWeek(_) => field == 15,
        Temporal::Month(_) => field == 23,
        _ => false,
    }
}

fn unit_method(
    unit: u8,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    if method == "ordinal" {
        return Ok(Some(JValue::Int(i32::from(unit))));
    }
    if let Some(answer) = unit_facts(unit, heap, method, args)? {
        return Ok(Some(answer));
    }
    if method != "between" {
        return Err(VmError::UnknownIntrinsic(format!(
            "java/time/temporal/ChronoUnit.{method}"
        )));
    }
    let read = |at: usize| match args.get(at) {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
            Some(HeapObject::Temporal(value)) => Some(*value),
            _ => None,
        },
        _ => None,
    };
    let (Some(start), Some(end)) = (read(0), read(1)) else {
        return Err(throw("java.lang.ClassCastException: not a temporal"));
    };
    // A date, a time and a date-time each reduce to (day, nano-of-day) —
    // NOT to nanoseconds on one line, which overflows two centuries out.
    let split = |value: Temporal| match value {
        Temporal::Date(date) => Some((date.to_epoch_day(), 0, true)),
        Temporal::Time(time) => Some((0, time.nano_of_day, false)),
        Temporal::DateTime(when) => Some((when.date.to_epoch_day(), when.time.nano_of_day, false)),
        _ => None,
    };
    let calendar = |value: Temporal| match value {
        Temporal::Date(date) => Some(date),
        Temporal::DateTime(when) => Some(when.date),
        _ => None,
    };
    // From MONTHS up these are CALENDAR units: they count whole months the
    // way `Period.between` does, not a number of nanoseconds.
    if unit >= 9 {
        let (Some(from), Some(to)) = (calendar(start), calendar(end)) else {
            return Err(throw("java.lang.ClassCastException: not a date"));
        };
        // An ERA away is a different era, which a proleptic calendar reaches
        // only across year zero; FOREVER is not a span anything measures.
        if unit == 14 {
            let era = |date: crate::time::Date| i64::from(u8::from(date.year >= 1));
            return Ok(Some(JValue::Long(era(to) - era(from))));
        }
        if unit == 15 {
            return Err(VmError::UncaughtException(String::from(
                "java.time.temporal.UnsupportedTemporalTypeException: Unsupported unit: Forever",
            )));
        }
        let months = crate::time::Period::between(from, to).total_months();
        let per = match unit {
            9 => 1,
            10 => 12,
            11 => 120,
            12 => 1200,
            _ => 12_000,
        };
        return Ok(Some(JValue::Long(months / per)));
    }
    let (Some((from_day, from_nano, from_is_date)), Some((to_day, to_nano, _))) =
        (split(start), split(end))
    else {
        return Err(throw("java.lang.ClassCastException: not a temporal"));
    };
    // A DATE has no hour to count, and `java.time` says so rather than
    // pretending it is midnight.
    if from_is_date && unit < 7 {
        return Err(VmError::UncaughtException(format!(
            "java.time.temporal.UnsupportedTemporalTypeException: Unsupported unit: {}",
            crate::time::unit_name(unit)
        )));
    }
    let days = to_day - from_day;
    let nanos = to_nano - from_nano;
    // Days and weeks count WHOLE ones: a day that is nineteen hours long is
    // none, whichever way round it runs.
    if unit >= 7 {
        let whole = if days > 0 && nanos < 0 {
            days - 1
        } else if days < 0 && nanos > 0 {
            days + 1
        } else {
            days
        };
        return Ok(Some(JValue::Long(if unit == 8 {
            whole / 7
        } else {
            whole
        })));
    }
    let Some(per_unit) = crate::time::unit_nanos(unit) else {
        return Err(VmError::UnknownIntrinsic(String::from(
            "java/time/temporal/ChronoUnit.between",
        )));
    };
    // In i128, because a span of centuries is more nanoseconds than an i64
    // holds — and then back, because the ANSWER fits once it is divided.
    let total = i128::from(days) * i128::from(crate::time::NANOS_PER_DAY) + i128::from(nanos);
    let answer = total / i128::from(per_unit);
    Ok(Some(JValue::Long(
        i64::try_from(answer).unwrap_or(i64::MAX),
    )))
}

/// `LocalDate.compareTo`'s answer: the first field that differs, as a raw
/// DIFFERENCE (not normalized), which is what a program printing it sees.
fn date_difference(left: crate::time::Date, right: crate::time::Date) -> i64 {
    if left.year != right.year {
        i64::from(left.year) - i64::from(right.year)
    } else if left.month != right.month {
        i64::from(left.month) - i64::from(right.month)
    } else {
        i64::from(left.day) - i64::from(right.day)
    }
}

/// `until(end)` is a `Period`; `until(end, unit)` is that unit's count, which
/// is `ChronoUnit.between` written the other way round.
fn date_until(
    date: crate::time::Date,
    heap: &mut Heap,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let read = |at: usize| match args.get(at) {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
            Some(HeapObject::Temporal(value)) => Some(*value),
            _ => None,
        },
        _ => None,
    };
    let Some(Temporal::Date(end)) = read(0) else {
        return Err(throw("java.lang.ClassCastException: not a LocalDate"));
    };
    match read(1) {
        // `until(end, unit)` is `unit.between(this, end)` — and the
        // receiver has to be handed over as a value, since `between`
        // reads BOTH ends from its arguments.
        Some(Temporal::Unit(unit)) => {
            let start = JValue::Ref(Some(heap.intern_temporal(Temporal::Date(date))));
            unit_method(unit, heap, "between", &[start, args[0]])
        }
        _ => Ok(Some(JValue::Ref(Some(heap.intern_temporal(
            Temporal::Period(crate::time::Period::between(date, end)),
        ))))),
    }
}

/// The readers: everything a date answers about ITSELF, with no argument.
fn date_reader(value: Temporal, heap: &mut Heap, method: &str) -> Option<JValue> {
    let Temporal::Date(date) = value else {
        return None;
    };
    let int = |value: i64| Some(JValue::Int(i32::try_from(value).unwrap_or(i32::MAX)));
    match method {
        "getYear" => int(i64::from(date.year)),
        "getMonthValue" => int(i64::from(date.month)),
        "getDayOfMonth" => int(i64::from(date.day)),
        "getDayOfYear" => int(i64::from(date.day_of_year())),
        "lengthOfMonth" => int(i64::from(crate::time::length_of_month(
            date.year, date.month,
        ))),
        "lengthOfYear" => int(i64::from(date.length_of_year())),
        "isLeapYear" => Some(JValue::Int(i32::from(crate::time::is_leap_year(date.year)))),
        "toEpochDay" => Some(JValue::Long(date.to_epoch_day())),
        "getDayOfWeek" => Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::DayOfWeek(date.day_of_week())),
        ))),
        "getMonth" => Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::Month(date.month)),
        ))),
        "atStartOfDay" => Some(JValue::Ref(Some(heap.intern_temporal(Temporal::DateTime(
            crate::time::DateTime {
                date,
                time: crate::time::Time { nano_of_day: 0 },
            },
        ))))),
        // The ISO calendar has two eras, and year one begins the later one.
        "getEra" => Some(JValue::Ref(Some(
            heap.intern_temporal(Temporal::Era(u8::from(date.year >= 1))),
        ))),
        _ => None,
    }
}

/// One `LocalDate`/`DayOfWeek`/`Month` method. These are VALUE types: every
/// answer is a new value, `equals` compares contents, and the two enums are
/// interned so `==` works on them as it does in Java.
#[allow(clippy::too_many_lines)] // one arm per shape
fn temporal_method(
    value: Temporal,
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let int = |value: i64| Ok(Some(JValue::Int(i32::try_from(value).unwrap_or(i32::MAX))));
    // The argument of a one-argument call, as a number and as another temporal.
    let number = || match args.first() {
        Some(JValue::Int(n)) => i64::from(*n),
        Some(JValue::Long(n)) => *n,
        _ => 0,
    };
    let other = |heap: &Heap| match args.first() {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
            Some(HeapObject::Temporal(other)) => Some(*other),
            _ => None,
        },
        _ => None,
    };

    if let Some(answer) = temporal_object_method(value, heap, method, args) {
        return Ok(Some(answer));
    }
    // `format(formatter)` reads the same on a date, a time and a date-time.
    if method == "format"
        && let Some(JValue::Ref(Some(formatter))) = args.first()
    {
        return format_temporal(value, heap, *formatter);
    }
    // `datesUntil(end)` — every date from here up to, but not including, the
    // end. A JDK's is lazy; a fixed stream answers the same questions.
    if method == "datesUntil"
        && let Temporal::Date(date) = value
        && let Some(JValue::Ref(Some(reference))) = args.first()
        && let Some(HeapObject::Temporal(Temporal::Date(end))) = heap.get(*reference)
    {
        let end = *end;
        let mut dates: Vec<JValue> = Vec::new();
        let mut at = date;
        while at.to_epoch_day() < end.to_epoch_day() {
            dates.push(JValue::Ref(Some(heap.intern_temporal(Temporal::Date(at)))));
            at = at.plus_days(1);
        }
        let stream = heap.alloc(HeapObject::Stream {
            source: crate::value::StreamSource::Fixed(dates),
            ops: Vec::new(),
        });
        return Ok(Some(JValue::Ref(Some(stream))));
    }
    // `with(adjuster)` and `with(temporal)` — one value adjusting another,
    // which reads the same on a date and a stamp.
    if method == "with"
        && args.len() == 1
        && let Some(JValue::Ref(Some(reference))) = args.first()
        && let Some(HeapObject::Temporal(adjuster)) = heap.get(*reference)
    {
        return adjust_with(value, *adjuster, heap);
    }
    // `plus(amount)` and `minus(amount)` where the amount is a `Period` or a
    // `Duration` — a whole amount rather than a number and a unit.
    if matches!(method, "plus" | "minus")
        && args.len() == 1
        && matches!(
            value,
            Temporal::Date(_) | Temporal::Time(_) | Temporal::DateTime(_)
        )
        && let Some(JValue::Ref(Some(reference))) = args.first()
        && let Some(HeapObject::Temporal(amount @ (Temporal::Period(_) | Temporal::Duration(_)))) =
            heap.get(*reference)
    {
        return shift_by_amount(value, *amount, method == "minus", heap);
    }
    // `plus(amount, unit)` and `minus(amount, unit)` read the same on all
    // three values, and so does `until(end, unit)`.
    if matches!(method, "plus" | "minus")
        && let Some(unit) = unit_argument(heap, args.get(1))
    {
        let amount = match args.first() {
            Some(JValue::Long(n)) => *n,
            Some(JValue::Int(n)) => i64::from(*n),
            _ => 0,
        };
        let amount = if method == "minus" { -amount } else { amount };
        return shift_by_unit(value, amount, unit, heap);
    }
    // `until(end, unit)` is `unit.between(this, end)` — and the receiver has
    // to be handed over as a value, since `between` reads BOTH ends from its
    // arguments.
    if method == "until"
        && let Some(unit) = unit_argument(heap, args.get(1))
        && let Some(end) = args.first().copied()
    {
        let start = JValue::Ref(Some(heap.intern_temporal(value)));
        return unit_method(unit, heap, "between", &[start, end]);
    }
    // The FIELD surface — `isSupported`, `get`, `getLong`, `range` and the
    // two-argument `with` — reads the same on every value that has fields,
    // which is why it sits above the per-type dispatch.
    if let Some(field) = field_argument(heap, args.first())
        && matches!(method, "isSupported" | "get" | "getLong" | "range" | "with")
    {
        return field_surface(value, field, heap, method, args);
    }
    // An enum answers the shared methods above; the rest are its own.
    match value {
        Temporal::DayOfWeek(_) | Temporal::Month(_) => {
            return temporal_enum_method(value, heap, method, args);
        }
        Temporal::Time(time) => return time_method(time, heap, method, args),
        Temporal::DateTime(when) => return date_time_method(when, heap, method, args),
        Temporal::Duration(amount) => return duration_method(amount, heap, method, args),
        Temporal::Period(period) => return period_method(period, heap, method, args),
        Temporal::Unit(unit) => return unit_method(unit, heap, method, args),
        Temporal::Field(field) => return field_method(field, heap, method, args),
        Temporal::Range(range) => return range_method(range, method, args),
        // An era answers only what an enum answers.
        Temporal::Era(era) => {
            return match method {
                "getValue" | "ordinal" => Ok(Some(JValue::Int(i32::from(era)))),
                "compareTo" => match args.first() {
                    Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                        Some(HeapObject::Temporal(Temporal::Era(other))) => {
                            Ok(Some(JValue::Int(i32::from(era) - i32::from(*other))))
                        }
                        _ => Err(throw("java.lang.ClassCastException: not an IsoEra")),
                    },
                    _ => Err(throw("java.lang.NullPointerException")),
                },
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "java/time/chrono/IsoEra.{method}"
                ))),
            };
        }
        // An adjuster is a rule, not a value: it answers nothing itself, and
        // reaches a date through `with`.
        Temporal::Adjuster(_) => {
            return Err(VmError::UnknownIntrinsic(format!(
                "java/time/temporal/TemporalAdjusters.{method}"
            )));
        }
        Temporal::Date(_) => {}
    }

    if let Some(answer) = date_reader(value, heap, method) {
        return Ok(Some(answer));
    }
    if let Temporal::Date(date) = value
        && (method == "atTime" || method.starts_with("with"))
    {
        return date_builder(date, heap, method, args);
    }
    match (value, method) {
        (Temporal::Date(date), _) if method.starts_with("plus") || method.starts_with("minus") => {
            let plus = method.starts_with("plus");
            let amount = if plus { number() } else { -number() };
            let unit = if plus {
                method.trim_start_matches("plus")
            } else {
                method.trim_start_matches("minus")
            };
            let moved = match unit {
                "Days" => date.plus_days(amount),
                "Weeks" => date.plus_days(amount.saturating_mul(7)),
                "Months" => date.plus_months(amount),
                "Years" => date.plus_years(amount),
                _ => {
                    return Err(VmError::UnknownIntrinsic(format!(
                        "java/time/LocalDate.{method}"
                    )));
                }
            };
            Ok(Some(JValue::Ref(Some(
                heap.intern_temporal(Temporal::Date(moved)),
            ))))
        }
        (Temporal::Date(date), "until") => date_until(date, heap, args),
        (Temporal::Date(date), "isBefore" | "isAfter" | "isEqual" | "compareTo") => {
            let Some(Temporal::Date(against)) = other(heap) else {
                return Err(throw("java.lang.ClassCastException: not a LocalDate"));
            };
            // `compareTo` answers the JDK's field-by-field difference, not a
            // normalized -1/0/1: a program that prints it sees the same number.
            if method == "compareTo" {
                return int(date_difference(date, against));
            }
            let answer = match method {
                "isBefore" => date < against,
                "isAfter" => date > against,
                _ => date == against,
            };
            Ok(Some(JValue::Int(i32::from(answer))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{}.{method}",
            value.class_name()
        ))),
    }
}

/// Write what a `PrintStream` was given where it points: a standard stream,
/// or the bytes of a `ByteArrayOutputStream` the program holds. A JDK encodes
/// with the platform charset; caturra's is UTF-8 both on the way in here and
/// on the way back out through `toString()`.
/// UTF-16 units as the bytes a `PrintStream` would write: UTF-8, with an
/// UNPAIRED surrogate replaced by `?` — which is what a JDK's encoder does
/// with a character it cannot map, and not the U+FFFD a lossy Rust decode
/// would put there.
pub(crate) fn units_to_utf8(units: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(units.len());
    for piece in char::decode_utf16(units.iter().copied()) {
        match piece {
            Ok(c) => {
                let mut buffer = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
            }
            Err(_) => out.push(b'?'),
        }
    }
    out
}

fn write_units_to_sink(
    sink: PrintSink,
    units: &[u16],
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
) {
    let bytes = units_to_utf8(units);
    match sink {
        PrintSink::Std(StdStream::Out) => console.stdout(&bytes),
        PrintSink::Std(StdStream::Err) => console.stderr(&bytes),
        PrintSink::Bytes(reference) => {
            if let Some(HeapObject::ByteStream(stream)) = heap.get_mut(reference) {
                stream.extend_from_slice(&bytes);
            }
        }
    }
}

fn write_to_sink(sink: PrintSink, text: &str, heap: &mut Heap, console: &mut dyn ConsoleIo) {
    match sink {
        PrintSink::Std(StdStream::Out) => console.stdout(text.as_bytes()),
        PrintSink::Std(StdStream::Err) => console.stderr(text.as_bytes()),
        PrintSink::Bytes(reference) => {
            if let Some(HeapObject::ByteStream(bytes)) = heap.get_mut(reference) {
                bytes.extend_from_slice(text.as_bytes());
            }
        }
    }
}

/// Instantiate an intrinsic class (the `new` opcode). Returns `None`
/// for classes the VM doesn't know how to construct.
#[must_use]
pub fn instantiate(class: &str) -> Option<HeapObject> {
    match class {
        // A bare `new Object()` — an identity-only object (used e.g. to test
        // that `equals` distinguishes an unrelated instance).
        "java/lang/Object" => Some(HeapObject::Instance {
            class_name: std::rc::Rc::from("java/lang/Object"),
            layout: std::rc::Rc::new(crate::value::ClassLayout::default()),
            fields: Vec::new(),
        }),
        "java/lang/String" => Some(HeapObject::JavaString(Vec::new())),
        "java/lang/StringBuilder" => Some(HeapObject::StringBuilder(Vec::new())),
        "java/util/Scanner" => Some(HeapObject::Scanner {
            buffer: String::new(),
            pos: 0,
            eof: false,
            // Until a `File` claims it, a Scanner reads standard input.
            stdin: true,
            closed: false,
            delimiter: None,
        }),
        "java/util/ArrayList" => Some(HeapObject::ArrayList(Vec::new())),
        "java/util/LinkedList" => Some(HeapObject::LinkedList(Vec::new())),
        "java/util/ArrayDeque" => Some(HeapObject::ArrayDeque(Vec::new())),
        "java/util/Stack" => Some(HeapObject::Stack(Vec::new())),
        "java/util/HashMap" => Some(HeapObject::HashMap(JavaHashMap::new())),
        "java/util/HashSet" => Some(HeapObject::HashSet(JavaHashMap::new())),
        // A LinkedHashMap/LinkedHashSet is the same structure iterated in
        // INSERTION order — which is the order the entries are stored in
        // anyway, so only the derived bucket order is skipped.
        "java/util/LinkedHashMap" => Some(HeapObject::HashMap(JavaHashMap::linked())),
        "java/util/LinkedHashSet" => Some(HeapObject::HashSet(JavaHashMap::linked())),
        // The enum-keyed collections ARE the sorted ones underneath: an enum's
        // natural ordering is its ordinal, which is the order a JDK's
        // EnumMap/EnumSet iterate. What they do not share is the class they
        // report and their tolerance of a null probe, and the interpreter
        // records both when it builds one.
        "java/util/TreeSet" | "java/util/EnumSet" => Some(HeapObject::TreeSet {
            values: Vec::new(),
            comparator: None,
        }),
        "java/util/PriorityQueue" => Some(HeapObject::PriorityQueue {
            heap: Vec::new(),
            comparator: None,
        }),
        "java/util/TreeMap" | "java/util/EnumMap" => Some(HeapObject::TreeMap {
            entries: Vec::new(),
            comparator: None,
        }),
        "java/io/File" => Some(HeapObject::File(String::new())),
        "java/io/ByteArrayOutputStream" => Some(HeapObject::ByteStream(Vec::new())),
        // Where it writes is decided by the constructor; standard out until
        // then, which is also `new PrintStream(System.out)`.
        "java/io/PrintStream" => Some(HeapObject::PrintStream(PrintSink::Std(StdStream::Out))),
        "java/io/PrintWriter" => Some(HeapObject::Writer {
            path: String::new(),
        }),
        "java/io/BufferedReader" | "java/io/FileReader" | "java/io/InputStreamReader" => {
            Some(HeapObject::Reader {
                buffer: String::new(),
                pos: 0,
                stdin: false,
                closed: false,
            })
        }
        _ => {
            if caturra_classfile::exceptions::is_exception_class(class) {
                return Some(HeapObject::Exception {
                    class_name: caturra_classfile::exceptions::dotted(class),
                    message: None,
                    cause: None,
                    suppressed: Vec::new(),
                });
            }
            None
        }
    }
}

/// Invoke an intrinsic constructor (`invokespecial <init>`).
#[allow(clippy::too_many_lines)]
pub fn invoke_special(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    receiver: HeapRef,
    class: &str,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<(), VmError> {
    // Object's constructor does nothing — every user constructor calls
    // it as the implicit super().
    if class == "java/lang/Object" && method == "<init>" && descriptor == "()V" {
        return Ok(());
    }

    let string_arg = |heap: &Heap, value: &JValue| -> Result<String, VmError> {
        match value {
            JValue::Ref(Some(reference)) => heap
                .string_text(*reference)
                .ok_or_else(|| throw("java.lang.ClassCastException: not a String")),
            JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
            _ => Err(throw("java.lang.VerifyError: expected a String argument")),
        }
    };
    let file_arg = |heap: &Heap, value: &JValue| -> Result<String, VmError> {
        match value {
            JValue::Ref(Some(reference)) => match heap.get(*reference) {
                Some(HeapObject::File(path)) => Ok(path.clone()),
                _ => Err(throw("java.lang.ClassCastException: not a File")),
            },
            JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
            _ => Err(throw("java.lang.VerifyError: expected a File argument")),
        }
    };

    match (method, descriptor) {
        // Fully initialized at `new` (the Scanner over System.in
        // ignores the stream object — stdin is the only stream). The
        // no-arg case also covers `super()` into a library throwable
        // from a user exception class.
        ("<init>", "(Ljava/io/InputStream;)V")
            if matches!(heap.get(receiver), Some(HeapObject::Reader { .. })) =>
        {
            // `new InputStreamReader(System.in)` — read from standard input.
            if let Some(HeapObject::Reader { stdin, .. }) = heap.get_mut(receiver) {
                *stdin = true;
            }
            Ok(())
        }
        // `new PrintStream(out)` — where an OutputStream is the only thing
        // caturra has to give it, a `ByteArrayOutputStream` the program owns.
        ("<init>", "(Ljava/io/OutputStream;)V") => {
            let JValue::Ref(Some(sink)) = args[0] else {
                return Err(throw("java.lang.NullPointerException"));
            };
            if !matches!(heap.get(sink), Some(HeapObject::ByteStream(_))) {
                return Err(throw("java.lang.ClassCastException: not an OutputStream"));
            }
            if let Some(HeapObject::PrintStream(stream)) = heap.get_mut(receiver) {
                *stream = PrintSink::Bytes(sink);
            }
            Ok(())
        }
        ("<init>", "()V" | "(Ljava/io/InputStream;)V") => Ok(()),
        // `new BufferedReader(reader)` — wrap the reader, inheriting its state.
        ("<init>", "(Ljava/io/Reader;)V") => {
            let JValue::Ref(Some(inner)) = args[0] else {
                return Err(throw("java.lang.NullPointerException"));
            };
            let wrapped = match heap.get(inner) {
                Some(HeapObject::Reader {
                    buffer,
                    pos,
                    stdin,
                    closed,
                }) => (buffer.clone(), *pos, *stdin, *closed),
                _ => return Err(throw("java.lang.ClassCastException: not a Reader")),
            };
            if let Some(HeapObject::Reader {
                buffer,
                pos,
                stdin,
                closed,
            }) = heap.get_mut(receiver)
            {
                *buffer = wrapped.0;
                *pos = wrapped.1;
                *stdin = wrapped.2;
                *closed = wrapped.3;
            }
            Ok(())
        }
        // `new StringBuilder(capacity)`: the hint IS the capacity, and a
        // negative one throws, as it allocates a `char[capacity]`.
        ("<init>", "(I)V") if matches!(heap.get(receiver), Some(HeapObject::StringBuilder(_))) => {
            if let JValue::Int(capacity) = args[0] {
                if capacity < 0 {
                    return Err(throw(format!(
                        "java.lang.NegativeArraySizeException: {capacity}"
                    )));
                }
                heap.set_builder_capacity(receiver, usize::try_from(capacity).unwrap_or(0));
            }
            Ok(())
        }
        // `new HashMap<>(initialCapacity)`: unlike a builder's, a map's
        // capacity IS observable — it sets the table length, and so the
        // iteration order.
        // `new ArrayIndexOutOfBoundsException(index)` and friends: the JDK's
        // int constructors word the message themselves.
        ("<init>", "(I)V")
            if caturra_classfile::exceptions::is_exception_class(class)
                && matches!(args.first(), Some(JValue::Int(_))) =>
        {
            let Some(JValue::Int(index)) = args.first() else {
                return Ok(());
            };
            let text = match class {
                "java/lang/StringIndexOutOfBoundsException" => {
                    format!("String index out of range: {index}")
                }
                "java/lang/ArrayIndexOutOfBoundsException" => {
                    format!("Array index out of range: {index}")
                }
                _ => format!("Index out of range: {index}"),
            };
            if let Some(HeapObject::Exception { message, .. }) = heap.get_mut(receiver) {
                *message = Some(text);
            } else {
                let reference = heap.alloc_string(&text);
                if let Some(field) = heap
                    .get_mut(receiver)
                    .and_then(|object| object.field_mut("__message"))
                {
                    *field = JValue::Ref(Some(reference));
                }
            }
            Ok(())
        }
        ("<init>", "(I)V") => {
            let JValue::Int(capacity) = args[0] else {
                return Err(throw("java.lang.VerifyError: expected an int argument"));
            };
            if capacity < 0 {
                return Err(throw(format!(
                    "java.lang.IllegalArgumentException: Illegal initial capacity: {capacity}"
                )));
            }
            match heap.get_mut(receiver) {
                // `new HashSet<>(initialCapacity)` builds `new HashMap<>(cap)`,
                // so the hint reaches the backing map identically.
                Some(HeapObject::HashMap(map) | HeapObject::HashSet(map)) => {
                    // The object was already built for its class, so it knows
                    // whether it iterates in insertion order; sizing it must
                    // not throw that away (`with_capacity_hint` starts from
                    // `default()`).
                    let linked = map.is_linked();
                    *map = JavaHashMap::with_capacity_hint(capacity).as_linked(linked);
                    Ok(())
                }
                _ => Ok(()),
            }
        }
        // `new HashMap<>(otherMap)`: copies the entries; the copy's own
        // table is sized to fit, so its order may differ from the source's.
        ("<init>", "(Ljava/util/Map;)V") => {
            let JValue::Ref(Some(source)) = args[0] else {
                return Err(throw("java.lang.NullPointerException"));
            };
            let entries = match heap.get(source) {
                Some(HeapObject::HashMap(map)) => map.hashed_entries_in_order(),
                _ => return Err(throw("java.lang.ClassCastException: not a Map")),
            };
            let mut copy = JavaHashMap::sized_for(entries.len());
            for (hash, key, value) in entries {
                copy.insert_new(hash, key, value);
            }
            match heap.get_mut(receiver) {
                Some(HeapObject::HashMap(map)) => *map = copy,
                _ => return Err(throw("java.lang.ClassCastException: not a Map")),
            }
            Ok(())
        }
        // `new File(parent, child)` — the two-argument constructors, whose
        // rule is `UnixFileSystem.resolve` and is not simple concatenation: an
        // empty parent means the ROOT, a null one means there is no parent at
        // all, and an empty child is the parent itself.
        (
            "<init>",
            "(Ljava/lang/String;Ljava/lang/String;)V" | "(Ljava/io/File;Ljava/lang/String;)V",
        ) if matches!(heap.get(receiver), Some(HeapObject::File(_))) => {
            let parent = match args.first() {
                Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::JavaString(units)) => {
                        Some(abstract_path(&String::from_utf16_lossy(units)))
                    }
                    Some(HeapObject::File(path)) => Some(path.clone()),
                    _ => return Err(throw("java.lang.ClassCastException: not a String")),
                },
                Some(JValue::Ref(None)) => None,
                _ => return Err(throw("java.lang.VerifyError: expected a String argument")),
            };
            let child = match args.get(1) {
                Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::JavaString(units)) => {
                        abstract_path(&String::from_utf16_lossy(units))
                    }
                    _ => return Err(throw("java.lang.ClassCastException: not a String")),
                },
                _ => return Err(throw("java.lang.NullPointerException")),
            };
            let resolved = resolve_file(parent.as_deref(), &child);
            match heap.get_mut(receiver) {
                Some(HeapObject::File(path)) => {
                    *path = resolved;
                    Ok(())
                }
                _ => unreachable!("receiver kind checked by the guard"),
            }
        }
        ("<init>", "(Ljava/lang/String;)V") => {
            // A THROWABLE takes a null message — `new RuntimeException(null)`
            // is legal Java, `getMessage()` answers null and `toString()` is
            // the bare class name. Every other receiver here (String,
            // StringBuilder, File, Scanner, PrintWriter) really does throw on
            // one, which is why the null was rejected outright and a program
            // that passed no message died with an NPE.
            if matches!(args.first(), Some(JValue::Ref(None))) {
                return match heap.get_mut(receiver) {
                    Some(HeapObject::Exception { message, .. }) => {
                        *message = None;
                        Ok(())
                    }
                    Some(HeapObject::Instance { .. })
                        if caturra_classfile::exceptions::is_exception_class(class) =>
                    {
                        if let Some(field) = heap
                            .get_mut(receiver)
                            .and_then(|object| object.field_mut("__message"))
                        {
                            *field = JValue::NULL;
                        }
                        Ok(())
                    }
                    _ => Err(throw("java.lang.NullPointerException")),
                };
            }
            // `new StringBuilder(otherBuilder)` reaches here too (a
            // StringBuilder IS a CharSequence), so read the argument's chars
            // whether it is a String or a builder.
            // The UNITS, not the text: routing through a Rust `String` would
            // replace an unpaired surrogate rather than copy it.
            let seed: Vec<u16> = match &args[0] {
                JValue::Ref(Some(reference)) => match heap.get(*reference) {
                    Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                        units.clone()
                    }
                    _ => return Err(throw("java.lang.ClassCastException: not a String")),
                },
                JValue::Ref(None) => return Err(throw("java.lang.NullPointerException")),
                _ => return Err(throw("java.lang.VerifyError: expected a String argument")),
            };
            let text = String::from_utf16_lossy(&seed);
            // `new StringBuilder(String)` reserves the seed's length plus the
            // default 16.
            if matches!(heap.get(receiver), Some(HeapObject::StringBuilder(_))) {
                let sized = seed
                    .len()
                    .saturating_add(crate::value::DEFAULT_BUILDER_CAPACITY);
                heap.set_builder_capacity(receiver, sized);
            }
            match heap.get_mut(receiver) {
                // `new String(String)` / `new StringBuilder(String)`: seed
                // with a fresh copy of the chars (both store UTF-16 units).
                Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                    *units = seed;
                    Ok(())
                }
                Some(HeapObject::File(path)) => {
                    *path = abstract_path(&text);
                    Ok(())
                }
                Some(HeapObject::Scanner { .. }) => {
                    // `new Scanner(String)`: the whole literal is the source,
                    // fully present (eof), and closing it must not touch stdin.
                    if let Some(HeapObject::Scanner {
                        buffer,
                        pos,
                        eof,
                        stdin,
                        ..
                    }) = heap.get_mut(receiver)
                    {
                        *buffer = text;
                        *pos = 0;
                        *eof = true;
                        *stdin = false;
                    }
                    Ok(())
                }
                Some(HeapObject::Exception { message, .. }) => {
                    *message = Some(text);
                    Ok(())
                }
                // A user exception class chaining `super("message")`
                // into its library throwable parent: stash the message
                // in the slot its layout reserved for it.
                Some(HeapObject::Instance { .. })
                    if caturra_classfile::exceptions::is_exception_class(class) =>
                {
                    let reference = heap.alloc_string(&text);
                    if let Some(field) = heap
                        .get_mut(receiver)
                        .and_then(|object| object.field_mut("__message"))
                    {
                        *field = JValue::Ref(Some(reference));
                    }
                    Ok(())
                }
                Some(HeapObject::Writer { path }) => {
                    // PrintWriter(String) truncates on open (like Java)
                    // and writes through as the program prints.
                    path.clone_from(&text);
                    vfs.write_file(&text, Vec::new())
                        .map_err(|e| throw(format!("java.io.FileNotFoundException: {e}")))?;
                    Ok(())
                }
                Some(HeapObject::Reader {
                    buffer,
                    pos,
                    stdin,
                    closed,
                }) => {
                    // FileReader(String): slurp the whole file up front.
                    let content = vfs.read_file(&text).map_err(|_| {
                        throw(format!(
                            "java.io.FileNotFoundException: {text} (No such file or directory)"
                        ))
                    })?;
                    *buffer = String::from_utf8_lossy(content).into_owned();
                    *pos = 0;
                    *stdin = false;
                    *closed = false;
                    Ok(())
                }
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                ))),
            }
        }
        // Exception chaining. `new X(message, cause)` stores both; `new
        // X(cause)` derives the message from the cause's toString, as Java's
        // `Throwable(Throwable)` does. `super(...)` from a user exception class
        // into its library parent lands here too (a user Instance receiver).
        ("<init>", "(Ljava/lang/String;Ljava/lang/Throwable;)V")
            if caturra_classfile::exceptions::is_exception_class(class) =>
        {
            // A null message is legal here too: `super(null, cause)` records
            // no message and keeps the cause.
            let text = match &args[0] {
                JValue::Ref(None) => None,
                other => Some(string_arg(heap, other)?),
            };
            let cause = ref_arg(&args[1]);
            set_exception_cause(heap, receiver, class, text, cause);
            Ok(())
        }
        ("<init>", "(Ljava/lang/Throwable;)V")
            if caturra_classfile::exceptions::is_exception_class(class) =>
        {
            let cause = ref_arg(&args[0]);
            let message = cause.map(|c| throwable_to_string(heap, c));
            set_exception_cause(heap, receiver, class, message, cause);
            Ok(())
        }
        // `new FileWriter(path, append)` / `(File, append)`: the writer keeps
        // what the file already holds when the flag is set, and truncates when
        // it is not — the same two behaviours a JDK's FileWriter has.
        ("<init>", "(Ljava/lang/String;Z)V" | "(Ljava/io/File;Z)V") => {
            let target = if descriptor.starts_with("(Ljava/io/File;") {
                file_arg(heap, &args[0])?
            } else {
                match &args[0] {
                    JValue::Ref(Some(reference)) => {
                        heap.string_text(*reference).unwrap_or_default()
                    }
                    _ => return Err(throw("java.lang.NullPointerException")),
                }
            };
            let appends = matches!(args.get(1), Some(JValue::Int(flag)) if *flag != 0);
            if !appends || vfs.read_file(&target).is_err() {
                vfs.write_file(&target, Vec::new())
                    .map_err(|e| throw(format!("java.io.FileNotFoundException: {e}")))?;
            }
            if let Some(HeapObject::Writer { path }) = heap.get_mut(receiver) {
                *path = target;
            }
            Ok(())
        }
        ("<init>", "(Ljava/io/File;)V") => {
            let target = file_arg(heap, &args[0])?;
            match heap.get(receiver) {
                Some(HeapObject::Writer { .. }) => {
                    vfs.write_file(&target, Vec::new())
                        .map_err(|e| throw(format!("java.io.FileNotFoundException: {e}")))?;
                    if let Some(HeapObject::Writer { path }) = heap.get_mut(receiver) {
                        *path = target;
                    }
                    Ok(())
                }
                Some(HeapObject::Scanner { .. }) => {
                    // Scanner(File): slurp the whole file up front.
                    let content = vfs
                        .read_file(&target)
                        .map_err(|_| {
                            throw(format!(
                                "java.io.FileNotFoundException: {target} \
                                 (No such file or directory)"
                            ))
                        })?
                        .to_vec();
                    let text = String::from_utf8_lossy(&content).into_owned();
                    if let Some(HeapObject::Scanner {
                        buffer,
                        eof,
                        pos,
                        stdin,
                        ..
                    }) = heap.get_mut(receiver)
                    {
                        *buffer = text;
                        *pos = 0;
                        *eof = true;
                        // A file scanner: closing it must not close stdin.
                        *stdin = false;
                    }
                    Ok(())
                }
                Some(HeapObject::Reader { .. }) => {
                    // FileReader(File): slurp the whole file up front.
                    let content = vfs
                        .read_file(&target)
                        .map_err(|_| {
                            throw(format!(
                                "java.io.FileNotFoundException: {target} \
                                 (No such file or directory)"
                            ))
                        })?
                        .to_vec();
                    let text = String::from_utf8_lossy(&content).into_owned();
                    if let Some(HeapObject::Reader {
                        buffer,
                        pos,
                        stdin,
                        closed,
                    }) = heap.get_mut(receiver)
                    {
                        *buffer = text;
                        *pos = 0;
                        *stdin = false;
                        *closed = false;
                    }
                    Ok(())
                }
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                ))),
            }
        }
        // `new String(bytes)` / `new String(bytes, charset)` — the decoder,
        // told which charset (UTF-8 by default, as the JDK's default is here).
        // Malformed input decodes to U+FFFD rather than throwing, which is what
        // a program reading a file relies on.
        ("<init>", "([B)V" | "([BLjava/nio/charset/Charset;)V" | "([BLjava/lang/String;)V") => {
            let bytes = byte_array_values(heap, &args[0])?;
            let charset = charset_argument(heap, args.get(1))?;
            let units = decode_charset(&bytes, &charset);
            if let Some(HeapObject::JavaString(target)) = heap.get_mut(receiver) {
                *target = units;
                Ok(())
            } else {
                Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                )))
            }
        }
        // ...and the SUBRANGE forms, which read a used prefix of a buffer.
        (
            "<init>",
            "([BII)V" | "([BIILjava/nio/charset/Charset;)V" | "([BIILjava/lang/String;)V",
        ) => {
            let bytes = byte_array_values(heap, &args[0])?;
            let (offset, count) = match (&args[1], &args[2]) {
                (JValue::Int(offset), JValue::Int(count)) => (*offset, *count),
                _ => return Err(throw("java.lang.VerifyError: expected two ints")),
            };
            let end = offset.checked_add(count);
            if offset < 0
                || count < 0
                || end.is_none_or(|end| end > i32::try_from(bytes.len()).unwrap_or(i32::MAX))
            {
                return Err(throw(format!(
                    "java.lang.StringIndexOutOfBoundsException: offset {offset}, count {count}, \
                     length {}",
                    bytes.len()
                )));
            }
            let start = usize::try_from(offset).unwrap_or(0);
            let taken = usize::try_from(count).unwrap_or(0);
            let charset = charset_argument(heap, args.get(3))?;
            let units = decode_charset(&bytes[start..start + taken], &charset);
            if let Some(HeapObject::JavaString(target)) = heap.get_mut(receiver) {
                *target = units;
                Ok(())
            } else {
                Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                )))
            }
        }
        // `new String(char[])` — char arrays are stored as int arrays.
        ("<init>", "([C)V") => {
            let units: Vec<u16> = match &args[0] {
                JValue::Ref(Some(reference)) => match heap.get(*reference) {
                    Some(HeapObject::IntArray(_, values)) => values
                        .iter()
                        .map(|v| u16::try_from(v & 0xFFFF).unwrap_or(0))
                        .collect(),
                    _ => return Err(throw("java.lang.ClassCastException: not a char[]")),
                },
                _ => return Err(throw("java.lang.NullPointerException")),
            };
            if let Some(HeapObject::JavaString(target)) = heap.get_mut(receiver) {
                *target = units;
                Ok(())
            } else {
                Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                )))
            }
        }
        // `new String(chars, offset, count)` — a SUBRANGE of the array, which
        // is how a char buffer's used prefix becomes a String. The JDK checks
        // the range and words the failure with all three numbers.
        ("<init>", "([CII)V") => {
            let units = char_array_units(heap, &args[0])?;
            let (offset, count) = match (&args[1], &args[2]) {
                (JValue::Int(offset), JValue::Int(count)) => (*offset, *count),
                _ => return Err(throw("java.lang.VerifyError: expected two ints")),
            };
            let end = offset.checked_add(count);
            if offset < 0
                || count < 0
                || end.is_none_or(|end| end > i32::try_from(units.len()).unwrap_or(i32::MAX))
            {
                return Err(throw(format!(
                    "java.lang.StringIndexOutOfBoundsException: offset {offset}, count {count}, \
                     length {}",
                    units.len()
                )));
            }
            let start = usize::try_from(offset).unwrap_or(0);
            let taken = usize::try_from(count).unwrap_or(0);
            let slice = units[start..start + taken].to_vec();
            if let Some(HeapObject::JavaString(target)) = heap.get_mut(receiver) {
                *target = slice;
                Ok(())
            } else {
                Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                )))
            }
        }
        // `new ArrayList<>(collection)` — copy the source collection's items.
        ("<init>", "(Ljava/util/Collection;)V") => {
            let items = match args.first() {
                // `list_values` reads through an `Arrays.asList` view too, so
                // `new ArrayList<>(Arrays.asList(...))` — the standard way to
                // get a mutable copy — works. (The richer collection sources
                // are handled by the interpreter's own copy path; this arm
                // only sees the list-shaped ones.)
                Some(JValue::Ref(Some(reference))) => match heap.list_values(*reference) {
                    Some(items) => items.clone(),
                    None => return Err(throw("java.lang.ClassCastException: not a Collection")),
                },
                Some(JValue::Ref(None)) => return Err(throw("java.lang.NullPointerException")),
                _ => Vec::new(),
            };
            if let Some(HeapObject::ArrayList(target)) = heap.get_mut(receiver) {
                *target = items;
                Ok(())
            } else {
                Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                )))
            }
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{class}.{method}{descriptor}"
        ))),
    }
}

/// Invoke an intrinsic instance method. Returns `Ok(None)` for `void`,
/// `Ok(Some(value))` for a result, or `Err` if the member is not an
/// intrinsic the VM knows.
#[allow(clippy::too_many_arguments)] // one boundary call from the dispatch loop
#[allow(clippy::too_many_lines)] // one intrinsic-dispatch matrix
pub fn invoke_virtual(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    vfs: &mut VirtualFileSystem,
    receiver: HeapRef,
    class: &str,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let receiver_object = heap.get(receiver).ok_or_else(|| VmError::MalformedClass {
        name: class.to_owned(),
        reason: format!("dangling heap reference {receiver}"),
    })?;

    // Boxed wrapper methods: unboxing accessors and Object methods.
    if let HeapObject::Boxed { class_name, value } = receiver_object {
        let class_name = class_name.clone();
        let value = *value;
        return boxed_virtual(heap, &class_name, value, method, args);
    }
    // `Object`'s `equals`/`hashCode`, for the classes that do not override
    // them. Every receiver kind used to have to remember these in its own
    // arms, and the ones that forgot — a PriorityQueue, a Comparator, a
    // Scanner, a Stream — were an "unknown native member" abort for a call
    // the compiler had already accepted.
    //
    // The kinds are listed rather than defaulted, so that a VALUE-based class
    // added later (its `equals` compares contents) cannot silently fall in
    // here and compare by identity instead.
    if uses_identity_equality(receiver_object) {
        match (method, args) {
            ("hashCode", []) => return Ok(Some(JValue::Int(identity_hash(receiver)))),
            ("equals", [argument]) => {
                let equal = matches!(argument, JValue::Ref(Some(other)) if *other == receiver);
                return Ok(Some(JValue::Int(i32::from(equal))));
            }
            _ => {}
        }
    }
    match (receiver_object, method) {
        // A `DateTimeFormatter` formats a value handed TO it, which is the
        // other way of writing `value.format(formatter)`.
        (HeapObject::DateFormat(_), "format") => {
            let Some(JValue::Ref(Some(target))) = args.first() else {
                return Err(throw("java.lang.NullPointerException: temporal"));
            };
            let Some(HeapObject::Temporal(value)) = heap.get(*target) else {
                return Err(throw("java.lang.ClassCastException: not a date or a time"));
            };
            let value = *value;
            format_temporal(value, heap, receiver)
        }
        (HeapObject::DateFormat(kind), "toString") => {
            // A JDK prints the PRINTER it built, not the pattern it was given.
            let text = match kind {
                crate::value::DateFormatKind::Pattern(pattern) => {
                    crate::time::parse_pattern(pattern).map_or_else(
                        |_| pattern.clone(),
                        |pieces| crate::time::describe_pieces(&pieces),
                    )
                }
                crate::value::DateFormatKind::Iso(which) => {
                    crate::time::iso_description(*which).to_owned()
                }
            };
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        // `java.time.LocalDate` and the two enums it answers with. Every
        // operation makes a NEW value; nothing here mutates.
        (HeapObject::Temporal(value), _) => temporal_method(*value, heap, method, args),
        // `java.io.ByteArrayOutputStream` — the bytes a capture collected.
        // `toString()` decodes them as UTF-8, which is what wrote them.
        (HeapObject::ByteStream(bytes), "toString") => {
            let text = String::from_utf8_lossy(bytes).into_owned();
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        (HeapObject::ByteStream(bytes), "size") => Ok(Some(JValue::Int(
            i32::try_from(bytes.len()).unwrap_or(i32::MAX),
        ))),
        (HeapObject::ByteStream(bytes), "toByteArray") => {
            let copied: Vec<i8> = bytes.iter().map(|b| (*b).cast_signed()).collect();
            Ok(Some(JValue::Ref(Some(
                heap.alloc(HeapObject::ByteArray(copied)),
            ))))
        }
        (HeapObject::ByteStream(_), "reset") => {
            if let Some(HeapObject::ByteStream(bytes)) = heap.get_mut(receiver) {
                bytes.clear();
            }
            Ok(None)
        }

        (HeapObject::ByteStream(_), "write") => {
            if let [JValue::Int(byte)] = args {
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let unit = (*byte & 0xFF) as u8;
                if let Some(HeapObject::ByteStream(bytes)) = heap.get_mut(receiver) {
                    bytes.push(unit);
                }
            }
            Ok(None)
        }
        // Neither has anything to flush — a byte stream is in memory and a
        // print stream writes straight through — and closing either "has no
        // effect" (the JDK says so of a ByteArrayOutputStream in as many
        // words, and standard out stays usable).
        (HeapObject::ByteStream(_) | HeapObject::PrintStream(_), "flush" | "close") => Ok(None),
        (HeapObject::PrintStream(_), "checkError") => Ok(Some(JValue::Int(0))),
        (HeapObject::PrintStream(stream), "printf" | "format") => {
            let stream = *stream;
            let template = match args.first() {
                Some(JValue::Ref(Some(reference))) => {
                    heap.string_text(*reference).unwrap_or_default()
                }
                Some(JValue::Ref(None)) => {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException: format is null",
                    )));
                }
                _ => String::new(),
            };
            let format_args = crate::format::args_from_descriptor(heap, descriptor, &args[1..])?;
            // The JDK's Formatter writes to its destination as it goes, so a
            // specifier that throws leaves everything before it PRINTED —
            // `printf("a%dz%s", 5)` shows "a5z" and then throws.
            let (produced, result) =
                crate::format::java_format_partial(heap, &template, &format_args);
            let text = match result {
                Ok(text) => text,
                Err(error) => {
                    write_units_to_sink(stream, &produced, heap, console);
                    return Err(error);
                }
            };
            write_units_to_sink(stream, &text, heap, console);
            // Both answer the stream in Java, and the compiler emits the
            // descriptor that says so where the value is USED — returning
            // nothing there would underflow the operand stack, and returning
            // something where the statement path emitted `)V` would leave one
            // behind.
            if descriptor.ends_with(")Ljava/io/PrintStream;") {
                return Ok(Some(JValue::Ref(Some(receiver))));
            }
            Ok(None)
        }
        // `write(int)` writes the low byte as one character; `append(...)`
        // is `print` that answers the stream (which caturra discards).
        (HeapObject::PrintStream(stream), "write" | "append") => {
            let stream = *stream;
            let text = match (method, args) {
                ("write", [JValue::Int(byte)]) => {
                    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    let unit = u32::from((*byte & 0xFF) as u8);
                    char::from_u32(unit).map(String::from).unwrap_or_default()
                }
                _ => print_argument_text(heap, descriptor, args)?,
            };
            if stream == PrintSink::Std(StdStream::Out) && console.capturing() {
                console.capture_message(&text);
            } else {
                write_to_sink(stream, &text, heap, console);
            }
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        (HeapObject::PrintStream(stream), "print" | "println") => {
            let stream = *stream;
            let text = print_argument_text(heap, descriptor, args)?;
            // While `SystemOutTestRunner` is capturing, each print/println call
            // on `System.out` is one message (the argument, without the
            // println newline) — matching javabuilder's per-call messages.
            if stream == PrintSink::Std(StdStream::Out) && console.capturing() {
                console.capture_message(&text);
            } else {
                let mut out = text;
                if method == "println" {
                    out.push('\n');
                }
                write_to_sink(stream, &out, heap, console);
            }
            Ok(None)
        }
        // The summary objects — five stored numbers, read back. The integral
        // kinds share a variant; only the class name they print and the width
        // of their accessors differ.
        (
            HeapObject::SummaryStats {
                count,
                sum,
                min,
                max,
                kind,
            },
            _,
        ) => {
            let (count, sum, min, max, kind) = (*count, *sum, *min, *max, *kind);
            let integral = kind == crate::value::SummaryKind::Long;
            #[allow(clippy::cast_precision_loss)] // the JDK divides in double too
            let average = if count == 0 {
                0.0
            } else {
                sum as f64 / count as f64
            };
            Ok(Some(match method {
                "getCount" => JValue::Long(count),
                // An `IntSummaryStatistics` sums into a long but reports its
                // bounds as ints, which is the one place the two kinds differ
                // in more than a name.
                "getSum" => JValue::Long(sum),
                "getMin" if integral => JValue::Long(min),
                "getMax" if integral => JValue::Long(max),
                "getMin" => JValue::Int(i32::try_from(min).unwrap_or(i32::MAX)),
                "getMax" => JValue::Int(i32::try_from(max).unwrap_or(i32::MIN)),
                "getAverage" => JValue::Double(average),
                "toString" => {
                    let text = summary_text(count, sum, min, max, kind);
                    JValue::Ref(Some(heap.alloc_string(&text)))
                }
                other => {
                    return Err(VmError::UnknownIntrinsic(format!(
                        "{}.{other}",
                        kind.class()
                    )));
                }
            }))
        }
        (
            HeapObject::DoubleSummaryStats {
                count,
                sum,
                min,
                max,
            },
            _,
        ) => {
            let (count, sum, min, max) = (*count, *sum, *min, *max);
            #[allow(clippy::cast_precision_loss)]
            let average = if count == 0 { 0.0 } else { sum / count as f64 };
            Ok(Some(match method {
                "getCount" => JValue::Long(count),
                "getSum" => JValue::Double(sum),
                "getMin" => JValue::Double(min),
                "getMax" => JValue::Double(max),
                "getAverage" => JValue::Double(average),
                // Every number here prints with `%f` — including the
                // `Infinity`/`-Infinity` an empty summary keeps, which `%f`
                // spells out rather than padding with zeros.
                "toString" => {
                    let text = double_summary_text(count, sum, min, max);
                    JValue::Ref(Some(heap.alloc_string(&text)))
                }
                other => {
                    return Err(VmError::UnknownIntrinsic(format!(
                        "DoubleSummaryStatistics.{other}"
                    )));
                }
            }))
        }
        (HeapObject::Iterator { .. }, _) => iterator_method(heap, receiver, method, args),
        (HeapObject::StringBuilder(_), _) => {
            builder_method(heap, receiver, method, descriptor, args)
        }
        (HeapObject::JavaString(_), _) => string_method(heap, receiver, method, args),
        (HeapObject::Scanner { .. }, _) => scanner_method(heap, console, receiver, method, args),
        (HeapObject::Reader { .. }, _) => reader_method(heap, console, receiver, method),
        (
            HeapObject::ArrayList(_) | HeapObject::LinkedList(_) | HeapObject::ArrayBackedList(_),
            _,
        ) => list_method(heap, receiver, method, descriptor, args),
        // An ArrayDeque shares the LinkedList `Deque` semantics (head-based
        // push/pop through `list_method`) but forbids null elements, so guard
        // every insertion before delegating.
        (HeapObject::ArrayDeque(_), _) => {
            array_deque_reject_null(heap, method, args)?;
            list_method(heap, receiver, method, descriptor, args)
        }
        // A `Stack` is a `List`, so it shares `list_method` for everything but
        // the five LIFO operations, whose `top`-end semantics and
        // `EmptyStackException` differ from the deque/list methods of the same
        // spelling.
        (HeapObject::Stack(_), _) => match stack_method(heap, receiver, method, args)? {
            Some(value) => Ok(Some(value)),
            None => list_method(heap, receiver, method, descriptor, args),
        },
        // A comparator built by `naturalOrder`/`comparing`/a lambda has no
        // text a program can depend on: a real JDK prints its LAMBDA class,
        // `Main$$Lambda$14/0x00000008000c9440@2f4d3709`, which differs between
        // runs and between JVMs (`Comparator.naturalOrder()` prints the enum
        // constant `INSTANCE` instead). So this is answerable but not
        // matchable, and it is written in `Object`'s shape — a class name and
        // an identity hash — rather than invented to look like a JDK's.
        // A comparator the PROGRAM declared is an ordinary instance and prints
        // its own `toString`; this covers only the ones caturra synthesized.
        (HeapObject::Comparator(_), "toString") => {
            let text = format!("java.util.Comparator$$Lambda@{:x}", identity_hash(receiver));
            let reference = heap.alloc_string(&text);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // A `Collector` is a value a program passes on, and printing one is
        // usually a mistake — but it must not be an internal error. The JDK
        // gives it `Object`'s shape, and names the class it really is.
        (HeapObject::Collector(_), "toString") => {
            let text = collector_text(receiver);
            let reference = heap.alloc_string(&text);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // Two Files naming one path are equal (the JDK compares the abstract
        // pathname).
        (HeapObject::File(path), "equals") => {
            let path = path.clone();
            let equal = match args.first() {
                Some(JValue::Ref(Some(other))) => {
                    matches!(heap.get(*other), Some(HeapObject::File(theirs)) if *theirs == path)
                }
                _ => false,
            };
            Ok(Some(JValue::Int(i32::from(equal))))
        }
        (HeapObject::File(_), _) => file_method(heap, vfs, receiver, method, args),
        (HeapObject::Path(_), _) => path_method(heap, receiver, method, args),
        // A `Charset` is its NAME: `toString`, `name` and `displayName` all
        // answer it, and two charsets are equal when they name the same one.
        (HeapObject::Charset(name), "toString" | "name" | "displayName") => {
            let name = name.clone();
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&name)))))
        }
        // A `Pattern`: its own text, a `Matcher` over some input, and the two
        // split forms — the same splitter `String.split` uses.
        (HeapObject::Pattern { source, .. }, "pattern" | "toString") => {
            let text = String::from_utf16_lossy(source);
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        (HeapObject::Pattern { .. }, "flags") => {
            let flags = match heap.get(receiver) {
                Some(HeapObject::Pattern { flags, .. }) => *flags,
                _ => 0,
            };
            Ok(Some(JValue::Int(flags)))
        }
        (HeapObject::Pattern { .. }, "matcher") => {
            let input = string_units(heap, args.first().unwrap_or(&JValue::NULL))?;
            let end = input.len();
            Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::Matcher {
                pattern: receiver,
                input,
                at: 0,
                last: None,
                region: (0, end),
                anchoring: true,
                transparent: false,
                hit_end: false,
                require_end: false,
                appended: 0,
            })))))
        }
        // The predicate a pattern answers, asked directly.
        (HeapObject::RegexPredicate { pattern, whole }, "test") => {
            let (pattern, whole) = (*pattern, *whole);
            let text = string_units(heap, args.first().unwrap_or(&JValue::NULL))?;
            Ok(Some(JValue::Int(i32::from(regex_predicate(
                heap, pattern, whole, &text,
            )?))))
        }
        // `asPredicate()` tests whether the pattern is FOUND;
        // `asMatchPredicate()` (Java 11) whether it matches the whole input.
        (HeapObject::Pattern { .. }, "asPredicate" | "asMatchPredicate") => Ok(Some(JValue::Ref(
            Some(heap.alloc(HeapObject::RegexPredicate {
                pattern: receiver,
                whole: method == "asMatchPredicate",
            })),
        ))),
        // A frozen match answers the same four questions a Matcher does.
        (HeapObject::MatchResult { input, groups }, _) => {
            let (input, groups) = (input.clone(), groups.clone());
            match_result_method(heap, receiver, &input, &groups, method, args)
        }
        (HeapObject::Pattern { source, flags }, "split" | "splitAsStream") => {
            let source = fold_regex_flags(source, *flags);
            let input = string_units(heap, args.first().unwrap_or(&JValue::NULL))?;
            let limit = match args.get(1) {
                Some(JValue::Int(limit)) => *limit,
                _ => 0,
            };
            let parts = split_regex(&input, &source, limit)?;
            let refs: Vec<JValue> = parts
                .into_iter()
                .map(|part| {
                    let text = String::from_utf16_lossy(&part);
                    JValue::Ref(Some(heap.alloc_string(&text)))
                })
                .collect();
            // `splitAsStream` is the same split, handed over as a pipeline.
            if method == "splitAsStream" {
                return Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::Stream {
                    source: crate::value::StreamSource::Fixed(refs),
                    ops: Vec::new(),
                })))));
            }
            Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::RefArray(
                String::from("java/lang/String"),
                refs,
            ))))))
        }
        (HeapObject::Matcher { .. }, _) => matcher_method(heap, receiver, method, args),
        // A `Charset` hashes as its name, the same answer a collection gets
        // from `native_hash` — written in both places, so a program can ask
        // either way and a `HashSet<Charset>` holds one of each charset.
        (HeapObject::Charset(name), "hashCode") => {
            Ok(Some(JValue::Int(java_string_hash(&name.clone()))))
        }
        (HeapObject::Charset(name), "equals") => {
            let name = name.clone();
            let equal = match args.first() {
                Some(JValue::Ref(Some(other))) => {
                    matches!(heap.get(*other), Some(HeapObject::Charset(theirs)) if *theirs == name)
                }
                _ => false,
            };
            Ok(Some(JValue::Int(i32::from(equal))))
        }
        (
            HeapObject::Exception {
                class_name,
                message,
                ..
            },
            "printStackTrace",
        ) => {
            // Real Java writes the trace to System.err. caturra keeps no frame
            // line info, so emit the exception's header line (the part
            // students actually read).
            let header = match message {
                Some(message) => format!("{class_name}: {message}\n"),
                None => format!("{class_name}\n"),
            };
            console.stderr(header.as_bytes());
            Ok(None)
        }
        (
            HeapObject::Exception {
                class_name,
                message,
                ..
            },
            "getMessage" | "toString" | "getLocalizedMessage",
        ) => {
            let rendered = if method == "getMessage" || method == "getLocalizedMessage" {
                match message {
                    Some(message) => message.clone(),
                    None => return Ok(Some(JValue::NULL)),
                }
            } else {
                match message {
                    Some(message) => format!("{class_name}: {message}"),
                    None => class_name.clone(),
                }
            };
            let reference = heap.alloc_string(&rendered);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // A `PatternSyntaxException` says what was wrong, and where. Its
        // message is BUILT from those three, in the JDK's documented layout,
        // so reading them back out of it keeps one source of truth.
        (
            HeapObject::Exception {
                class_name,
                message,
                ..
            },
            "getDescription" | "getPattern" | "getIndex",
        ) if class_name == "java.util.regex.PatternSyntaxException" => {
            let message = message.clone().unwrap_or_default();
            let mut lines = message.lines();
            let first = lines.next().unwrap_or_default().to_owned();
            let (description, index) = match first.rsplit_once(" near index ") {
                Some((description, index)) => {
                    (description.to_owned(), index.parse::<i32>().unwrap_or(-1))
                }
                None => (first, -1),
            };
            match method {
                "getIndex" => Ok(Some(JValue::Int(index))),
                "getDescription" => Ok(Some(JValue::Ref(Some(heap.alloc_string(&description))))),
                _ => {
                    let pattern = lines.next().unwrap_or_default().to_owned();
                    Ok(Some(JValue::Ref(Some(heap.alloc_string(&pattern)))))
                }
            }
        }
        // `getCause()` — the chained cause, or null.
        (HeapObject::Exception { cause, .. }, "getCause") => Ok(Some(JValue::Ref(*cause))),
        // Every object has an identity hash; a throwable does not override it.
        (HeapObject::Exception { .. }, "hashCode") => Ok(Some(JValue::Int(
            i32::try_from(receiver).unwrap_or(i32::MAX),
        ))),
        // `initCause(Throwable)` sets the cause ONCE and returns `this`. A
        // second call is an error, not an overwrite: the JDK refuses so that a
        // cause set at construction cannot be silently replaced.
        (HeapObject::Exception { .. }, "initCause") => {
            let cause = args.first().and_then(ref_arg);
            if let Some(JValue::Ref(Some(_))) = args.first().copied()
                && cause == Some(receiver)
            {
                return Err(throw(
                    "java.lang.IllegalArgumentException: Self-causation not permitted",
                ));
            }
            if let Some(HeapObject::Exception {
                cause: Some(existing),
                ..
            }) = heap.get(receiver)
            {
                let existing = *existing;
                return Err(throw(format!(
                    "java.lang.IllegalStateException: Can't overwrite cause with {}",
                    match cause {
                        Some(_) => throwable_to_string(heap, cause.unwrap_or(existing)),
                        None => String::from("a null"),
                    }
                )));
            }
            if let Some(HeapObject::Exception { cause: slot, .. }) = heap.get_mut(receiver) {
                *slot = cause;
            }
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        // `getSuppressed()` — the exceptions a try-with-resources attached
        // because `close()` threw while this one was already propagating.
        (HeapObject::Exception { suppressed, .. }, "getSuppressed") => {
            let elements: Vec<JValue> = suppressed
                .iter()
                .map(|reference| JValue::Ref(Some(*reference)))
                .collect();
            let reference = heap.alloc(HeapObject::RefArray(
                String::from("[Ljava/lang/Throwable;"),
                elements,
            ));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // `addSuppressed(t)` — what the try-with-resources desugaring calls
        // when a resource's `close()` throws and the body's exception wins.
        (HeapObject::Exception { .. }, "addSuppressed") => {
            match args.first().copied() {
                // An exception cannot suppress ITSELF: the JDK rejects it
                // outright rather than building a self-referential chain that
                // `printStackTrace` would never finish rendering. A resource
                // whose `close()` rethrows the very exception the body threw
                // reaches exactly this case.
                Some(JValue::Ref(Some(extra))) if extra == receiver => {
                    return Err(throw(
                        "java.lang.IllegalArgumentException: Self-suppression not permitted",
                    ));
                }
                Some(JValue::Ref(Some(extra))) => {
                    if let Some(HeapObject::Exception { suppressed, .. }) = heap.get_mut(receiver) {
                        suppressed.push(extra);
                    }
                }
                Some(JValue::Ref(None)) => {
                    return Err(throw(
                        "java.lang.NullPointerException: Cannot suppress a null exception.",
                    ));
                }
                _ => {}
            }
            Ok(None)
        }
        // The same two on a USER exception class, whose suppressed list lives
        // in the `__suppressed` slot its layout reserves (a library throwable
        // carries one on its own heap object). Without these, an ordinary
        // `class AppException extends RuntimeException` could not be the
        // primary exception of a try-with-resources whose `close()` also
        // threw: `addSuppressed` was an unknown native member.
        (HeapObject::Instance { .. }, "getSuppressed") => {
            let elements = match heap.get(receiver).and_then(|o| o.field("__suppressed")) {
                Some(JValue::Ref(Some(array))) => match heap.get(array) {
                    Some(HeapObject::RefArray(_, values)) => values.clone(),
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            };
            let reference = heap.alloc(HeapObject::RefArray(
                String::from("[Ljava/lang/Throwable;"),
                elements,
            ));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        (HeapObject::Instance { .. }, "addSuppressed") => {
            match args.first().copied() {
                Some(JValue::Ref(Some(extra))) if extra == receiver => {
                    return Err(throw(
                        "java.lang.IllegalArgumentException: Self-suppression not permitted",
                    ));
                }
                Some(JValue::Ref(Some(extra))) => {
                    let mut values = match heap.get(receiver).and_then(|o| o.field("__suppressed"))
                    {
                        Some(JValue::Ref(Some(array))) => match heap.get(array) {
                            Some(HeapObject::RefArray(_, values)) => values.clone(),
                            _ => Vec::new(),
                        },
                        _ => Vec::new(),
                    };
                    values.push(JValue::Ref(Some(extra)));
                    let array = heap.alloc(HeapObject::RefArray(
                        String::from("[Ljava/lang/Throwable;"),
                        values,
                    ));
                    if let Some(field) = heap
                        .get_mut(receiver)
                        .and_then(|object| object.field_mut("__suppressed"))
                    {
                        *field = JValue::Ref(Some(array));
                    }
                }
                Some(JValue::Ref(None)) => {
                    return Err(throw(
                        "java.lang.NullPointerException: Cannot suppress a null exception.",
                    ));
                }
                _ => {}
            }
            Ok(None)
        }
        (HeapObject::Writer { .. }, _) => {
            writer_method(heap, vfs, receiver, method, descriptor, args)
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{class}.{method}{descriptor}"
        ))),
    }
}

/// Whether this object inherits `Object`'s IDENTITY `equals`/`hashCode` —
/// its Java class does not override them.
///
/// Deliberately a list and not a default. The value-based classes (`String`,
/// every collection, `Optional`, `File`, `StackTraceElement`, `Map.Entry`)
/// answer in their own arms, and a new one that forgot would rather be an
/// honest abort than compare by identity behind the program's back.
///
/// A user `Instance` is absent on purpose: its `equals` may be overridden in
/// the program's own source, which virtual dispatch resolves before any
/// intrinsic is consulted.
/// `String.hashCode` (JLS: `s[0]*31^(n-1) + ...`), over UTF-16 units.
pub(crate) fn java_string_hash_units(units: &[u16]) -> i32 {
    units.iter().fold(0i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(*unit))
    })
}

/// `String.hashCode` of a Rust string, for the value-based classes whose hash
/// the JDK builds out of its fields' hashes.
pub(crate) fn java_string_hash(text: &str) -> i32 {
    text.encode_utf16().fold(0i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(unit))
    })
}

pub(crate) fn uses_identity_equality(object: &HeapObject) -> bool {
    matches!(
        object,
        HeapObject::Scanner { .. }
            | HeapObject::Reader { .. }
            | HeapObject::Writer { .. }
            | HeapObject::PrintStream(_)
            | HeapObject::InputStream
            | HeapObject::PriorityQueue { .. }
            | HeapObject::Stream { .. }
            | HeapObject::Comparator(_)
            | HeapObject::Collector(_)
            | HeapObject::SummaryStats { .. }
            | HeapObject::DoubleSummaryStats { .. }
            | HeapObject::StringBuilder(_)
            | HeapObject::Exception { .. }
            | HeapObject::Iterator { .. }
            // The regex trio: a JDK's Pattern, Matcher and MatchResult all
            // inherit `Object.equals`, so two equal patterns are not equal.
            | HeapObject::Pattern { .. }
            | HeapObject::Matcher { .. }
            | HeapObject::MatchResult { .. }
            | HeapObject::RegexPredicate { .. }
    )
}

fn throw(message: impl Into<String>) -> VmError {
    VmError::UncaughtException(message.into())
}

/// The heap reference an argument holds (`None` for `null` or a non-reference).
fn ref_arg(value: &JValue) -> Option<HeapRef> {
    match value {
        JValue::Ref(r) => *r,
        _ => None,
    }
}

/// A throwable's default `toString`: its binary class name, plus `": message"`
/// when it has one. Used to derive `new X(cause)`'s message, as
/// `Throwable(Throwable)` does. (A user override of `toString` is not consulted
/// — this is only the constructor's message derivation.)
fn throwable_to_string(heap: &Heap, reference: HeapRef) -> String {
    match heap.get(reference) {
        Some(HeapObject::Exception {
            class_name,
            message,
            ..
        }) => match message {
            Some(message) => format!("{class_name}: {message}"),
            None => class_name.clone(),
        },
        Some(HeapObject::Instance { class_name, .. }) => {
            let message = heap
                .get(reference)
                .and_then(|o| o.field("__message"))
                .and_then(|v| match v {
                    JValue::Ref(Some(r)) => heap.string_text(r),
                    _ => None,
                });
            let dotted = class_name.replace('/', ".");
            match message {
                Some(message) => format!("{dotted}: {message}"),
                None => dotted,
            }
        }
        _ => String::from("java.lang.Throwable"),
    }
}

/// Store an exception's message and chained cause: a library `Exception`
/// object sets its own fields; a user exception `Instance` (reached through
/// `super(message, cause)`) stashes them in the `__message`/`__cause` slots its
/// layout reserves.
fn set_exception_cause(
    heap: &mut Heap,
    receiver: HeapRef,
    class: &str,
    message: Option<String>,
    cause: Option<HeapRef>,
) {
    match heap.get(receiver) {
        Some(HeapObject::Exception { .. }) => {
            if let Some(HeapObject::Exception {
                message: slot,
                cause: cause_slot,
                ..
            }) = heap.get_mut(receiver)
            {
                *slot = message;
                *cause_slot = cause;
            }
        }
        Some(HeapObject::Instance { .. })
            if caturra_classfile::exceptions::is_exception_class(class) =>
        {
            if let Some(text) = message {
                let reference = heap.alloc_string(&text);
                if let Some(field) = heap
                    .get_mut(receiver)
                    .and_then(|object| object.field_mut("__message"))
                {
                    *field = JValue::Ref(Some(reference));
                }
            }
            if let Some(field) = heap
                .get_mut(receiver)
                .and_then(|object| object.field_mut("__cause"))
            {
                *field = JValue::Ref(cause);
            }
        }
        _ => {}
    }
}

/// `java.lang.String` instance methods over UTF-16 code units.
#[allow(clippy::too_many_lines)]
fn string_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let Some(HeapObject::JavaString(units)) = heap.get(receiver) else {
        unreachable!("receiver kind checked by caller");
    };
    let units = units.clone();
    let len = i32::try_from(units.len()).unwrap_or(i32::MAX);

    // Resolve a string argument's code units (null → NPE like Java).
    let arg_units = |value: &JValue| -> Result<Vec<u16>, VmError> {
        match value {
            JValue::Ref(Some(reference)) => match heap.get(*reference) {
                // A StringBuilder is a CharSequence too: `str.contentEquals(sb)`.
                Some(HeapObject::JavaString(other) | HeapObject::StringBuilder(other)) => {
                    Ok(other.clone())
                }
                _ => Err(throw("java.lang.ClassCastException: not a String")),
            },
            JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
            _ => Err(throw("java.lang.VerifyError: expected a String argument")),
        }
    };

    match (method, args) {
        ("length", []) => Ok(Some(JValue::Int(len))),
        ("isEmpty", []) => Ok(Some(JValue::Int(i32::from(units.is_empty())))),
        ("charAt", [JValue::Int(index)]) => {
            let unit = usize::try_from(*index)
                .ok()
                .and_then(|i| units.get(i))
                .ok_or_else(|| char_at_error(&units, *index))?;
            Ok(Some(JValue::Int(i32::from(*unit))))
        }
        // The JDK returns THIS string when the range is the whole of it —
        // `s.substring(0) == s` is true, and so is `s.subSequence(0, s.length())
        // == s`. Allocating a copy made every such identity false.
        ("substring", [JValue::Int(begin)]) if *begin == 0 => Ok(Some(JValue::Ref(Some(receiver)))),
        ("substring" | "subSequence", [JValue::Int(0), JValue::Int(end)]) if *end == len => {
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("substring", [JValue::Int(begin)]) => substring(heap, &units, *begin, len),
        // subSequence is substring by another name (CharSequence view).
        ("substring" | "subSequence", [JValue::Int(begin), JValue::Int(end)]) => {
            substring_range(heap, &units, *begin, *end)
        }
        ("indexOf", [needle @ JValue::Ref(_)]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(index_of(&units, &needle))))
        }
        ("contains", [needle]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(i32::from(index_of(&units, &needle) >= 0))))
        }
        ("startsWith", [prefix]) => {
            let prefix = arg_units(prefix)?;
            Ok(Some(JValue::Int(i32::from(units.starts_with(&prefix)))))
        }
        ("endsWith", [suffix]) => {
            let suffix = arg_units(suffix)?;
            Ok(Some(JValue::Int(i32::from(units.ends_with(&suffix)))))
        }
        ("equals", [other]) => {
            // equals(Object): a null or non-string argument is false in
            // Java; ours can only receive strings or null.
            let result = match other {
                JValue::Ref(Some(reference)) => match heap.get(*reference) {
                    Some(HeapObject::JavaString(other_units)) => *other_units == units,
                    _ => false,
                },
                _ => false,
            };
            Ok(Some(JValue::Int(i32::from(result))))
        }
        ("equalsIgnoreCase", [other]) => {
            // Like `equals`, a null argument answers false (the Javadoc says
            // so explicitly) — it must not NPE.
            if matches!(other, JValue::Ref(None)) {
                return Ok(Some(JValue::Int(0)));
            }
            // Java compares CODE UNIT by code unit with the SIMPLE case
            // mappings: equal, or equal uppercased, or equal lowercased
            // after that. Lowercasing the whole string with Rust's FULL
            // mapping is a different question — it expands `\u0130` to two
            // characters, so "\u0130".equalsIgnoreCase("i") answered false
            // where a JDK says true.
            let other = arg_units(other)?;
            let same = units.len() == other.len()
                && units.iter().zip(&other).all(|(a, b)| {
                    if a == b {
                        return true;
                    }
                    // `equalsIgnoreCase` compares each unit's uppercase and
                    // THEN its lowercase, which is what makes `\u0130` equal
                    // `i`. Units, not `char`s: a lone surrogate has to compare
                    // too, and `char::from_u32` rejects one.
                    let (upper_a, upper_b) = (java_upper_unit(*a), java_upper_unit(*b));
                    upper_a == upper_b || java_lower_unit(upper_a) == java_lower_unit(upper_b)
                });
            Ok(Some(JValue::Int(i32::from(same))))
        }
        ("compareTo", [other]) => {
            let other = arg_units(other)?;
            Ok(Some(JValue::Int(compare_utf16(&units, &other))))
        }
        ("split", [pattern]) => {
            let pattern = arg_units(pattern)?;
            let parts = split_regex(&units, &pattern, 0)?;
            let refs: Vec<JValue> = parts
                .into_iter()
                .map(|part| JValue::Ref(Some(heap.alloc(HeapObject::JavaString(part)))))
                .collect();
            let reference = heap.alloc(HeapObject::RefArray(
                String::from("[Ljava/lang/String;"),
                refs,
            ));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("replace", [JValue::Int(from), JValue::Int(to)]) => {
            let from = u16::try_from(*from).unwrap_or(u16::MAX);
            let to = u16::try_from(*to).unwrap_or(u16::MAX);
            let replaced: Vec<u16> = units
                .iter()
                .map(|unit| if *unit == from { to } else { *unit })
                .collect();
            let reference = heap.alloc(HeapObject::JavaString(replaced));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("replace", [from, to]) => {
            let from = arg_units(from)?;
            let to = arg_units(to)?;
            let replaced = replace_units(&units, &from, &to);
            let reference = heap.alloc(HeapObject::JavaString(replaced));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("matches", [pattern]) => {
            let pattern = arg_units(pattern)?;
            Ok(Some(JValue::Int(i32::from(matches_regex(
                &units, &pattern,
            )?))))
        }
        ("replaceAll", [pattern, replacement]) => {
            let pattern = arg_units(pattern)?;
            let replacement = arg_units(replacement)?;
            let replaced = replace_regex(&units, &pattern, &replacement, false)?;
            let reference = heap.alloc(HeapObject::JavaString(replaced));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("replaceFirst", [pattern, replacement]) => {
            let pattern = arg_units(pattern)?;
            let replacement = arg_units(replacement)?;
            let replaced = replace_regex(&units, &pattern, &replacement, true)?;
            let reference = heap.alloc(HeapObject::JavaString(replaced));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // The FULL mappings, from the JDK 11 tables rather than from Rust's
        // (a newer Unicode version, which disagreed on thousands of units).
        // By CODE POINT, so a surrogate PAIR is cased as the character it
        // spells — the supplementary scripts that have a case are Deseret,
        // Osage, Warang Citi, Adlam and Medefaidrin — while an unpaired
        // surrogate passes through untouched.
        ("toUpperCase", []) => {
            let reference = heap.alloc(HeapObject::JavaString(unicode::map_case(&units, true)));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("toLowerCase", []) => {
            let reference = heap.alloc(HeapObject::JavaString(unicode::map_case(&units, false)));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // `lines()` — a Stream of the lines, split on \n, \r\n or \r, with
        // no trailing empty line for a trailing terminator (the JDK's rule).
        ("lines", []) => {
            let text = String::from_utf16_lossy(&units);
            let mut line_refs: Vec<JValue> = Vec::new();
            let mut current = String::new();
            let mut chars = text.chars().peekable();
            while let Some(c) = chars.next() {
                match c {
                    '\n' => {
                        line_refs.push(JValue::Ref(Some(heap.alloc_string(&current))));
                        current.clear();
                    }
                    '\r' => {
                        if chars.peek() == Some(&'\n') {
                            chars.next();
                        }
                        line_refs.push(JValue::Ref(Some(heap.alloc_string(&current))));
                        current.clear();
                    }
                    other => current.push(other),
                }
            }
            if !current.is_empty() {
                line_refs.push(JValue::Ref(Some(heap.alloc_string(&current))));
            }
            let stream = heap.alloc(HeapObject::Stream {
                source: crate::value::StreamSource::Fixed(line_refs),
                ops: Vec::new(),
            });
            Ok(Some(JValue::Ref(Some(stream))))
        }
        // `codePoints()` — like `chars()`, but a well-formed surrogate PAIR
        // yields the single code point it spells.
        ("codePoints", []) => {
            let mut source: Vec<JValue> = Vec::new();
            let mut at = 0usize;
            while at < units.len() {
                let unit = units[at];
                let paired = (0xD800..=0xDBFF).contains(&unit)
                    && units
                        .get(at + 1)
                        .is_some_and(|low| (0xDC00..=0xDFFF).contains(low));
                if paired {
                    let low = units[at + 1];
                    let code_point =
                        0x10000 + ((i32::from(unit) - 0xD800) << 10) + (i32::from(low) - 0xDC00);
                    source.push(JValue::Int(code_point));
                    at += 2;
                } else {
                    source.push(JValue::Int(i32::from(unit)));
                    at += 1;
                }
            }
            let stream = heap.alloc(HeapObject::Stream {
                source: crate::value::StreamSource::Fixed(source),
                ops: Vec::new(),
            });
            Ok(Some(JValue::Ref(Some(stream))))
        }
        // `chars()` — an IntStream of the UTF-16 code units, in order.
        ("chars", []) => {
            let source: Vec<JValue> = units.iter().map(|u| JValue::Int(i32::from(*u))).collect();
            let stream = heap.alloc(HeapObject::Stream {
                source: crate::value::StreamSource::Fixed(source),
                ops: Vec::new(),
            });
            Ok(Some(JValue::Ref(Some(stream))))
        }
        ("trim", []) => {
            // Over UNITS: an unpaired surrogate is not whitespace and must
            // survive, which a round trip through a Rust `String` cannot.
            let trimmed = trim_units(&units, true, true, |unit| unit <= u16::from(b' '));
            let reference = heap.alloc_string_units(trimmed);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("strip" | "stripLeading" | "stripTrailing", []) => {
            // JAVA's whitespace (`Character.isWhitespace`), not Unicode's:
            // the two differ on U+001C..1F (Java: yes) and the non-breaking
            // spaces (Java: no). Rust's `char::is_whitespace` silently used
            // the Unicode set, so `strip()` disagreed with caturra's own
            // `Character.isWhitespace`.
            let white = |unit: u16| unicode::is_whitespace(u32::from(unit));
            let stripped = match method {
                "strip" => trim_units(&units, true, true, white),
                "stripLeading" => trim_units(&units, true, false, white),
                _ => trim_units(&units, false, true, white),
            };
            let reference = heap.alloc_string_units(stripped);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("isBlank", []) => {
            let blank = String::from_utf16_lossy(&units)
                .chars()
                .all(java_is_whitespace);
            Ok(Some(JValue::Int(i32::from(blank))))
        }
        ("repeat", [JValue::Int(count)]) => {
            if *count < 0 {
                return Err(throw(format!(
                    "java.lang.IllegalArgumentException: count is negative: {count}"
                )));
            }
            // `repeat(1)` answers THIS string and `repeat(0)` the empty one
            // (the JDK returns `this` and `""` respectively), so both identities
            // hold rather than each allocating a copy.
            if *count == 1 {
                return Ok(Some(JValue::Ref(Some(receiver))));
            }
            if *count == 0 || units.is_empty() {
                // The JDK answers the `""` LITERAL, so `s.repeat(0) == ""` is
                // true — reuse the pooled empty string rather than mint one.
                let empty = heap
                    .find_string(&[])
                    .unwrap_or_else(|| heap.alloc_string(""));
                return Ok(Some(JValue::Ref(Some(empty))));
            }
            let mut repeated =
                Vec::with_capacity(units.len() * usize::try_from(*count).unwrap_or(0));
            for _ in 0..*count {
                repeated.extend_from_slice(&units);
            }
            let reference = heap.alloc(HeapObject::JavaString(repeated));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("concat", [other]) => {
            let other = arg_units(other)?;
            // The Javadoc guarantees `this` is returned when the argument is
            // empty — `a.concat("") == a` is observably true on a JDK (while
            // `a + ""` mints a new string).
            if other.is_empty() {
                return Ok(Some(JValue::Ref(Some(receiver))));
            }
            let mut joined = units.clone();
            joined.extend(other);
            let reference = heap.alloc(HeapObject::JavaString(joined));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("compareToIgnoreCase", [other]) => {
            let other = arg_units(other)?;
            let folded_self: Vec<u16> = units.iter().map(|u| java_fold_unit(*u)).collect();
            let folded_other: Vec<u16> = other.iter().map(|u| java_fold_unit(*u)).collect();
            Ok(Some(JValue::Int(compare_utf16(
                &folded_self,
                &folded_other,
            ))))
        }
        ("contentEquals", [other]) => {
            let other = arg_units(other)?;
            Ok(Some(JValue::Int(i32::from(units == other))))
        }
        // `regionMatches(toffset, other, ooffset, len)` and its
        // case-insensitive form. A negative offset or an over-long region is
        // `false`, never an exception.
        (
            "regionMatches",
            [
                JValue::Int(toffset),
                other,
                JValue::Int(ooffset),
                JValue::Int(count),
            ],
        ) => {
            let other = arg_units(other)?;
            Ok(Some(JValue::Int(i32::from(region_matches(
                &units, *toffset, &other, *ooffset, *count, false,
            )))))
        }
        (
            "regionMatches",
            [
                JValue::Int(ignore_case),
                JValue::Int(toffset),
                other,
                JValue::Int(ooffset),
                JValue::Int(count),
            ],
        ) => {
            let other = arg_units(other)?;
            Ok(Some(JValue::Int(i32::from(region_matches(
                &units,
                *toffset,
                &other,
                *ooffset,
                *count,
                *ignore_case != 0,
            )))))
        }
        ("hashCode", []) => Ok(Some(JValue::Int(java_string_hash_units(&units)))),
        ("indexOf", [JValue::Int(ch)]) => Ok(Some(JValue::Int(index_of_char(&units, *ch, 0)))),
        ("indexOf", [JValue::Int(ch), JValue::Int(from)]) => {
            Ok(Some(JValue::Int(index_of_char(&units, *ch, *from))))
        }
        ("indexOf", [needle, JValue::Int(from)]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(index_of_from(&units, &needle, *from))))
        }
        ("lastIndexOf", [JValue::Int(ch)]) => {
            Ok(Some(JValue::Int(last_index_of_char(&units, *ch, i32::MAX))))
        }
        ("lastIndexOf", [JValue::Int(ch), JValue::Int(from)]) => {
            Ok(Some(JValue::Int(last_index_of_char(&units, *ch, *from))))
        }
        ("lastIndexOf", [needle]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(last_index_of_from(
                &units,
                &needle,
                i32::MAX,
            ))))
        }
        ("lastIndexOf", [needle, JValue::Int(from)]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(last_index_of_from(
                &units, &needle, *from,
            ))))
        }
        ("split", [pattern, JValue::Int(limit)]) => {
            let pattern = arg_units(pattern)?;
            let parts = split_regex(&units, &pattern, *limit)?;
            let refs: Vec<JValue> = parts
                .into_iter()
                .map(|part| JValue::Ref(Some(heap.alloc(HeapObject::JavaString(part)))))
                .collect();
            let reference = heap.alloc(HeapObject::RefArray(
                String::from("[Ljava/lang/String;"),
                refs,
            ));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("startsWith", [prefix, JValue::Int(offset)]) => {
            let prefix = arg_units(prefix)?;
            let starts = usize::try_from(*offset)
                .ok()
                .and_then(|at| units.get(at..))
                .is_some_and(|rest| rest.starts_with(&prefix));
            Ok(Some(JValue::Int(i32::from(starts))))
        }
        ("toCharArray", []) => {
            let values: Vec<i32> = units.iter().map(|u| i32::from(*u)).collect();
            let reference = heap.alloc(HeapObject::IntArray(IntKind::Char, values));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // `getBytes()` encodes in the default charset, which is UTF-8 here.
        // An unpaired surrogate is unencodable and becomes `?`, the same
        // substitution the console makes.
        // `getBytes(charset)` / `getBytes("UTF-8")` — the same encoder, told
        // which charset. An unknown NAME is the JDK's checked
        // `UnsupportedEncodingException`; an unknown Charset cannot happen,
        // since `forName` refuses to build one.
        ("getBytes", [JValue::Ref(Some(which))]) => {
            let name = match heap.get(*which) {
                Some(HeapObject::Charset(name)) => name.clone(),
                Some(HeapObject::JavaString(units)) => {
                    let written = String::from_utf16_lossy(units);
                    match canonical_charset(&written) {
                        Some(name) => name.to_owned(),
                        None => {
                            return Err(throw(format!(
                                "java.io.UnsupportedEncodingException: {written}"
                            )));
                        }
                    }
                }
                _ => return Err(throw("java.lang.NullPointerException")),
            };
            let bytes = encode_charset(&units, &name);
            let reference = heap.alloc(HeapObject::ByteArray(bytes));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("getBytes", []) => {
            let mut bytes: Vec<i8> = Vec::new();
            for decoded in char::decode_utf16(units.iter().copied()) {
                match decoded {
                    Ok(character) => {
                        let mut buffer = [0u8; 4];
                        for byte in character.encode_utf8(&mut buffer).as_bytes() {
                            bytes.push(byte.cast_signed());
                        }
                    }
                    Err(_) => bytes.push(b'?'.cast_signed()),
                }
            }
            let reference = heap.alloc(HeapObject::ByteArray(bytes));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        (
            "getChars",
            [
                JValue::Int(begin),
                JValue::Int(end),
                JValue::Ref(target),
                JValue::Int(at),
            ],
        ) => match target {
            // A STRING's `getChars` words the destination failure its own way
            // ("offset 8, count 6, length 10") and throws the String flavour,
            // where the builder's throws the plain IndexOutOfBoundsException.
            Some(target) => {
                let copied = end.saturating_sub(*begin);
                let destination = match heap.get(*target) {
                    Some(HeapObject::IntArray(_, values)) => {
                        i32::try_from(values.len()).unwrap_or(i32::MAX)
                    }
                    _ => return Err(throw("java.lang.NullPointerException")),
                };
                if *begin >= 0
                    && end >= begin
                    && *end <= i32::try_from(units.len()).unwrap_or(i32::MAX)
                    && (*at < 0 || at.saturating_add(copied) > destination)
                {
                    return Err(throw(format!(
                        "java.lang.StringIndexOutOfBoundsException: offset {at}, \
                         count {copied}, length {destination}"
                    )));
                }
                get_chars(heap, &units, *begin, *end, *target, *at)
            }
            // A NULL destination is an ordinary NullPointerException; the arm
            // used to require a reference, so the call fell through to
            // "unknown native member" and aborted the whole run.
            None => Err(throw("java.lang.NullPointerException")),
        },
        ("toString", []) => Ok(Some(JValue::Ref(Some(receiver)))),
        ("intern", []) => {
            let canonical = heap.find_string(&units).unwrap_or(receiver);
            Ok(Some(JValue::Ref(Some(canonical))))
        }
        ("codePointAt", [JValue::Int(index)]) => {
            code_point_at(&units, *index).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointBefore", [JValue::Int(index)]) => {
            code_point_before(&units, *index).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointCount", [JValue::Int(begin), JValue::Int(end)]) => {
            code_point_count(&units, *begin, *end).map(|n| Some(JValue::Int(n)))
        }
        ("offsetByCodePoints", [JValue::Int(index), JValue::Int(offset)]) => {
            offset_by_code_points(&units, *index, *offset).map(|at| Some(JValue::Int(at)))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("String.{method}"))),
    }
}

/// `getChars(begin, end, dest, at)` — copy UTF-16 units into a `char[]`.
/// Shared by `String` and `StringBuilder`.
fn get_chars(
    heap: &mut Heap,
    units: &[u16],
    begin: i32,
    end: i32,
    target: HeapRef,
    at: i32,
) -> Result<Option<JValue>, VmError> {
    // The JDK words the SOURCE range failure as `start … end … length` (it goes
    // through `AbstractStringBuilder.checkRangeSIOOBE`), and the DESTINATION
    // one as a plain IndexOutOfBoundsException describing the destination's
    // range — not an array-index message about one element.
    let source = usize::try_from(begin)
        .ok()
        .zip(usize::try_from(end).ok())
        .filter(|(b, e)| b <= e && *e <= units.len())
        .map(|(b, e)| units[b..e].to_vec())
        .ok_or_else(|| {
            throw(format!(
                "java.lang.StringIndexOutOfBoundsException: start {begin}, end {end}, \
                 length {}",
                units.len()
            ))
        })?;
    let copied = i32::try_from(source.len()).unwrap_or(i32::MAX);
    let destination_len = match heap.get(target) {
        Some(HeapObject::IntArray(_, values)) => i32::try_from(values.len()).unwrap_or(i32::MAX),
        // `getChars(…, null, 0)` is a plain NullPointerException, not an
        // internal "not implemented".
        _ => return Err(throw("java.lang.NullPointerException")),
    };
    if at < 0 || at.saturating_add(copied) > destination_len {
        return Err(throw(format!(
            "java.lang.IndexOutOfBoundsException: start {at}, end {}, length {destination_len}",
            at.saturating_add(copied)
        )));
    }
    let at = usize::try_from(at).unwrap_or(0);
    let Some(HeapObject::IntArray(_, values)) = heap.get_mut(target) else {
        return Err(throw("java.lang.NullPointerException"));
    };
    if at + source.len() > values.len() {
        let bad = at + source.len() - 1;
        let len = values.len();
        return Err(throw(format!(
            "java.lang.ArrayIndexOutOfBoundsException: Index {bad} out of bounds \
             for length {len}"
        )));
    }
    for (index, unit) in source.iter().enumerate() {
        values[at + index] = i32::from(*unit);
    }
    Ok(None)
}

/// `codePointBefore(index)`: the code point ending just before `index`.
fn code_point_before(units: &[u16], index: i32) -> Result<i32, VmError> {
    let before = index - 1;
    if before < 0 {
        // `codePointBefore` reports the OTHER of the JDK's two String
        // wordings — `codePointAt` names the length, this one does not.
        return Err(throw(format!(
            "java.lang.StringIndexOutOfBoundsException: String index out of range: {index}"
        )));
    }
    // A low surrogate preceded by a high one forms a pair.
    let at = usize::try_from(before).unwrap_or(usize::MAX);
    if at > 0
        && units.get(at).is_some_and(|u| is_low_surrogate(*u))
        && units.get(at - 1).is_some_and(|u| is_high_surrogate(*u))
    {
        return code_point_at(units, before - 1);
    }
    // Otherwise the UNIT, whatever it is. Reading it as a code point instead
    // looked FORWARD and combined a pair that starts here — so
    // `"a\ud83d\ude00b".codePointBefore(2)` answered the whole emoji where a
    // JDK answers the high surrogate alone.
    let Some(unit) = units.get(at) else {
        return code_point_at(units, before);
    };
    Ok(i32::from(*unit))
}

/// `codePointCount(begin, end)`: code points in the half-open range.
/// The units behind a `CharSequence` that `Character` was handed. Unlike the
/// concatenating one, a null here is a `NullPointerException` rather than the
/// four characters of "null".
fn code_point_source(heap: &Heap, value: &JValue) -> Result<Vec<u16>, VmError> {
    match value {
        JValue::Ref(Some(reference)) => match heap.get(*reference) {
            Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                Ok(units.clone())
            }
            _ => Err(throw("java.lang.ClassCastException: not a CharSequence")),
        },
        _ => Err(throw("java.lang.NullPointerException")),
    }
}

fn code_point_count(units: &[u16], begin: i32, end: i32) -> Result<i32, VmError> {
    let (begin, end) = usize::try_from(begin)
        .ok()
        .zip(usize::try_from(end).ok())
        .filter(|(b, e)| b <= e && *e <= units.len())
        // The JDK reaches this through a bounds check that throws a BARE
        // `IndexOutOfBoundsException`, so `getMessage()` is null — printed as
        // "null", not as a description of the range.
        .ok_or_else(|| throw(String::from("java.lang.IndexOutOfBoundsException")))?;
    let count = char::decode_utf16(units[begin..end].iter().copied()).count();
    Ok(i32::try_from(count).unwrap_or(i32::MAX))
}

/// `offsetByCodePoints(index, offset)`: the unit index `offset` code
/// points away from `index`.
fn offset_by_code_points(units: &[u16], index: i32, offset: i32) -> Result<i32, VmError> {
    let mut at = usize::try_from(index)
        .ok()
        .filter(|at| *at <= units.len())
        .ok_or_else(|| throw(String::from("java.lang.IndexOutOfBoundsException")))?;
    let out_of_range = || throw(String::from("java.lang.IndexOutOfBoundsException"));
    let mut remaining = offset;
    while remaining > 0 {
        if at >= units.len() {
            return Err(out_of_range());
        }
        at += if units.get(at).is_some_and(|u| is_high_surrogate(*u))
            && units.get(at + 1).is_some_and(|u| is_low_surrogate(*u))
        {
            2
        } else {
            1
        };
        remaining -= 1;
    }
    while remaining < 0 {
        if at == 0 {
            return Err(out_of_range());
        }
        at -= if at >= 2
            && units.get(at - 1).is_some_and(|u| is_low_surrogate(*u))
            && units.get(at - 2).is_some_and(|u| is_high_surrogate(*u))
        {
            2
        } else {
            1
        };
        remaining += 1;
    }
    Ok(i32::try_from(at).unwrap_or(i32::MAX))
}

fn is_high_surrogate(unit: u16) -> bool {
    (0xD800..0xDC00).contains(&unit)
}

fn is_low_surrogate(unit: u16) -> bool {
    (0xDC00..0xE000).contains(&unit)
}

/// `java.lang.StringBuilder` instance methods. The builder stores UTF-16
/// code units, exactly as `String` does, so indices mean what Java says
/// they mean even across supplementary characters.
///
/// `capacity` is modelled too, in a side table on the heap: it is a pure
/// function of how the builder was BUILT, not of what it now holds, so it
/// cannot be derived from the contents.
#[allow(clippy::too_many_lines)] // one method table
fn builder_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let Some(HeapObject::StringBuilder(units)) = heap.get(receiver) else {
        unreachable!("receiver kind checked by caller");
    };
    let units = units.clone();
    let count = units.len();
    let len = i32::try_from(count).unwrap_or(i32::MAX);
    let params = descriptor_params(descriptor);

    // Resolve a String argument's code units (null → NPE like Java).
    let arg_units = |value: &JValue| -> Result<Vec<u16>, VmError> {
        match value {
            JValue::Ref(Some(reference)) => match heap.get(*reference) {
                Some(HeapObject::JavaString(other)) => Ok(other.clone()),
                _ => Err(throw("java.lang.ClassCastException: not a String")),
            },
            JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
            _ => Err(throw("java.lang.VerifyError: expected a String argument")),
        }
    };

    match (method, args) {
        ("length", []) => Ok(Some(JValue::Int(len))),
        ("capacity", []) => Ok(Some(JValue::Int(
            i32::try_from(heap.builder_capacity(receiver)).unwrap_or(i32::MAX),
        ))),
        ("ensureCapacity", [JValue::Int(wanted)]) => {
            if let Ok(wanted) = usize::try_from(*wanted) {
                heap.grow_builder_capacity(receiver, wanted);
            }
            Ok(None)
        }
        // `trimToSize` drops the slack — the one operation that SHRINKS it.
        ("trimToSize", []) => {
            if count < heap.builder_capacity(receiver) {
                heap.set_builder_capacity(receiver, count);
            }
            Ok(None)
        }
        ("charAt", [JValue::Int(index)]) => {
            let at = check_index(*index, count)?;
            Ok(Some(JValue::Int(i32::from(units[at]))))
        }
        ("toString", []) => {
            let text = heap.alloc(HeapObject::JavaString(units));
            Ok(Some(JValue::Ref(Some(text))))
        }
        // `equals` is Object identity: StringBuilder does not override it, so
        // two builders are equal only when they are the SAME object.
        ("equals", [other]) => {
            let same = matches!(other, JValue::Ref(Some(r)) if *r == receiver);
            Ok(Some(JValue::Int(i32::from(same))))
        }
        // `append(char[], offset, LEN)` and `append(CharSequence, start,
        // END)` — sub-range appends, told apart by descriptor. Both check
        // with the JDK's `checkRange`: a plain IndexOutOfBoundsException
        // saying "start S, end E, length L", where the char[] form's end is
        // offset + len (int addition, so it can wrap like Java's).
        ("append", [source, JValue::Int(a), JValue::Int(b)]) => {
            let (chars, start, end) = if params.starts_with("[C") {
                (char_array_units(heap, source)?, *a, a.wrapping_add(*b))
            } else {
                (char_sequence_units(heap, source)?, *a, *b)
            };
            let (start, end) = check_subrange(start, end, chars.len(), false)?;
            let mut appended = units;
            appended.extend_from_slice(&chars[start..end]);
            builder_store(heap, receiver, appended);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("append", [value]) => {
            let mut appended = units;
            appended.extend(builder_value_text(heap, params, value)?);
            builder_store(heap, receiver, appended);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        // `insert(dst, CharSequence)` — JDK semantics: the destination offset
        // is checked first, then the sequence (null inserts "null") copies IN
        // PLACE after the shift, so inserting a builder into itself reads the
        // already-shifted chars (`new StringBuilder("abab").insert(1, self)`
        // is "aaaaabab", not the snapshot "aababbab").
        ("insert", [JValue::Int(offset), value]) if params == "ILjava/lang/CharSequence;" => {
            let at = check_offset(*offset, count)?;
            let source = char_sequence_units(heap, value)?;
            let end = source.len();
            let inserted = insert_aliasing(units, at, value, receiver, &source, 0, end);
            builder_store(heap, receiver, inserted);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("insert", [JValue::Int(offset), value]) => {
            // The offset is the first parameter; the value's own descriptor
            // is whatever follows it.
            let value_units = builder_value_text(heap, &params["I".len()..], value)?;
            let at = check_offset(*offset, count)?;
            let mut inserted = units;
            inserted.splice(at..at, value_units);
            builder_store(heap, receiver, inserted);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        // `insert(dst, char[], offset, LEN)` and `insert(dst, CharSequence,
        // start, END)`. The destination check comes first (SIOOBE "offset
        // D,length C"); the sub-range check is a StringIndexOutOfBounds for
        // the char[] form and a plain IndexOutOfBounds for the CharSequence
        // form, both saying "start S, end E, length L" — the JDK's split.
        ("insert", [JValue::Int(offset), source, JValue::Int(a), JValue::Int(b)]) => {
            let at = check_offset(*offset, count)?;
            let is_chars = params.starts_with("I[C");
            let (chars, start, end) = if is_chars {
                (char_array_units(heap, source)?, *a, a.wrapping_add(*b))
            } else {
                (char_sequence_units(heap, source)?, *a, *b)
            };
            let (start, end) = check_subrange(start, end, chars.len(), is_chars)?;
            let inserted = insert_aliasing(units, at, source, receiver, &chars, start, end);
            builder_store(heap, receiver, inserted);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("appendCodePoint", [JValue::Int(code_point)]) => {
            let mut appended = units;
            appended.extend(code_point_units(*code_point)?);
            builder_store(heap, receiver, appended);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("delete", [JValue::Int(start), JValue::Int(end)]) => {
            let (start, end) = check_range(*start, *end, count)?;
            let mut deleted = units;
            deleted.drain(start..end);
            builder_store(heap, receiver, deleted);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("deleteCharAt", [JValue::Int(index)]) => {
            let at = check_index(*index, count)?;
            let mut deleted = units;
            deleted.remove(at);
            builder_store(heap, receiver, deleted);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("replace", [JValue::Int(start), JValue::Int(end), value]) => {
            let value_units = arg_units(value)?;
            let (start, end) = check_range(*start, *end, count)?;
            let mut replaced = units;
            replaced.splice(start..end, value_units);
            builder_store(heap, receiver, replaced);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("reverse", []) => {
            builder_store(heap, receiver, reverse_units(&units));
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("setCharAt", [JValue::Int(index), JValue::Int(ch)]) => {
            let at = check_index(*index, count)?;
            let mut updated = units;
            updated[at] = u16::try_from(*ch).unwrap_or(u16::MAX);
            builder_store(heap, receiver, updated);
            Ok(None)
        }
        ("setLength", [JValue::Int(new_length)]) => {
            let new_length = usize::try_from(*new_length).map_err(|_| {
                throw(format!(
                    "java.lang.StringIndexOutOfBoundsException: String index out of range: {new_length}"
                ))
            })?;
            let mut resized = units;
            // Growing pads with the null character, as Java does.
            resized.resize(new_length, 0);
            builder_store(heap, receiver, resized);
            Ok(None)
        }
        ("indexOf", [needle]) => Ok(Some(JValue::Int(index_of(&units, &arg_units(needle)?)))),
        ("indexOf", [needle, JValue::Int(from)]) => Ok(Some(JValue::Int(index_of_from(
            &units,
            &arg_units(needle)?,
            *from,
        )))),
        ("lastIndexOf", [needle]) => Ok(Some(JValue::Int(last_index_of_from(
            &units,
            &arg_units(needle)?,
            i32::MAX,
        )))),
        ("lastIndexOf", [needle, JValue::Int(from)]) => Ok(Some(JValue::Int(last_index_of_from(
            &units,
            &arg_units(needle)?,
            *from,
        )))),
        // A BUILDER's one-argument substring is `substring(start, count)` in
        // the JDK, so its failure carries the same `start, end, length` wording
        // as the two-argument form — not String's "String index out of range".
        ("substring", [JValue::Int(begin)]) => builder_substring(heap, &units, *begin, len),
        // subSequence is substring by another name (CharSequence view). Unlike
        // String's substring, StringBuilder's out-of-range message says
        // "start"/"end" (not "begin"), and does NOT clamp end.
        ("substring" | "subSequence", [JValue::Int(begin), JValue::Int(end)]) => {
            builder_substring(heap, &units, *begin, *end)
        }
        ("compareTo", [other]) => {
            let other = match other {
                JValue::Ref(Some(reference)) => match heap.get(*reference) {
                    Some(HeapObject::StringBuilder(other)) => other.clone(),
                    _ => return Err(throw("java.lang.ClassCastException: not a StringBuilder")),
                },
                _ => return Err(throw("java.lang.NullPointerException")),
            };
            Ok(Some(JValue::Int(compare_utf16(&units, &other))))
        }
        (
            "getChars",
            [
                JValue::Int(begin),
                JValue::Int(end),
                JValue::Ref(target),
                JValue::Int(at),
            ],
        ) => match target {
            Some(target) => get_chars(heap, &units, *begin, *end, *target, *at),
            // A NULL destination is an ordinary NullPointerException; the arm
            // used to require a reference, so the call fell through to
            // "unknown native member" and aborted the whole run.
            None => Err(throw("java.lang.NullPointerException")),
        },
        ("codePointAt", [JValue::Int(index)]) => {
            code_point_at(&units, *index).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointBefore", [JValue::Int(index)]) => {
            code_point_before(&units, *index).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointCount", [JValue::Int(begin), JValue::Int(end)]) => {
            code_point_count(&units, *begin, *end).map(|n| Some(JValue::Int(n)))
        }
        ("offsetByCodePoints", [JValue::Int(index), JValue::Int(offset)]) => {
            offset_by_code_points(&units, *index, *offset).map(|at| Some(JValue::Int(at)))
        }
        // A builder does NOT override hashCode either: it is Object's, and so
        // stable for the object's lifetime whatever the text becomes.
        ("hashCode", []) => Ok(Some(JValue::Int(identity_hash(receiver)))),
        // `chars()`/`codePoints()` are `CharSequence`'s, so a builder answers
        // them exactly as a String does — over ITS units, at the moment of the
        // call.
        ("chars", []) => {
            let source: Vec<JValue> = units.iter().map(|u| JValue::Int(i32::from(*u))).collect();
            let stream = heap.alloc(HeapObject::Stream {
                source: crate::value::StreamSource::Fixed(source),
                ops: Vec::new(),
            });
            Ok(Some(JValue::Ref(Some(stream))))
        }
        ("codePoints", []) => {
            let text = String::from_utf16_lossy(&units);
            let source: Vec<JValue> = text
                .chars()
                .map(|c| JValue::Int(i32::try_from(u32::from(c)).unwrap_or(i32::MAX)))
                .collect();
            let stream = heap.alloc(HeapObject::Stream {
                source: crate::value::StreamSource::Fixed(source),
                ops: Vec::new(),
            });
            Ok(Some(JValue::Ref(Some(stream))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("StringBuilder.{method}"))),
    }
}

/// Overwrite a builder's contents.
fn builder_store(heap: &mut Heap, receiver: HeapRef, units: Vec<u16>) {
    // Capacity only ever grows, and only when the contents outrun it — the
    // one place every mutation passes through.
    heap.grow_builder_capacity(receiver, units.len());
    let Some(HeapObject::StringBuilder(slot)) = heap.get_mut(receiver) else {
        unreachable!("receiver kind checked by caller");
    };
    *slot = units;
}

/// An index into existing contents (`0 <= index < count`).
fn check_index(index: i32, count: usize) -> Result<usize, VmError> {
    usize::try_from(index)
        .ok()
        .filter(|at| *at < count)
        .ok_or_else(|| {
            throw(format!(
                "java.lang.StringIndexOutOfBoundsException: index {index},length {count}"
            ))
        })
}

/// The JDK's `checkRange` for the sub-range append/insert overloads: no
/// clamping, "start S, end E, length L". The char[] INSERT form throws
/// `StringIndexOutOfBounds` where every other form throws the plain
/// `IndexOutOfBounds` — the JDK's own split.
fn check_subrange(
    start: i32,
    end: i32,
    length: usize,
    string_flavored: bool,
) -> Result<(usize, usize), VmError> {
    let fits = start >= 0 && start <= end && i32::try_from(length).is_ok_and(|len| end <= len);
    if !fits {
        let class = if string_flavored {
            "java.lang.StringIndexOutOfBoundsException"
        } else {
            "java.lang.IndexOutOfBoundsException"
        };
        return Err(throw(format!(
            "{class}: start {start}, end {end}, length {length}"
        )));
    }
    Ok((
        usize::try_from(start).unwrap_or_default(),
        usize::try_from(end).unwrap_or_default(),
    ))
}

/// The UTF-16 units of a `CharSequence` argument — a `String`, a
/// `StringBuilder`, or null (which reads as the four characters of "null",
/// as the JDK's sub-range overloads do).
fn char_sequence_units(heap: &Heap, value: &JValue) -> Result<Vec<u16>, VmError> {
    match value {
        JValue::Ref(None) => Ok("null".encode_utf16().collect()),
        JValue::Ref(Some(reference)) => match heap.get(*reference) {
            Some(HeapObject::JavaString(source) | HeapObject::StringBuilder(source)) => {
                Ok(source.clone())
            }
            _ => Err(VmError::UnknownIntrinsic(String::from(
                "argument is not a CharSequence",
            ))),
        },
        _ => Err(VmError::UnknownIntrinsic(String::from(
            "argument is not a CharSequence",
        ))),
    }
}

/// The JDK's `CharSequence` insert: shift the tail, then copy `charAt` by
/// `charAt` from the LIVE source — which, when the source IS the receiver,
/// reads the already-shifted buffer (`new StringBuilder("abab").insert(1,
/// self)` is "aaaaabab", not the snapshot's "aababbab"). Any other source
/// is unaffected by the shift, so a plain copy is identical.
fn insert_aliasing(
    units: Vec<u16>,
    at: usize,
    source_value: &JValue,
    receiver: HeapRef,
    source_units: &[u16],
    start: usize,
    end: usize,
) -> Vec<u16> {
    let self_insert = matches!(source_value, JValue::Ref(Some(r)) if *r == receiver);
    let mut buf = units;
    buf.splice(at..at, std::iter::repeat_n(0u16, end - start));
    if self_insert {
        for (dst, i) in (at..).zip(start..end) {
            buf[dst] = buf[i];
        }
    } else {
        buf[at..at + (end - start)].copy_from_slice(&source_units[start..end]);
    }
    buf
}

/// A position between characters (`0 <= offset <= count`), as `insert` takes.
fn check_offset(offset: i32, count: usize) -> Result<usize, VmError> {
    usize::try_from(offset)
        .ok()
        .filter(|at| *at <= count)
        .ok_or_else(|| {
            throw(format!(
                "java.lang.StringIndexOutOfBoundsException: offset {offset},length {count}"
            ))
        })
}

/// A `delete`/`replace` range: `start` must be a valid offset no greater
/// than `end`, and `end` is clamped to the length (Java tolerates a large
/// `end` here, unlike `substring`).
fn check_range(start: i32, end: i32, count: usize) -> Result<(usize, usize), VmError> {
    // JDK 11 clamps `end` to the length FIRST, so the message reports the
    // clamped value: `replace(9, 10, ...)` on a length-3 builder is
    // "start 9, end 3, length 3", not "end 10".
    let clamped_end = end.min(i32::try_from(count).unwrap_or(i32::MAX));
    let bad = || {
        throw(format!(
            "java.lang.StringIndexOutOfBoundsException: start {start}, \
             end {clamped_end}, length {count}"
        ))
    };
    if start > clamped_end {
        return Err(bad());
    }
    let start = usize::try_from(start).ok().filter(|s| *s <= count);
    let start = start.ok_or_else(bad)?;
    let end = usize::try_from(clamped_end).unwrap_or(0);
    Ok((start, end))
}

/// The UTF-16 units of a code point, for `appendCodePoint`.
fn code_point_units(code_point: i32) -> Result<Vec<u16>, VmError> {
    // The message shows the value as UPPERCASE hex of its unsigned bits —
    // `-1` is `0xFFFFFFFF`, `0x110000` is itself (JDK 11's
    // `Character.toString(int)` / `appendCodePoint`).
    let invalid = || {
        throw(format!(
            "java.lang.IllegalArgumentException: Not a valid Unicode code point: 0x{:X}",
            code_point.cast_unsigned()
        ))
    };
    let valid = u32::try_from(code_point)
        .ok()
        .filter(|cp| *cp <= 0x0010_FFFF)
        .ok_or_else(invalid)?;
    // Surrogate code points have no scalar value but are still one `char`.
    if (0xD800..0xE000).contains(&valid) {
        return Ok(vec![u16::try_from(valid).unwrap_or(u16::MAX)]);
    }
    let Some(ch) = char::from_u32(valid) else {
        return Err(invalid());
    };
    let mut buffer = [0u16; 2];
    Ok(ch.encode_utf16(&mut buffer).to_vec())
}

/// `StringBuilder.reverse()`: reverse the code units, but keep the two
/// halves of a valid surrogate pair in order — Java reverses by code point.
fn reverse_units(units: &[u16]) -> Vec<u16> {
    let mut reversed = Vec::with_capacity(units.len());
    let mut at = units.len();
    while at > 0 {
        if at >= 2 && is_low_surrogate(units[at - 1]) && is_high_surrogate(units[at - 2]) {
            reversed.extend_from_slice(&units[at - 2..at]);
            at -= 2;
        } else {
            reversed.push(units[at - 1]);
            at -= 1;
        }
    }
    reversed
}

/// `String.codePointAt`: the code point at a UTF-16 index (pairs
/// combine; unpaired surrogates return themselves, like Java).
/// `charAt`'s out-of-range message, which COMPACT STRINGS make observable.
///
/// A JDK 9+ string is stored as Latin-1 when every character fits in a byte
/// and as UTF-16 otherwise, and the two implementations word this differently:
/// `StringLatin1` says "String index out of range: i", `StringUTF16` says
/// "index i,length n". So the same call on `"abc"` and on `"ab\u0100"`
/// reports differently, and only the second names the length.
fn char_at_error(units: &[u16], index: i32) -> VmError {
    if units.iter().any(|unit| *unit > 0xFF) {
        return throw(format!(
            "java.lang.StringIndexOutOfBoundsException: index {index},length {}",
            units.len()
        ));
    }
    throw(format!(
        "java.lang.StringIndexOutOfBoundsException: String index out of range: {index}"
    ))
}

fn code_point_at(units: &[u16], index: i32) -> Result<i32, VmError> {
    let at = usize::try_from(index)
        .ok()
        .filter(|i| *i < units.len())
        .ok_or_else(|| {
            throw(format!(
                "java.lang.StringIndexOutOfBoundsException: index {index},length {}",
                units.len()
            ))
        })?;
    let unit = units[at];
    if (0xD800..0xDC00).contains(&unit)
        && let Some(low) = units.get(at + 1)
        && (0xDC00..0xE000).contains(low)
    {
        let combined = 0x10000 + ((u32::from(unit) - 0xD800) << 10) + (u32::from(*low) - 0xDC00);
        return Ok(i32::try_from(combined).unwrap_or(i32::MAX));
    }
    Ok(i32::from(unit))
}

/// `indexOf(int ch, int from)` — the char as its UTF-16 encoding.
/// `String.regionMatches` — do the two regions hold the same characters? A
/// negative offset or a region running past either end answers `false` rather
/// than throwing, as the JDK's does. The case-insensitive form compares each
/// unit by its uppercase AND then its lowercase form, exactly as
/// `String.regionMatches(true, …)` documents.
/// One UTF-16 unit uppercased, then lowercased, the way
/// `String.compareToIgnoreCase` and `regionMatches(true, …)` fold — per UNIT,
/// with a mapping that grows to more than one character left alone (the JDK
/// folds `Character.toUpperCase`, which is the SIMPLE mapping).
/// `String.CASE_INSENSITIVE_ORDER.compare` — the same per-unit fold
/// `compareToIgnoreCase` uses.
pub(crate) fn compare_ignore_case(left: &str, right: &str) -> i32 {
    let fold = |text: &str| -> Vec<u16> { text.encode_utf16().map(java_fold_unit).collect() };
    compare_utf16(&fold(left), &fold(right))
}

fn java_fold_unit(unit: u16) -> u16 {
    java_lower_unit(java_upper_unit(unit))
}

/// `Character.toUpperCase` on one unit: the SIMPLE mapping, so a character
/// whose uppercase form is several characters (`ß` → `SS`) is unchanged. Using
/// Rust's full mapping made `compareToIgnoreCase` call equal strings unequal.
fn java_upper_unit(unit: u16) -> u16 {
    unicode::simple_upper(unit)
}

/// `Character.toLowerCase` on one unit. Unlike the uppercase side, a
/// multi-character full mapping does NOT mean "no simple mapping": `\u0130`
/// lowercases to two characters in full (`i` and a combining dot) but its
/// SIMPLE mapping is a plain `i`, which is what the JDK answers. Leaving it
/// unchanged made `"\u0130".compareToIgnoreCase("i")` non-zero.
fn java_lower_unit(unit: u16) -> u16 {
    unicode::simple_lower(unit)
}

fn region_matches(
    units: &[u16],
    toffset: i32,
    other: &[u16],
    ooffset: i32,
    count: i32,
    ignore_case: bool,
) -> bool {
    let (Ok(here), Ok(there), Ok(count)) = (
        usize::try_from(toffset),
        usize::try_from(ooffset),
        usize::try_from(count),
    ) else {
        return false;
    };
    if here + count > units.len() || there + count > other.len() {
        return false;
    }
    (0..count).all(|i| {
        let (mine, theirs) = (units[here + i], other[there + i]);
        if mine == theirs {
            return true;
        }
        ignore_case
            && (java_upper_unit(mine) == java_upper_unit(theirs)
                || java_lower_unit(mine) == java_lower_unit(theirs))
    })
}

fn index_of_char(haystack: &[u16], ch: i32, from: i32) -> i32 {
    let Some(encoded) = encode_char(ch) else {
        return -1;
    };
    index_of_from(haystack, &encoded, from)
}

fn last_index_of_char(haystack: &[u16], ch: i32, from: i32) -> i32 {
    let Some(encoded) = encode_char(ch) else {
        return -1;
    };
    last_index_of_from(haystack, &encoded, from)
}

fn encode_char(ch: i32) -> Option<Vec<u16>> {
    let ch = u32::try_from(ch).ok()?;
    // A lone SURROGATE is a perfectly findable code unit — `indexOf(0xD83D)`
    // asks about the high half of an emoji, which the string really contains.
    // `char::from_u32` rejects one, so every such search answered -1.
    if (0xD800..0xE000).contains(&ch) {
        return Some(vec![u16::try_from(ch).ok()?]);
    }
    let c = char::from_u32(ch)?;
    let mut buffer = [0u16; 2];
    Some(c.encode_utf16(&mut buffer).to_vec())
}

/// `indexOf(needle, fromIndex)` with Java's clamping.
fn index_of_from(haystack: &[u16], needle: &[u16], from: i32) -> i32 {
    let start = usize::try_from(from.max(0)).unwrap_or(0);
    if needle.is_empty() {
        return i32::try_from(start.min(haystack.len())).unwrap_or(i32::MAX);
    }
    if start >= haystack.len() || needle.len() > haystack.len() - start {
        return -1;
    }
    for at in start..=(haystack.len() - needle.len()) {
        if &haystack[at..at + needle.len()] == needle {
            return i32::try_from(at).unwrap_or(i32::MAX);
        }
    }
    -1
}

/// `lastIndexOf(needle, fromIndex)`: rightmost match at or before
/// `from`, with Java's clamping.
fn last_index_of_from(haystack: &[u16], needle: &[u16], from: i32) -> i32 {
    if from < 0 {
        return -1;
    }
    let limit = usize::try_from(from)
        .unwrap_or(usize::MAX)
        .min(haystack.len().saturating_sub(needle.len()));
    if needle.is_empty() {
        return i32::try_from(
            usize::try_from(from)
                .unwrap_or(usize::MAX)
                .min(haystack.len()),
        )
        .unwrap_or(i32::MAX);
    }
    if needle.len() > haystack.len() {
        return -1;
    }
    for at in (0..=limit).rev() {
        if haystack[at..].starts_with(needle) {
            return i32::try_from(at).unwrap_or(i32::MAX);
        }
    }
    -1
}

fn substring(
    heap: &mut Heap,
    units: &[u16],
    begin: i32,
    len: i32,
) -> Result<Option<JValue>, VmError> {
    // JDK 11's single-arg `substring(beginIndex)` reports "String index out of
    // range: N": a negative beginIndex reports beginIndex itself, and an
    // over-long one reports the negative subLen (`length - beginIndex`). This
    // differs from the two-arg form's "begin/end/length" wording.
    if begin < 0 {
        return Err(throw(format!(
            "java.lang.StringIndexOutOfBoundsException: String index out of range: {begin}"
        )));
    }
    let sub_len = len - begin;
    if sub_len < 0 {
        return Err(throw(format!(
            "java.lang.StringIndexOutOfBoundsException: String index out of range: {sub_len}"
        )));
    }
    let begin_usize = usize::try_from(begin).unwrap_or(0).min(units.len());
    let reference = heap.alloc(HeapObject::JavaString(units[begin_usize..].to_vec()));
    Ok(Some(JValue::Ref(Some(reference))))
}

/// `StringBuilder.substring(start, end)` — like `String.substring` but with
/// the builder's `start`/`end` message wording (String says `begin`), and it
/// does NOT clamp `end` (an over-long `end` is an error, unlike `delete`).
fn builder_substring(
    heap: &mut Heap,
    units: &[u16],
    start: i32,
    end: i32,
) -> Result<Option<JValue>, VmError> {
    let length = units.len();
    let bad = || {
        throw(format!(
            "java.lang.StringIndexOutOfBoundsException: start {start}, end {end}, length {length}"
        ))
    };
    let valid = usize::try_from(start)
        .ok()
        .zip(usize::try_from(end).ok())
        .filter(|(s, e)| s <= e && *e <= length);
    let Some((start, end)) = valid else {
        return Err(bad());
    };
    let reference = heap.alloc(HeapObject::JavaString(units[start..end].to_vec()));
    Ok(Some(JValue::Ref(Some(reference))))
}

fn substring_range(
    heap: &mut Heap,
    units: &[u16],
    begin: i32,
    end: i32,
) -> Result<Option<JValue>, VmError> {
    let length = units.len();
    let valid = usize::try_from(begin)
        .ok()
        .zip(usize::try_from(end).ok())
        .filter(|(b, e)| b <= e && *e <= length);
    let Some((begin_usize, end_usize)) = valid else {
        return Err(throw(format!(
            "java.lang.StringIndexOutOfBoundsException: begin {begin}, end {end}, \
             length {length}"
        )));
    };
    let reference = heap.alloc(HeapObject::JavaString(
        units[begin_usize..end_usize].to_vec(),
    ));
    Ok(Some(JValue::Ref(Some(reference))))
}

/// `String.indexOf(String)` over UTF-16 units (-1 when absent).
fn index_of(haystack: &[u16], needle: &[u16]) -> i32 {
    if needle.is_empty() {
        return 0;
    }
    if needle.len() > haystack.len() {
        return -1;
    }
    for start in 0..=(haystack.len() - needle.len()) {
        if &haystack[start..start + needle.len()] == needle {
            return i32::try_from(start).unwrap_or(i32::MAX);
        }
    }
    -1
}

/// `String.compareTo` semantics: difference of first differing code
/// unit, else length difference.
pub(crate) fn compare_utf16(a: &[u16], b: &[u16]) -> i32 {
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            return i32::from(*x) - i32::from(*y);
        }
    }
    i32::try_from(a.len()).unwrap_or(i32::MAX) - i32::try_from(b.len()).unwrap_or(i32::MAX)
}

/// Compile a pattern, reporting a bad one as Java's `PatternSyntaxException`.
fn compile_regex(pattern: &[u16]) -> Result<crate::regex::Regex, VmError> {
    crate::regex::Regex::new(pattern).map_err(|error| {
        throw(format!(
            "java.util.regex.PatternSyntaxException: {}",
            error.message()
        ))
    })
}

/// Successive matches of `regex` in `input`, as Java's `Matcher.find` walks
/// them: each search resumes at the previous match's end, and a zero-width
/// match advances one unit so the loop cannot stall.
fn find_matches(regex: &crate::regex::Regex, input: &[u16]) -> Vec<crate::regex::Match> {
    let mut found = Vec::new();
    let mut at = 0;
    while at <= input.len() {
        let Some(one) = regex.find_at(input, at) else {
            break;
        };
        at = if one.end == one.start {
            one.end + 1
        } else {
            one.end
        };
        found.push(one);
    }
    found
}

/// `String.split(regex, limit)` — JDK 11's algorithm, step for step.
///
/// The delimiter is a REGULAR EXPRESSION, not a literal. Several rules here
/// look arbitrary but are load-bearing, and each one is a case caturra used to
/// get wrong:
///
/// * a zero-width match at position 0 contributes no leading `""`;
/// * if nothing matched at all, the result is the whole input as ONE element,
///   which is why `"".split(",")` is `[""]` and not `[]`;
/// * `limit > 0` caps the parts, with the last one holding the entire rest;
/// * `limit == 0` drops trailing empty strings, `limit < 0` keeps them.
fn split_regex(input: &[u16], pattern: &[u16], limit: i32) -> Result<Vec<Vec<u16>>, VmError> {
    let regex = compile_regex(pattern)?;
    let match_limited = limit > 0;
    let cap = usize::try_from(limit).unwrap_or(0);
    let mut parts: Vec<Vec<u16>> = Vec::new();
    let mut index = 0;

    for one in find_matches(&regex, input) {
        if !match_limited || parts.len() < cap - 1 {
            // No empty leading substring for a zero-width match at the very
            // beginning of the input.
            if index == 0 && one.start == 0 && one.end == 0 {
                continue;
            }
            parts.push(input[index..one.start].to_vec());
            index = one.end;
        } else if parts.len() == cap - 1 {
            // The last permitted part takes everything that is left.
            parts.push(input[index..].to_vec());
            index = one.end;
        }
    }

    // No match anywhere: the input comes back whole and untouched.
    if index == 0 {
        return Ok(vec![input.to_vec()]);
    }
    if !match_limited || parts.len() < cap {
        parts.push(input[index..].to_vec());
    }
    if limit == 0 {
        while parts.last().is_some_and(Vec::is_empty) {
            parts.pop();
        }
    }
    Ok(parts)
}

/// `String.matches` — the pattern must match the entire string.
fn matches_regex(input: &[u16], pattern: &[u16]) -> Result<bool, VmError> {
    Ok(compile_regex(pattern)?.matches_whole(input))
}

/// `String.replaceAll` / `replaceFirst`.
///
/// The replacement is not literal: `$1` inserts a group and `\` escapes the
/// next character, matching `Matcher.appendReplacement`. A `$` naming a group
/// that does not exist is an error in Java, not a literal dollar.
fn replace_regex(
    input: &[u16],
    pattern: &[u16],
    replacement: &[u16],
    first_only: bool,
) -> Result<Vec<u16>, VmError> {
    let regex = compile_regex(pattern)?;
    let mut out: Vec<u16> = Vec::new();
    let mut index = 0;
    for one in find_matches(&regex, input) {
        if one.start < index {
            // A zero-width match immediately after a consumed one.
            continue;
        }
        out.extend_from_slice(&input[index..one.start]);
        out.extend(expand_replacement(input, &one, replacement, &regex)?);
        index = one.end;
        if first_only {
            break;
        }
    }
    out.extend_from_slice(&input[index..]);
    Ok(out)
}

/// Expand `$n` group references and `\` escapes in a replacement string.
fn expand_replacement(
    input: &[u16],
    one: &crate::regex::Match,
    replacement: &[u16],
    regex: &crate::regex::Regex,
) -> Result<Vec<u16>, VmError> {
    let mut out: Vec<u16> = Vec::new();
    let mut at = 0;
    while at < replacement.len() {
        let unit = replacement[at];
        if unit == u16::from(b'\\') {
            at += 1;
            // A replacement ending in a LONE backslash is an error: the JDK's
            // `appendExpandedReplacement` says "character to be escaped is
            // missing". Silently dropping it accepted a replacement a JDK
            // refuses.
            if at >= replacement.len() {
                return Err(throw(String::from(
                    "java.lang.IllegalArgumentException: character to be escaped is missing",
                )));
            }
            out.push(replacement[at]);
            at += 1;
            continue;
        }
        if unit != u16::from(b'$') {
            out.push(unit);
            at += 1;
            continue;
        }
        at += 1;
        // `${name}` names a group written as `(?<name>X)`.
        if replacement.get(at) == Some(&u16::from(b'{')) {
            let mut name = String::new();
            let mut cursor = at + 1;
            while let Some(unit) = replacement.get(cursor).copied() {
                if unit == u16::from(b'}') {
                    break;
                }
                match u8::try_from(unit) {
                    Ok(byte) if byte.is_ascii_alphanumeric() => name.push(char::from(byte)),
                    _ => {
                        return Err(throw(String::from(
                            "java.lang.IllegalArgumentException: \
                             named capturing group is missing trailing '}'",
                        )));
                    }
                }
                cursor += 1;
            }
            if replacement.get(cursor) != Some(&u16::from(b'}')) {
                return Err(throw(String::from(
                    "java.lang.IllegalArgumentException: \
                     named capturing group is missing trailing '}'",
                )));
            }
            let Some(index) = regex.group_named(&name) else {
                return Err(throw(format!(
                    "java.lang.IllegalArgumentException: No group with name {{{name}}}"
                )));
            };
            if let Some((from, to)) = one.groups.get(index).copied().flatten() {
                out.extend_from_slice(&input[from..to]);
            }
            at = cursor + 1;
            continue;
        }
        // Java takes the longest run of digits that still names a group.
        let mut group = None;
        while at < replacement.len() {
            let digit = replacement[at];
            if !(0x30..=0x39).contains(&digit) {
                break;
            }
            // The FIRST digit is consumed unconditionally (the JDK's rule);
            // further digits extend the number only while it stays a valid
            // group. `$5` against a groupless pattern is therefore group 5 —
            // and an out-of-range group is `Matcher.group`'s
            // IndexOutOfBoundsException, not an illegal reference.
            let extended = group.unwrap_or(0) * 10 + usize::from(digit - 0x30);
            if group.is_some() && extended > regex.group_count() {
                break;
            }
            group = Some(extended);
            at += 1;
        }
        let Some(group) = group else {
            // A bare `$` at the very END has its own JDK message; a `$`
            // followed by something that is not a digit or `{` is the generic
            // one.
            return Err(throw(String::from(if at >= replacement.len() {
                "java.lang.IllegalArgumentException: Illegal group reference: group index is missing"
            } else {
                "java.lang.IllegalArgumentException: Illegal group reference"
            })));
        };
        if group > regex.group_count() {
            return Err(throw(format!(
                "java.lang.IndexOutOfBoundsException: No group {group}"
            )));
        }
        if let Some(Some((from, to))) = one.groups.get(group).copied() {
            out.extend_from_slice(&input[from..to]);
        }
    }
    Ok(out)
}

/// `String.replace(CharSequence, CharSequence)` — literal in Java too.
fn replace_units(haystack: &[u16], from: &[u16], to: &[u16]) -> Vec<u16> {
    if from.is_empty() {
        // Java inserts `to` between every character.
        let mut out = to.to_vec();
        for unit in haystack {
            out.push(*unit);
            out.extend_from_slice(to);
        }
        return out;
    }
    let mut out = Vec::with_capacity(haystack.len());
    let mut at = 0;
    while at < haystack.len() {
        if at + from.len() <= haystack.len() && &haystack[at..at + from.len()] == from {
            out.extend_from_slice(to);
            at += from.len();
        } else {
            out.push(haystack[at]);
            at += 1;
        }
    }
    out
}

/// `java.util.Scanner` methods, pulling lines from the console on
/// demand. Tokens are whitespace-delimited (Java's default).
#[allow(clippy::too_many_lines)] // one arm per Scanner method
fn scanner_method(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    // Every method but `close` refuses a closed Scanner, as the JDK's does.
    // Closing twice is a no-op there, so `close` is exempt.
    let (stdin, closed) = match heap.get(receiver) {
        Some(HeapObject::Scanner { stdin, closed, .. }) => (*stdin, *closed),
        _ => unreachable!("receiver kind checked by caller"),
    };
    if closed && method != "close" {
        return Err(throw("java.lang.IllegalStateException: Scanner closed"));
    }
    match method {
        "nextLine" => {
            let line = scanner_next_line(heap, console, receiver)?
                .ok_or_else(|| throw("java.util.NoSuchElementException: No line found"))?;
            let reference = heap.alloc_string(&line);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        "hasNextLine" => {
            let has = scanner_peek_line(heap, console, receiver)?;
            Ok(Some(JValue::Int(i32::from(has))))
        }
        "nextLong" => {
            let value = scanner_take_msg(heap, console, receiver, |t| {
                let digits = scanner_ungroup(t).ok_or(None)?;
                digits.parse::<i64>().map_err(|_| {
                    scanner_numeric_token(&digits, 10)
                        .then(|| format!("For input string: \"{digits}\""))
                })
            })?;
            Ok(Some(JValue::Long(value)))
        }
        "hasNextLong" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token
                .and_then(|t| scanner_ungroup(&t))
                .is_some_and(|t| t.parse::<i64>().is_ok());
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "nextFloat" => {
            let value = scanner_take(heap, console, receiver, |t| {
                is_java_float_token(t)
                    .then(|| t.parse::<f32>().ok())
                    .flatten()
            })?;
            Ok(Some(JValue::Float(value)))
        }
        "hasNextFloat" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token.is_some_and(|t| is_java_float_token(&t));
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "nextShort" | "nextByte" => {
            let (lo, hi) = if method == "nextShort" {
                (i32::from(i16::MIN), i32::from(i16::MAX))
            } else {
                (i32::from(i8::MIN), i32::from(i8::MAX))
            };
            let value = scanner_take_msg(heap, console, receiver, |t| {
                let digits = scanner_ungroup(t).ok_or(None)?;
                let parsed = digits.parse::<i64>().map_err(|_| None)?;
                if parsed < i64::from(lo) || parsed > i64::from(hi) {
                    return Err(Some(format!(
                        "Value out of range. Value:\"{digits}\" Radix:10"
                    )));
                }
                i32::try_from(parsed).map_err(|_| None)
            })?;
            Ok(Some(JValue::Int(value)))
        }
        "hasNextShort" | "hasNextByte" => {
            let (lo, hi) = if method == "hasNextShort" {
                (i32::from(i16::MIN), i32::from(i16::MAX))
            } else {
                (i32::from(i8::MIN), i32::from(i8::MAX))
            };
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token
                .and_then(|t| t.parse::<i32>().ok())
                .is_some_and(|v| v >= lo && v <= hi);
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "nextBoolean" => {
            let value = scanner_take(heap, console, receiver, |t| {
                if t.eq_ignore_ascii_case("true") {
                    Some(1)
                } else if t.eq_ignore_ascii_case("false") {
                    Some(0)
                } else {
                    None
                }
            })?;
            Ok(Some(JValue::Int(value)))
        }
        "hasNextBoolean" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token
                .is_some_and(|t| t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("false"));
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        // `close()` on a `System.in` scanner closes the stream itself, so a
        // later `new Scanner(System.in)` reads nothing — the JDK does this,
        // and a program that closes stdin and reads again dies on a real JVM.
        // A file scanner closes only itself.
        "close" => {
            if let Some(HeapObject::Scanner { closed, .. }) = heap.get_mut(receiver) {
                *closed = true;
            }
            if stdin {
                console.close_stdin();
            }
            Ok(None)
        }
        // `useDelimiter(pattern)` answers the SCANNER, so it chains onto the
        // constructor the way a program writes it.
        "useDelimiter" => {
            let pattern = match args.first() {
                Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                _ => None,
            };
            let Some(pattern) = pattern else {
                return Err(throw("java.lang.NullPointerException"));
            };
            // Reject a malformed pattern HERE, where the JDK's
            // `Pattern.compile` does, rather than at the first read.
            compile_scanner_delimiter(&pattern)?;
            if let Some(HeapObject::Scanner { delimiter, .. }) = heap.get_mut(receiver) {
                *delimiter = Some(pattern);
            }
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        // `next(pattern)` — the next token, but only if it MATCHES the
        // pattern in full; anything else is an `InputMismatchException`, the
        // same refusal `nextInt` gives a token that is not a number. The token
        // is not consumed when it does not match, so a program can try another
        // pattern (which is the whole point of the overload).
        "next" if !args.is_empty() => {
            let pattern = scanner_pattern_arg(heap, args)?;
            let Some((token, consumed)) = scanner_scan(heap, console, receiver)? else {
                return Err(scanner_exhausted(heap, receiver));
            };
            if !scanner_token_matches(&token, &pattern)? {
                scanner_skip_to_token(heap, receiver, &token, consumed);
                return Err(throw("java.util.InputMismatchException"));
            }
            scanner_next_token(heap, console, receiver)?;
            let reference = heap.alloc_string(&token);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        "next" => {
            let Some(token) = scanner_next_token(heap, console, receiver)? else {
                return Err(scanner_exhausted(heap, receiver));
            };
            let reference = heap.alloc_string(&token);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        "hasNext" if !args.is_empty() => {
            let pattern = scanner_pattern_arg(heap, args)?;
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = match token {
                Some(token) => scanner_token_matches(&token, &pattern)?,
                None => false,
            };
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "hasNext" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            Ok(Some(JValue::Int(i32::from(token.is_some()))))
        }
        "nextInt" => {
            let radix = match args {
                [JValue::Int(radix)] => u32::try_from(*radix).unwrap_or(10),
                _ => 10,
            };
            let value = scanner_take_msg(heap, console, receiver, |t| {
                let digits = scanner_ungroup(t).ok_or(None)?;
                i32::from_str_radix(&digits, radix).map_err(|_| {
                    // A token that IS a number but does not fit reports the
                    // parse failure; anything else is a plain mismatch.
                    scanner_numeric_token(&digits, radix)
                        .then(|| format!("For input string: \"{digits}\""))
                })
            })?;
            Ok(Some(JValue::Int(value)))
        }
        "hasNextInt" => {
            let radix = match args {
                [JValue::Int(radix)] => u32::try_from(*radix).unwrap_or(10),
                _ => 10,
            };
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token
                .and_then(|t| scanner_ungroup(&t))
                .is_some_and(|t| i32::from_str_radix(&t, radix).is_ok());
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "nextDouble" => {
            let value = scanner_take(heap, console, receiver, |t| {
                let t = scanner_ungroup(t)?;
                is_java_float_token(&t)
                    .then(|| t.parse::<f64>().ok())
                    .flatten()
            })?;
            Ok(Some(JValue::Double(value)))
        }
        "hasNextDouble" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token.is_some_and(|t| is_java_float_token(&t));
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Scanner.{method}"))),
    }
}

/// `java.io.BufferedReader`/`FileReader` methods. A file reader hands out lines
/// from its slurped buffer; a `System.in` reader pulls each line from the
/// console. `readLine` returns null at end of stream (not an exception).
fn reader_method(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
    method: &str,
) -> Result<Option<JValue>, VmError> {
    let (stdin, closed) = match heap.get(receiver) {
        Some(HeapObject::Reader { stdin, closed, .. }) => (*stdin, *closed),
        _ => unreachable!("receiver kind checked by caller"),
    };
    match method {
        "readLine" => {
            let line = if closed {
                None
            } else if stdin {
                console.read_line()
            } else {
                reader_next_line(heap, receiver)
            };
            Ok(Some(match line {
                Some(text) => JValue::Ref(Some(heap.alloc_string(&text))),
                None => JValue::Ref(None),
            }))
        }
        "read" => {
            let ch = if closed || stdin {
                -1
            } else {
                reader_next_char(heap, receiver)
            };
            Ok(Some(JValue::Int(ch)))
        }
        "ready" => {
            let ready = !closed
                && !stdin
                && matches!(heap.get(receiver), Some(HeapObject::Reader { buffer, pos, .. }) if *pos < buffer.len());
            Ok(Some(JValue::Int(i32::from(ready))))
        }
        "close" => {
            if let Some(HeapObject::Reader { closed, .. }) = heap.get_mut(receiver) {
                *closed = true;
            }
            Ok(None)
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "BufferedReader.{method}"
        ))),
    }
}

/// The next line from a file reader's buffer, without its terminator, or `None`
/// at end. A line ends at `\n`, `\r`, or `\r\n` (both a real `BufferedReader`
/// and this treat them alike). `\n`/`\r` are ASCII, so scanning bytes never splits a
/// multi-byte UTF-8 character.
fn reader_next_line(heap: &mut Heap, receiver: HeapRef) -> Option<String> {
    let Some(HeapObject::Reader { buffer, pos, .. }) = heap.get_mut(receiver) else {
        return None;
    };
    if *pos >= buffer.len() {
        return None;
    }
    let bytes = buffer.as_bytes();
    let start = *pos;
    let mut end = start;
    while end < bytes.len() && bytes[end] != b'\n' && bytes[end] != b'\r' {
        end += 1;
    }
    let line = buffer[start..end].to_string();
    let mut next = end;
    if next < bytes.len() {
        if bytes[next] == b'\r' {
            next += 1;
            if next < bytes.len() && bytes[next] == b'\n' {
                next += 1;
            }
        } else {
            next += 1; // '\n'
        }
    }
    *pos = next;
    Some(line)
}

/// The next character from a file reader's buffer as its code point, or -1 at
/// end. (A supplementary character would arrive as two UTF-16 units on a JVM;
/// this hands back the code point, which agrees for the Basic Multilingual
/// Plane a `read()` almost always sees.)
fn reader_next_char(heap: &mut Heap, receiver: HeapRef) -> i32 {
    let Some(HeapObject::Reader { buffer, pos, .. }) = heap.get_mut(receiver) else {
        return -1;
    };
    let Some(ch) = buffer[*pos..].chars().next() else {
        return -1;
    };
    *pos += ch.len_utf8();
    i32::try_from(u32::from(ch)).unwrap_or(-1)
}

/// Pull one more line of input into the scanner's buffer. Returns
/// whether a line was added.
fn scanner_fill(heap: &mut Heap, console: &mut dyn ConsoleIo, receiver: HeapRef) {
    let line = console.read_line();
    let Some(HeapObject::Scanner { buffer, eof, .. }) = heap.get_mut(receiver) else {
        unreachable!("receiver kind checked by caller");
    };
    if let Some(line) = line {
        buffer.push_str(&line);
        buffer.push('\n');
    } else {
        *eof = true;
    }
}

fn scanner_state(heap: &Heap, receiver: HeapRef) -> (String, usize, bool) {
    match heap.get(receiver) {
        Some(HeapObject::Scanner {
            buffer, pos, eof, ..
        }) => (buffer.clone(), *pos, *eof),
        _ => unreachable!("receiver kind checked by caller"),
    }
}

fn scanner_set_pos(heap: &mut Heap, receiver: HeapRef, new_pos: usize) {
    if let Some(HeapObject::Scanner { pos, .. }) = heap.get_mut(receiver) {
        *pos = new_pos;
    }
}

/// Read up to the next newline (consuming it); `None` at EOF.
/// Drop a single trailing carriage return, so a `\r\n`-terminated line reads
/// the way the JDK's `nextLine` returns it (terminator excluded).
fn strip_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

fn scanner_next_line(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<Option<String>, VmError> {
    loop {
        let (buffer, pos, eof) = scanner_state(heap, receiver);
        if let Some(offset) = buffer[pos..].find('\n') {
            // The JDK's line separator matches `\r\n` as one terminator, so a
            // CRLF source yields `a`, not `a\r` — strip the carriage return.
            let line = strip_cr(&buffer[pos..pos + offset]).to_owned();
            scanner_set_pos(heap, receiver, pos + offset + 1);
            return Ok(Some(line));
        }
        if eof {
            // Trailing text without a newline still counts as a line.
            if pos < buffer.len() {
                let line = strip_cr(&buffer[pos..]).to_owned();
                scanner_set_pos(heap, receiver, buffer.len());
                return Ok(Some(line));
            }
            return Ok(None);
        }
        scanner_fill(heap, console, receiver);
    }
}

fn scanner_peek_line(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<bool, VmError> {
    loop {
        let (buffer, pos, eof) = scanner_state(heap, receiver);
        if buffer[pos..].contains('\n') || (eof && pos < buffer.len()) {
            return Ok(true);
        }
        if eof {
            return Ok(false);
        }
        scanner_fill(heap, console, receiver);
    }
}

/// Read one token and translate it, consuming it only if the translation works.
///
/// `java.util.Scanner` "will not pass the token that caused the exception", so
/// after `nextInt()` throws `InputMismatchException` the offending token is
/// still there — which is what makes the usual recovery loop
/// (`catch (InputMismatchException e) { in.next(); }`) skip the bad word rather
/// than the one after it. `None` from `parse` means a mismatch.
/// A `java.util.Scanner` integer token with its LOCALE grouping separators
/// removed — the default locale's is `,`, and the JDK accepts it only in
/// well-formed groups: `1,234` and `1,234,567` are numbers, `1,23` and
/// `,123` are not. `None` when the grouping is malformed.
/// Whether a token is a well-formed numeral in `radix` — the JDK reports a
/// parse failure ("For input string") only for one of these; anything else
/// is a plain mismatch with no message.
fn scanner_numeric_token(token: &str, radix: u32) -> bool {
    let digits = token.strip_prefix(['-', '+']).unwrap_or(token);
    !digits.is_empty() && digits.chars().all(|c| c.is_digit(radix))
}

fn scanner_ungroup(token: &str) -> Option<String> {
    if !token.contains(',') {
        return Some(token.to_owned());
    }
    // A floating token groups its INTEGER part only (`1,234.5`); the
    // fraction and exponent come along untouched.
    if let Some((integer, rest)) = token.split_once('.') {
        let integer = scanner_ungroup(integer)?;
        return Some(format!("{integer}.{rest}"));
    }
    let (sign, digits) = match token.strip_prefix(['-', '+']) {
        Some(rest) => (&token[..1], rest),
        None => ("", token),
    };
    let groups: Vec<&str> = digits.split(',').collect();
    let [first, rest @ ..] = groups.as_slice() else {
        return None;
    };
    if rest.is_empty()
        || first.is_empty()
        || first.len() > 3
        || !first.bytes().all(|b| b.is_ascii_digit())
        || !rest
            .iter()
            .all(|group| group.len() == 3 && group.bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    Some(format!("{sign}{first}{}", rest.concat()))
}

/// `scanner_take`, with the JDK's `InputMismatchException` MESSAGE: a token
/// that is a well-formed number but does not fit reports the failure the
/// underlying parse would have ("For input string", or the range complaint
/// the byte/short paths give).
/// The exception a token read raises when the input has run out — and the
/// SIDE EFFECT that comes with it. A JDK's `next()` skips delimiters looking
/// for a token, and the whitespace it skipped stays skipped: after a failed
/// `next()` at the end of input the position is at the end, so a `hasNextLine()`
/// that followed answered false where caturra, which left the cursor where it
/// was, still saw the trailing newline as a line.
fn scanner_exhausted(heap: &mut Heap, receiver: HeapRef) -> VmError {
    let (buffer, _, _) = scanner_state(heap, receiver);
    scanner_set_pos(heap, receiver, buffer.len());
    throw("java.util.NoSuchElementException")
}

fn scanner_take_msg<T>(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
    parse: impl FnOnce(&str) -> Result<T, Option<String>>,
) -> Result<T, VmError> {
    let Some((token, consumed)) = scanner_scan(heap, console, receiver)? else {
        return Err(scanner_exhausted(heap, receiver));
    };
    let value = match parse(&token) {
        Ok(value) => value,
        Err(message) => {
            scanner_skip_to_token(heap, receiver, &token, consumed);
            return Err(match message {
                Some(message) => throw(format!("java.util.InputMismatchException: {message}")),
                None => throw("java.util.InputMismatchException"),
            });
        }
    };
    scanner_next_token(heap, console, receiver)?;
    Ok(value)
}

fn scanner_take<T>(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Result<T, VmError> {
    let Some((token, consumed)) = scanner_scan(heap, console, receiver)? else {
        return Err(scanner_exhausted(heap, receiver));
    };
    let Some(value) = parse(&token) else {
        scanner_skip_to_token(heap, receiver, &token, consumed);
        return Err(throw("java.util.InputMismatchException"));
    };
    scanner_next_token(heap, console, receiver)?;
    Ok(value)
}

/// Whether a token is a float/double literal *as `java.util.Scanner` reads it*:
/// an optional sign, then either exactly `NaN` / `Infinity` or a decimal number.
///
/// Rust's `from_str` is more generous — it also takes `nan`, `inf`, `infinity`
/// in any case — so parsing directly would make `hasNextDouble("inf")` true
/// where Java says false. Anything with a letter other than an exponent `e`/`E`
/// is rejected, which also covers `1.5f`, `1.5d` and `0x10` exactly as Java does.
fn is_java_float_token(token: &str) -> bool {
    let digits = token.strip_prefix(['-', '+']).unwrap_or(token);
    if digits == "NaN" || digits == "Infinity" {
        return true;
    }
    if digits.is_empty()
        || digits
            .bytes()
            .any(|b| b.is_ascii_alphabetic() && b != b'e' && b != b'E')
    {
        return false;
    }
    token.parse::<f64>().is_ok()
}

/// Advance past whitespace and read one token; `None` at EOF.
fn scanner_next_token(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<Option<String>, VmError> {
    let Some((token, consumed)) = scanner_scan(heap, console, receiver)? else {
        return Ok(None);
    };
    let (_, pos, _) = scanner_state(heap, receiver);
    scanner_set_pos(heap, receiver, pos + consumed);
    Ok(Some(token))
}

fn scanner_peek_token(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<Option<String>, VmError> {
    Ok(scanner_scan(heap, console, receiver)?.map(|(token, _)| token))
}

/// Move the cursor to the START of the next token, past the delimiters before
/// it. A JDK's failed `nextInt()` "will not pass the token that caused the
/// exception" — but it does not put back the WHITESPACE it skipped either, so
/// the `nextLine()` that follows answers the rest of the line from the token
/// onwards. caturra left the cursor where it was and returned the leading
/// spaces too.
fn scanner_skip_to_token(heap: &mut Heap, receiver: HeapRef, token: &str, consumed: usize) {
    let (_, pos, _) = scanner_state(heap, receiver);
    scanner_set_pos(heap, receiver, pos + consumed.saturating_sub(token.len()));
}

/// The next token AND how many bytes reading it consumes — the token itself
/// plus whatever separated it from the last one. `next()` and `hasNext()` are
/// the same scan; only one of them moves the cursor, which is why the two
/// answers travel together (a token that is EMPTY, which a custom delimiter
/// can produce, has no text to search for afterwards).
fn scanner_scan(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<Option<(String, usize)>, VmError> {
    loop {
        let (buffer, pos, eof) = scanner_state(heap, receiver);
        let rest = &buffer[pos..];
        if let Some(pattern) = scanner_delimiter(heap, receiver) {
            // A CUSTOM delimiter: the token is whatever precedes the next
            // match, and an empty one between two delimiters is a token like
            // any other (`"a,,b"` split on `,` yields three). Reading on until
            // a delimiter or EOF is the same rule the default follows — a
            // token is only complete once something ends it.
            let regex = compile_scanner_delimiter(&pattern)?;
            let units: Vec<u16> = rest.encode_utf16().collect();
            // A token is `delimiter? token delimiter?` in the JDK: ONE
            // delimiter match at the cursor is skipped before reading, and the
            // one after it is LEFT for the next call. Consuming the trailing
            // one instead looks the same until the edges: `",a"` would answer
            // an empty token first (a JDK answers `a`), and a `useDelimiter`
            // between two reads would start after a separator the new pattern
            // no longer treats as one.
            let start = match regex.find_at(&units, 0) {
                Some(found) if found.start == 0 && found.end > 0 => found.end,
                _ => 0,
            };
            if let Some(found) = regex.find_at(&units, start)
                && found.end > found.start
            {
                let token = String::from_utf16_lossy(&units[start..found.start]);
                let upto = String::from_utf16_lossy(&units[..found.start]);
                return Ok(Some((token, upto.len())));
            }
            if eof {
                let token = String::from_utf16_lossy(&units[start..]);
                let consumed = rest.len();
                return Ok((!token.is_empty()).then_some((token, consumed)));
            }
            scanner_fill(heap, console, receiver);
            continue;
        }
        let trimmed = rest.trim_start();
        if !trimmed.is_empty() {
            // A complete token needs trailing whitespace or EOF.
            let token: String = trimmed.chars().take_while(|c| !c.is_whitespace()).collect();
            if trimmed.len() > token.len() || eof {
                let leading = rest.len() - trimmed.len();
                let consumed = leading + token.len();
                return Ok(Some((token, consumed)));
            }
        }
        if eof {
            return Ok(None);
        }
        scanner_fill(heap, console, receiver);
    }
}

/// The delimiter pattern `useDelimiter` set, if any.
fn scanner_delimiter(heap: &Heap, receiver: HeapRef) -> Option<String> {
    match heap.get(receiver) {
        Some(HeapObject::Scanner { delimiter, .. }) => delimiter.clone(),
        _ => None,
    }
}

/// The pattern argument of `hasNext(String)` / `next(String)`.
fn scanner_pattern_arg(heap: &Heap, args: &[JValue]) -> Result<String, VmError> {
    match args.first() {
        Some(JValue::Ref(Some(reference))) => Ok(heap.string_text(*reference).unwrap_or_default()),
        _ => Err(throw("java.lang.NullPointerException")),
    }
}

/// Whether a token matches a pattern IN FULL — `String.matches` semantics,
/// which is what the JDK's `hasNext(String)` asks of the token it peeked.
fn scanner_token_matches(token: &str, pattern: &str) -> Result<bool, VmError> {
    let token: Vec<u16> = token.encode_utf16().collect();
    let pattern: Vec<u16> = pattern.encode_utf16().collect();
    matches_regex(&token, &pattern)
}

/// The delimiter as a compiled pattern. A malformed one is the JDK's
/// `PatternSyntaxException`, thrown from `useDelimiter` itself — but caturra
/// compiles it lazily, so it surfaces at the first read.
fn compile_scanner_delimiter(pattern: &str) -> Result<crate::regex::Regex, VmError> {
    let units: Vec<u16> = pattern.encode_utf16().collect();
    crate::regex::Regex::new(&units).map_err(|error| {
        throw(format!(
            "java.util.regex.PatternSyntaxException: {}",
            error.message()
        ))
    })
}

/// Element equality for list membership: value equality for numbers
/// and strings (Java's `equals`), reference equality for user objects
/// (correct here, since `equals` overriding is not supported).
/// The elements `System.arraycopy` lifts out of the source before writing
/// them, so that a copy within one array behaves as Java's does: as if it
/// went through a temporary.
enum ArrayChunk {
    Int(Vec<i32>),
    Double(Vec<f64>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Short(Vec<i16>),
    Byte(Vec<i8>),
    Ref(Vec<JValue>),
}

/// `System.arraycopy(src, srcPos, dest, destPos, length)`.
/// The `count` elements of `source` starting at `from`, as a chunk that
/// borrows nothing — `arraycopy` writes back into the same heap, and a
/// self-copy must read the old values.
fn take_chunk(heap: &Heap, source: HeapRef, from: usize, count: usize) -> Option<ArrayChunk> {
    Some(match heap.get(source) {
        Some(HeapObject::IntArray(_, values)) => {
            ArrayChunk::Int(values[from..from + count].to_vec())
        }
        Some(HeapObject::DoubleArray(values)) => {
            ArrayChunk::Double(values[from..from + count].to_vec())
        }
        Some(HeapObject::LongArray(values)) => {
            ArrayChunk::Long(values[from..from + count].to_vec())
        }
        Some(HeapObject::FloatArray(values)) => {
            ArrayChunk::Float(values[from..from + count].to_vec())
        }
        Some(HeapObject::ShortArray(values)) => {
            ArrayChunk::Short(values[from..from + count].to_vec())
        }
        Some(HeapObject::ByteArray(values)) => {
            ArrayChunk::Byte(values[from..from + count].to_vec())
        }
        Some(HeapObject::RefArray(_, values)) => {
            ArrayChunk::Ref(values[from..from + count].to_vec())
        }
        _ => return None,
    })
}

/// The name JDK arraycopy diagnostics give an array's type: the primitive
/// for a primitive array, and the catch-all "object array" for every
/// reference array (the JVM's own wording — it does not name the component).
pub(crate) fn arraycopy_type_name(object: &HeapObject) -> Option<&'static str> {
    Some(match object {
        HeapObject::IntArray(kind, _) => match kind {
            IntKind::Int => "int",
            IntKind::Boolean => "boolean",
            IntKind::Char => "char",
        },
        HeapObject::DoubleArray(_) => "double",
        HeapObject::LongArray(_) => "long",
        HeapObject::FloatArray(_) => "float",
        HeapObject::ShortArray(_) => "short",
        HeapObject::ByteArray(_) => "byte",
        HeapObject::RefArray(_, _) => "object array",
        _ => return None,
    })
}

fn arraycopy_len(object: &HeapObject) -> Option<usize> {
    Some(match object {
        HeapObject::IntArray(_, values) => values.len(),
        HeapObject::DoubleArray(values) => values.len(),
        HeapObject::LongArray(values) => values.len(),
        HeapObject::FloatArray(values) => values.len(),
        HeapObject::ShortArray(values) => values.len(),
        HeapObject::ByteArray(values) => values.len(),
        HeapObject::RefArray(_, values) => values.len(),
        _ => return None,
    })
}

/// `System.arraycopy`, with the JVM's own checks and messages: both
/// arguments must be arrays of the same kind, the range must fit, and a
/// reference copy whose element types differ is checked ELEMENT BY ELEMENT —
/// copying the prefix that fits before throwing, exactly as the JDK does.
///
/// The reference-element check needs the class hierarchy, so it is not done
/// here; the caller (`Interpreter::arrays_static_intrinsic`) supplies it.
#[allow(clippy::too_many_lines)] // the JVM's check sequence, in its order
pub(crate) fn system_arraycopy(
    heap: &mut Heap,
    args: &[JValue],
    fits: &dyn Fn(&Heap, &str, JValue) -> bool,
) -> Result<Option<JValue>, VmError> {
    let [
        JValue::Ref(source),
        JValue::Int(source_pos),
        JValue::Ref(destination),
        JValue::Int(destination_pos),
        JValue::Int(length),
    ] = args
    else {
        return Err(VmError::UnknownIntrinsic(String::from("System.arraycopy")));
    };
    let (Some(source), Some(destination)) = (*source, *destination) else {
        return Err(throw("java.lang.NullPointerException"));
    };
    let (Some(source_object), Some(destination_object)) = (heap.get(source), heap.get(destination))
    else {
        return Err(throw("java.lang.ArrayStoreException: arraycopy"));
    };
    // Not an array at all (the parameters are declared `Object`).
    let not_array = |which: &str, reference: HeapRef| {
        throw(format!(
            "java.lang.ArrayStoreException: arraycopy: {which} type {} is not an array",
            crate::interpreter::heap_binary_name(heap, reference)
        ))
    };
    let Some(source_kind) = arraycopy_type_name(source_object) else {
        return Err(not_array("source", source));
    };
    let Some(destination_kind) = arraycopy_type_name(destination_object) else {
        return Err(not_array("destination", destination));
    };
    // Kinds must match exactly: a `boolean[]` never copies into an `int[]`,
    // even though both hold their elements as 32-bit words.
    if source_kind != destination_kind {
        return Err(throw(format!(
            "java.lang.ArrayStoreException: arraycopy: type mismatch: can not copy \
             {source_kind}[] into {destination_kind}[]"
        )));
    }
    let source_len = arraycopy_len(source_object).unwrap_or(0);
    let destination_len = arraycopy_len(destination_object).unwrap_or(0);

    // The JVM's order: source index, destination index, negative length,
    // then the two last-index checks.
    let bounds = |text: String| throw(format!("java.lang.ArrayIndexOutOfBoundsException: {text}"));
    if *source_pos < 0 {
        return Err(bounds(format!(
            "arraycopy: source index {source_pos} out of bounds for {source_kind}[{source_len}]"
        )));
    }
    if *destination_pos < 0 {
        return Err(bounds(format!(
            "arraycopy: destination index {destination_pos} out of bounds for \
             {destination_kind}[{destination_len}]"
        )));
    }
    if *length < 0 {
        return Err(bounds(format!("arraycopy: length {length} is negative")));
    }
    let (from, to, count) = (
        usize::try_from(*source_pos).unwrap_or(usize::MAX),
        usize::try_from(*destination_pos).unwrap_or(usize::MAX),
        usize::try_from(*length).unwrap_or(usize::MAX),
    );
    if from.saturating_add(count) > source_len {
        return Err(bounds(format!(
            "arraycopy: last source index {} out of bounds for {source_kind}[{source_len}]",
            i64::from(*source_pos) + i64::from(*length)
        )));
    }
    if to.saturating_add(count) > destination_len {
        return Err(bounds(format!(
            "arraycopy: last destination index {} out of bounds for \
             {destination_kind}[{destination_len}]",
            i64::from(*destination_pos) + i64::from(*length)
        )));
    }

    // A reference copy between arrays of DIFFERENT element types is checked
    // per element (JLS §10.5 store semantics): the elements that fit are
    // copied, and the first that does not throws — so a partially-copied
    // destination is observable, as on a real JVM.
    if let (
        Some(HeapObject::RefArray(source_class, _)),
        Some(HeapObject::RefArray(destination_class, _)),
    ) = (heap.get(source), heap.get(destination))
        && source_class != destination_class
    {
        let (source_class, destination_class) = (source_class.clone(), destination_class.clone());
        let element = destination_class.strip_prefix('[').unwrap_or("").to_owned();
        let taken = match heap.get(source) {
            Some(HeapObject::RefArray(_, values)) => values[from..from + count].to_vec(),
            _ => Vec::new(),
        };
        for (offset, value) in taken.into_iter().enumerate() {
            if !fits(heap, &element, value) {
                return Err(throw(format!(
                    "java.lang.ArrayStoreException: arraycopy: element type mismatch: can not \
                     cast one of the elements of {} to the type of the destination array, {}",
                    crate::interpreter::descriptor_type_name(&source_class),
                    crate::interpreter::descriptor_type_name(&element)
                )));
            }
            if let Some(HeapObject::RefArray(_, values)) = heap.get_mut(destination) {
                values[to + offset] = value;
            }
        }
        return Ok(None);
    }

    let store_error = || throw("java.lang.ArrayStoreException: incompatible array types");
    let taken = take_chunk(heap, source, from, count).ok_or_else(store_error)?;
    match (heap.get_mut(destination), taken) {
        (Some(HeapObject::IntArray(_, values)), ArrayChunk::Int(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::DoubleArray(values)), ArrayChunk::Double(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::LongArray(values)), ArrayChunk::Long(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::FloatArray(values)), ArrayChunk::Float(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::ShortArray(values)), ArrayChunk::Short(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::ByteArray(values)), ArrayChunk::Byte(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::RefArray(_, values)), ArrayChunk::Ref(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        _ => return Err(store_error()),
    }
    Ok(None)
}

/// `2^n`, exactly, for `-1022 <= n <= 1023` — every such power is a normal
/// `double`, so it is just an exponent field.
fn power_of_two(exponent: i32) -> f64 {
    f64::from_bits(u64::try_from(exponent + 1023).unwrap_or(0) << 52)
}

/// `Math.scalb(d, n)`: `d * 2^n`, rounded once. The JDK gets the single
/// rounding by scaling in one small step and then in 512-sized ones, rather
/// than by multiplying by `2^n` directly — which would round twice whenever
/// the result underflows into the subnormals.
fn java_scalb_double(value: f64, scale: i32) -> f64 {
    // `Double.MAX_EXPONENT + -Double.MIN_EXPONENT + SIGNIFICAND_WIDTH + 1`:
    // past this, the result is certainly zero or infinity.
    const MAX_SCALE: i32 = 1023 + 1022 + 53 + 1;
    let (mut scale, increment, delta) = if scale < 0 {
        (scale.max(-MAX_SCALE), -512, power_of_two(-512))
    } else {
        (scale.min(MAX_SCALE), 512, power_of_two(512))
    };
    // The remainder of `scale` toward zero, in `[-511, 511]`.
    let toward_zero = ((scale >> 8).cast_unsigned() >> 23).cast_signed();
    let adjust = ((scale + toward_zero) & 511) - toward_zero;

    let mut value = value * power_of_two(adjust);
    scale -= adjust;
    while scale != 0 {
        value *= delta;
        scale -= increment;
    }
    value
}

/// `Math.scalb(f, n)`. A `float` needs no staging: every scale that matters
/// fits in one exact `double` multiply.
fn java_scalb_float(value: f32, scale: i32) -> f32 {
    // `Float.MAX_EXPONENT + -Float.MIN_EXPONENT + SIGNIFICAND_WIDTH + 1`.
    const MAX_SCALE: i32 = 127 + 126 + 24 + 1;
    let scale = scale.clamp(-MAX_SCALE, MAX_SCALE);
    #[allow(clippy::cast_possible_truncation)]
    let scaled = (f64::from(value) * power_of_two(scale)) as f32;
    scaled
}

/// The wrapper class and the primitive behind a value, when it is boxed.
fn unboxed(heap: &Heap, value: JValue) -> (Option<&str>, JValue) {
    if let JValue::Ref(Some(reference)) = value
        && let Some(HeapObject::Boxed { class_name, value }) = heap.get(reference)
    {
        return (Some(class_name), *value);
    }
    (None, value)
}

/// `a.equals(b)` for every value the VM can compare without running Java:
/// primitives, wrappers, strings, and `null`. A user object compares by
/// identity — [`Interpreter::java_equals`] overrides that with its `equals`.
///
/// `Double.equals` and `Float.equals` compare raw bits, not numbers, so
/// `-0.0` differs from `0.0` and `NaN` equals itself. `==` says the opposite
/// of both.
pub(crate) fn native_equals(heap: &Heap, a: JValue, b: JValue) -> bool {
    let (class_a, a) = unboxed(heap, a);
    let (class_b, b) = unboxed(heap, b);
    // `Integer.valueOf(1).equals(Long.valueOf(1))` is false.
    if let (Some(class_a), Some(class_b)) = (class_a, class_b)
        && class_a != class_b
    {
        return false;
    }
    match (a, b) {
        (JValue::Int(x), JValue::Int(y)) => x == y,
        (JValue::Long(x), JValue::Long(y)) => x == y,
        (JValue::Double(x), JValue::Double(y)) => {
            x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan())
        }
        (JValue::Float(x), JValue::Float(y)) => {
            x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan())
        }
        (JValue::Ref(None), JValue::Ref(None)) => true,
        (JValue::Ref(Some(x)), JValue::Ref(Some(y))) => {
            // The VALUE-based library objects. A direct `a.equals(b)` reaches
            // the arms above, but a collection asks HERE — so a `File` whose
            // own `equals` compares pathnames was still identity-compared by
            // `contains`, `indexOf`, `remove` and a Set, which is the one
            // place a program can see the difference.
            x == y
                || match (heap.get(x), heap.get(y)) {
                    (Some(HeapObject::JavaString(sx)), Some(HeapObject::JavaString(sy))) => {
                        sx == sy
                    }
                    (Some(HeapObject::File(sx)), Some(HeapObject::File(sy)))
                    | (Some(HeapObject::Charset(sx)), Some(HeapObject::Charset(sy))) => sx == sy,
                    // A `java.time` value compares by its FIELDS, which is
                    // what makes `dates.contains(LocalDate.of(...))` answer
                    // the way a JDK's does.
                    (Some(HeapObject::Temporal(vx)), Some(HeapObject::Temporal(vy))) => vx == vy,
                    _ => false,
                }
        }
        _ => false,
    }
}

/// A stored value's Java hash code, for `ArrayList.hashCode`.
#[allow(clippy::cast_possible_wrap)]
/// `value.hashCode()` for every value the VM can hash without running Java.
/// A user object hashes by identity — [`Interpreter::java_hash_code`]
/// overrides that with its `hashCode`.
pub(crate) fn native_hash(heap: &Heap, value: JValue) -> i32 {
    let (class, value) = unboxed(heap, value);
    // `Boolean` is the one wrapper whose hash is not its value.
    if class == Some("java/lang/Boolean") {
        return if value == JValue::Int(0) { 1237 } else { 1231 };
    }
    match value {
        // Integer, Short, Byte and Character all hash to their value.
        JValue::Int(v) => v,
        JValue::Long(v) => fold_to_int(v),
        JValue::Double(v) => java_double_hash(v),
        JValue::Float(v) => float_to_int_bits(v),
        JValue::Ref(None) => 0,
        JValue::Ref(Some(reference)) => match heap.get(reference) {
            Some(HeapObject::JavaString(units)) => units.iter().fold(0i32, |hash, unit| {
                hash.wrapping_mul(31).wrapping_add(i32::from(*unit))
            }),
            // Paired with the value equality above: two equal objects have to
            // hash alike or a `HashSet` holds both of them. `UnixFileSystem`
            // xors the path's hash with a constant; a `Charset` hashes as its
            // name.
            Some(HeapObject::File(path)) => java_string_hash(path) ^ 0x0012_d591,
            Some(HeapObject::Charset(name)) => java_string_hash(name),
            // Identity hash (arbitrary in Java too).
            _ => reference.cast_signed(),
        },
    }
}

/// `(int) (v ^ (v >>> 32))`, how `Long` and `Double` hash a 64-bit value.
pub(crate) fn fold_to_int(value: i64) -> i32 {
    (value ^ (value.cast_unsigned() >> 32).cast_signed()) as i32
}

/// `Float.floatToIntBits`, which collapses every NaN payload to one.
fn float_to_int_bits(value: f32) -> i32 {
    if value.is_nan() {
        f32::NAN.to_bits().cast_signed()
    } else {
        value.to_bits().cast_signed()
    }
}

/// `Some(true)` for a method whose descriptor returns `boolean`, else `None`
/// (a void method). Lets one arm serve both an appending `boolean` form
/// (`offer`) and a void one (`addLast`) without unbalancing the caller's stack.
fn boolean_return_if(descriptor: &str) -> Option<JValue> {
    descriptor.ends_with(")Z").then_some(JValue::Int(1))
}

/// `java.util.ArrayDeque` forbids null elements: every insertion throws
/// `NullPointerException` on a null, unlike the null-tolerant `LinkedList`
/// backing it shares. Guards the single-element inserts and — best effort for
/// the common list-backed source — `addAll`, before the shared `list_method`
/// performs the insertion.
fn array_deque_reject_null(heap: &Heap, method: &str, args: &[JValue]) -> Result<(), VmError> {
    let inserts_single = matches!(
        method,
        "add" | "offer" | "addFirst" | "addLast" | "offerFirst" | "offerLast" | "push"
    );
    if inserts_single && matches!(args.first(), Some(JValue::Ref(None))) {
        return Err(throw("java.lang.NullPointerException"));
    }
    if method == "addAll"
        && let Some(JValue::Ref(Some(source))) = args.first()
        && heap
            .list_values(*source)
            .is_some_and(|values| values.iter().any(|v| matches!(v, JValue::Ref(None))))
    {
        return Err(throw("java.lang.NullPointerException"));
    }
    Ok(())
}

/// The `java.util.Stack` LIFO operations, which differ from the like-named
/// deque/list methods: `push`/`pop`/`peek` act on the TOP (the vector's end,
/// not a deque's head), and an empty `pop`/`peek` throws `EmptyStackException`.
/// Every one of them returns a value, so `Ok(None)` unambiguously means "not a
/// stack method" and the caller falls through to the shared `list_method` (a
/// Stack is a `List`). `search` compares elements, so it lives in the
/// interpreter with the other element-comparing list methods.
fn stack_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let Some(values) = heap.list_values_mut(receiver) else {
        unreachable!("stack receiver checked by caller")
    };
    let answer = match (method, args) {
        // `push` returns the pushed element (not a boolean like `add`).
        ("push", [value]) => {
            let value = *value;
            values.push(value);
            value
        }
        ("pop", []) => values
            .pop()
            .ok_or_else(|| throw("java.util.EmptyStackException"))?,
        ("peek", []) => values
            .last()
            .copied()
            .ok_or_else(|| throw("java.util.EmptyStackException"))?,
        ("empty", []) => JValue::Int(i32::from(values.is_empty())),
        _ => return Ok(None),
    };
    Ok(Some(answer))
}

/// `java.util.ArrayList` methods, and the positional `java.util.LinkedList`
/// (`Queue`/`Deque`) methods — both store their elements the same way, so the
/// index-and-end operations that need no element comparison share this layer.
/// (`contains`/`indexOf`/`remove(Object)`/`equals` compare elements, so they
/// live in the interpreter, which can call a user `equals`.)
#[allow(clippy::too_many_lines)] // one arm per documented method
/// The map a `keySet()`/`values()` view iterates, if `source` is one.
fn view_map(heap: &Heap, source: HeapRef) -> Option<HeapRef> {
    match heap.get(source) {
        Some(HeapObject::MapView { map, .. }) => Some(*map),
        _ => None,
    }
}

/// The element count of a collection an iterator walks (list, set, or map view).
/// Throw `ConcurrentModificationException` if the collection changed size
/// since the iterator (or for-each loop) last agreed with it.
///
/// This is the fail-fast check the JDK spells `checkForComodification`. It
/// exists so a program that mutates a collection while walking it fails HERE,
/// loudly, instead of silently skipping an element — before this, removing
/// during a for-each quietly skipped the next one, and adding looped forever.
pub(crate) fn check_comodification(
    heap: &Heap,
    source: HeapRef,
    expected_len: usize,
) -> Result<(), VmError> {
    // A view of a sorted collection is checked against its BACKING, not
    // against itself: a JDK's view cursor carries the TREE's `modCount`, so
    // adding a key OUTSIDE the range still ends a walk of the view with a CME,
    // and the view's own length — which that add never touched — could not
    // notice. `seen` is the length stamped when the walk began.
    if let Some(HeapObject::SortedView { backing, seen, .. }) = heap.get(source) {
        if sorted_backing_pairs(heap, *backing).len() == *seen {
            return Ok(());
        }
        return Err(throw("java.util.ConcurrentModificationException"));
    }
    if iterated_len(heap, source) == expected_len {
        return Ok(());
    }
    Err(throw("java.util.ConcurrentModificationException"))
}

/// `iterated_len` for callers outside this module.
pub(crate) fn iterated_len_of(heap: &Heap, source: HeapRef) -> usize {
    iterated_len(heap, source)
}

fn iterated_len(heap: &Heap, source: HeapRef) -> usize {
    if let Some(values) = heap.list_values(source) {
        return values.len();
    }
    // A `subList` view iterates ITS OWN range, not the backing list's length —
    // reading the latter made an ordinary for-each over a view look like a
    // concurrent modification.
    if let Some(HeapObject::SubList { len, .. }) = heap.get(source) {
        return *len;
    }
    if let Some(map) = view_map(heap, source) {
        return iterated_len(heap, map);
    }
    // A sorted view iterates ITS OWN slice, the same reason a `subList` does.
    if matches!(heap.get(source), Some(HeapObject::SortedView { .. })) {
        let (from, to) = sorted_view_range(heap, source);
        return to.saturating_sub(from);
    }
    // An unmodifiable wrapper iterates the collection it wraps. Missing these
    // made the length read as 0, which the comodification check saw as a
    // change and turned into a spurious CME on a perfectly ordinary for-each.
    if let Some(
        HeapObject::UnmodifiableList(inner)
        | HeapObject::UnmodifiableSet(inner)
        | HeapObject::UnmodifiableMap(inner),
    ) = heap.get(source)
    {
        return iterated_len(heap, *inner);
    }
    match heap.get(source) {
        Some(HeapObject::HashSet(entries) | HeapObject::HashMap(entries)) => entries.len(),
        Some(HeapObject::TreeSet { values, .. }) => values.len(),
        Some(HeapObject::TreeMap { entries, .. }) => entries.len(),
        // A PriorityQueue iterates its HEAP ARRAY, in no particular order —
        // the JDK says so explicitly, and caturra models the array exactly.
        Some(HeapObject::PriorityQueue { heap: items, .. }) => items.len(),
        _ => 0,
    }
}

/// The element at `index` in that collection's iteration order. For a `keySet()`
/// view it is the key; for a `values()` view, the value.
/// The element at `index`, BOXED if it is a primitive — what `Iterator.next`
/// and `ListIterator.previous` hand back (a `List<Integer>` returns an
/// `Integer`, matching a set, whose elements are already references).
fn box_iterated_element(heap: &mut Heap, source: HeapRef, index: usize) -> JValue {
    match iterated_get(heap, source, index) {
        primitive @ (JValue::Int(_) | JValue::Long(_) | JValue::Double(_) | JValue::Float(_)) => {
            let class_name = match primitive {
                JValue::Long(_) => "java/lang/Long",
                JValue::Double(_) => "java/lang/Double",
                JValue::Float(_) => "java/lang/Float",
                _ => "java/lang/Integer",
            };
            let boxed = heap.alloc(HeapObject::Boxed {
                class_name: std::rc::Rc::from(class_name),
                value: primitive,
            });
            JValue::Ref(Some(boxed))
        }
        reference => reference,
    }
}

/// A `subList` view's backing list and the offset its index 0 sits at, so a
/// cursor over one reads and writes the RANGE. Every reader and writer below
/// asks here rather than carrying the arithmetic itself.
fn sublist_target(heap: &Heap, source: HeapRef) -> Option<(HeapRef, usize, usize)> {
    match heap.get(source)? {
        HeapObject::SubList {
            backing, from, len, ..
        } => Some((*backing, *from, *len)),
        _ => None,
    }
}

/// Re-agree a view with its backing after a write THROUGH the view.
fn sublist_resize(heap: &mut Heap, view: HeapRef, delta: isize) {
    if let Some(HeapObject::SubList { len, seen, .. }) = heap.get_mut(view) {
        *len = len.saturating_add_signed(delta);
        *seen = seen.saturating_add_signed(delta);
    }
}

/// Whether a cursor's source hands out its elements the way `HashMap`'s and
/// `TreeMap`'s iterators do — computing the NEXT one as each is returned,
/// rather than comparing a cursor against the collection's current size.
fn hash_like(heap: &Heap, source: HeapRef) -> bool {
    let source = match heap.get(source) {
        Some(HeapObject::MapView { map, .. }) => *map,
        _ => source,
    };
    matches!(
        heap.get(source),
        Some(
            HeapObject::HashMap(_)
                | HeapObject::HashSet(_)
                | HeapObject::TreeMap { .. }
                | HeapObject::TreeSet { .. }
                // A view of a tree hands out a TREE cursor: its `hasNext` is
                // the pointer the last `next()` computed, so adding to the
                // backing mid-loop ends the loop quietly rather than throwing.
                | HeapObject::SortedView { .. }
        )
    )
}

fn iterated_get(heap: &Heap, source: HeapRef, index: usize) -> JValue {
    if let Some(values) = heap.list_values(source) {
        return values.get(index).copied().unwrap_or(JValue::NULL);
    }
    if let Some((backing, from, len)) = sublist_target(heap, source) {
        if index >= len {
            return JValue::NULL;
        }
        return iterated_get(heap, backing, from + index);
    }
    if let Some(HeapObject::MapView { map, kind, .. }) = heap.get(source) {
        return match kind {
            MapViewKind::Values => map_value_at(heap, *map, index),
            _ => map_key_at(heap, *map, index),
        };
    }
    // A sorted view hands out the position within ITS slice, which is where
    // the descending faces get their reversal from.
    if let Some(HeapObject::SortedView { face, .. }) = heap.get(source) {
        let face = *face;
        return sorted_view_pairs(heap, source)
            .get(index)
            .map_or(JValue::NULL, |(key, value)| {
                if matches!(face, crate::value::SortedFace::Map) {
                    *value
                } else {
                    *key
                }
            });
    }
    match heap.get(source) {
        // A HashSet stores its elements as the KEYS of its backing map — in
        // ITERATION order, which is the JDK's bucket order and not the order
        // they were stored in. Indexing the storage directly made an explicit
        // `iterator()` walk a different order from the for-each, the stream and
        // `toString` over the very same set, all three of which ask for the
        // ordered position.
        Some(HeapObject::HashSet(entries)) => {
            entries.entry_at(index).map_or(JValue::NULL, |(key, _)| key)
        }
        Some(HeapObject::TreeSet { values, .. }) => {
            values.get(index).copied().unwrap_or(JValue::NULL)
        }
        Some(HeapObject::PriorityQueue { heap: items, .. }) => {
            items.get(index).copied().unwrap_or(JValue::NULL)
        }
        _ => JValue::NULL,
    }
}

fn map_key_at(heap: &Heap, map: HeapRef, index: usize) -> JValue {
    match heap.get(map) {
        Some(HeapObject::HashMap(entries)) => {
            entries.entry_at(index).map_or(JValue::NULL, |(key, _)| key)
        }
        Some(HeapObject::TreeMap { entries, .. }) => {
            entries.get(index).map_or(JValue::NULL, |(key, _)| *key)
        }
        // `subMap(...).keySet()` is a `MapView` whose map is the VIEW.
        Some(HeapObject::SortedView { .. }) => sorted_view_pairs(heap, map)
            .get(index)
            .map_or(JValue::NULL, |(key, _)| *key),
        _ => JValue::NULL,
    }
}

fn map_value_at(heap: &Heap, map: HeapRef, index: usize) -> JValue {
    match heap.get(map) {
        Some(HeapObject::HashMap(entries)) => entries
            .entry_at(index)
            .map_or(JValue::NULL, |(_, value)| value),
        Some(HeapObject::TreeMap { entries, .. }) => {
            entries.get(index).map_or(JValue::NULL, |(_, value)| *value)
        }
        Some(HeapObject::SortedView { .. }) => sorted_view_pairs(heap, map)
            .get(index)
            .map_or(JValue::NULL, |(_, value)| *value),
        _ => JValue::NULL,
    }
}

/// Remove the element at `index` from the collection an iterator is walking —
/// `Iterator.remove()`, which takes the last element `next()` returned out of the
/// underlying collection. Through a `keySet()`/`values()` view this writes back to
/// the map, as Java's do.
fn iterated_remove(heap: &mut Heap, source: HeapRef, index: usize) {
    if let Some(values) = heap.list_values_mut(source)
        && index < values.len()
    {
        values.remove(index);
        return;
    }
    // Through a view: the element leaves the BACKING list, and the view is one
    // shorter. A cursor's `remove` is the one legal modification during
    // iteration, so the view must re-agree rather than call it a change.
    if let Some((backing, from, len)) = sublist_target(heap, source)
        && index < len
    {
        iterated_remove(heap, backing, from + index);
        sublist_resize(heap, source, -1);
        return;
    }
    let target = view_map(heap, source).unwrap_or(source);
    match heap.get_mut(target) {
        Some(HeapObject::HashSet(entries) | HeapObject::HashMap(entries)) => {
            entries.remove_at(index);
        }
        Some(HeapObject::TreeSet { values, .. }) if index < values.len() => {
            values.remove(index);
        }
        Some(HeapObject::TreeMap { entries, .. }) if index < entries.len() => {
            entries.remove(index);
        }
        _ => {}
    }
}

/// One step of `Iterator.forEachRemaining`: the next element, or `None` once
/// the cursor is spent. `forEachRemaining` lives in the interpreter (it runs a
/// user `accept`), but its stepping is exactly the JDK default's
/// `while (hasNext()) action.accept(next())` — fail-fast check included.
pub(crate) fn iterator_step(heap: &mut Heap, cursor: HeapRef) -> Result<Option<JValue>, VmError> {
    if !matches!(
        iterator_method(heap, cursor, "hasNext", &[])?,
        Some(JValue::Int(1))
    ) {
        return Ok(None);
    }
    iterator_method(heap, cursor, "next", &[])
}

/// `java.util.Iterator`: `hasNext`/`next`/`remove` over the position the iterator
/// holds. The collection is read live, so a `remove()` (or any other mutation)
/// shows through on the next call — caturra does not model
/// `ConcurrentModificationException`.
#[allow(clippy::too_many_lines)] // one arm per Iterator/ListIterator method
fn iterator_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let Some(HeapObject::Iterator {
        source,
        index,
        last,
        expected_len,
        writes,
        descending,
        ..
    }) = heap.get(receiver)
    else {
        unreachable!("receiver kind checked by caller");
    };
    let (source, index, last, expected_len, writes, descending) =
        (*source, *index, *last, *expected_len, *writes, *descending);
    // A cursor over a read-only view refuses the same mutators the view does —
    // `set` survives on a FIXED-SIZE `Arrays.asList`, whose element write goes
    // through to the array. The JDK reaches this by having `Itr.remove` call
    // the collection's own `remove`; here the capability rides on the cursor.
    let allowed = matches!(
        (method, writes),
        (_, IteratorWrites::All) | ("set", IteratorWrites::FixedSize)
    );
    if !allowed {
        match (method, writes) {
            ("remove", IteratorWrites::ArrayCursor) => {
                return Err(throw("java.lang.UnsupportedOperationException: remove"));
            }
            // A GENERIC cursor checks its OWN state before asking the
            // collection to change: a `remove()` with no `next()` first is an
            // IllegalStateException, and only a well-placed one reaches the
            // collection's refusal. A view's own cursor (`IteratorWrites::None`)
            // has no such order — it throws outright.
            ("remove" | "set", IteratorWrites::FixedSize | IteratorWrites::NoneChecked)
                if last.is_none() =>
            {
                return Err(throw("java.lang.IllegalStateException"));
            }
            ("remove" | "set" | "add", _) => {
                return Err(throw("java.lang.UnsupportedOperationException"));
            }
            _ => {}
        }
    }
    match method {
        // `hasNext` does NOT check for comodification — the JDK's is a bare
        // `cursor != size`. That is not an oversight to fix: it is what makes
        // removing the SECOND-TO-LAST element end a for-each silently instead
        // of throwing, because the shortened size makes `hasNext` false before
        // `next` ever gets to complain. Checking here would "improve" caturra
        // into disagreeing with every real JVM.
        // A DESCENDING cursor walks toward the front, so it is done at 0
        // rather than at the end. The JDK's `hasNext` is a bare cursor
        // comparison either way, which is what lets removing the
        // second-to-last element end a for-each silently rather than throw.
        "hasNext" if descending => Ok(Some(JValue::Int(i32::from(index != 0)))),
        "hasNext" => Ok(Some(JValue::Int(i32::from(if hash_like(heap, source) {
            // A HASH or TREE cursor's `hasNext` is `next != null` — a pointer
            // the LAST `next()` computed, before any later insertion. So
            // adding to a one-entry map while iterating it ends the loop
            // quietly, where the same code over an ArrayList throws: the list
            // cursor's `hasNext` is a bare `cursor != size`, and the longer
            // size makes it true. Reading the CURRENT length for both made
            // caturra throw where a JDK finishes.
            index < expected_len
        } else {
            index != iterated_len(heap, source)
        })))),
        // `next()` on a descending cursor is `previous()`: step back first,
        // then read. The index is the position AFTER the element returned, so
        // `remove()` lands on the one just handed out.
        "next" if descending => {
            check_comodification(heap, source, expected_len)?;
            if index == 0 {
                return Err(throw("java.util.NoSuchElementException"));
            }
            let at = index - 1;
            let element = box_iterated_element(heap, source, at);
            if let Some(HeapObject::Iterator { index, last, .. }) = heap.get_mut(receiver) {
                *index = at;
                *last = Some(at);
            }
            Ok(Some(element))
        }
        "next" => {
            check_comodification(heap, source, expected_len)?;
            if index >= iterated_len(heap, source) {
                return Err(throw("java.util.NoSuchElementException"));
            }
            // An `entrySet()` iterator returns a `Map.Entry` — a reference to the
            // map and the key at this position — resolved live like the entries a
            // for-each yields.
            if let Some(HeapObject::MapView {
                map,
                kind: MapViewKind::Entries,
                ..
            }) = heap.get(source)
            {
                let (map, key) = (*map, map_key_at(heap, *map, index));
                // A cursor that may not write back hands out entries that may
                // not either — `setValue` on one would reach the backing map.
                let entry = heap.alloc(HeapObject::MapEntry {
                    map,
                    key,
                    read_only: writes == IteratorWrites::None,
                });
                if let Some(HeapObject::Iterator { index, last, .. }) = heap.get_mut(receiver) {
                    *last = Some(*index);
                    *index += 1;
                }
                return Ok(Some(JValue::Ref(Some(entry))));
            }
            // A list stores primitives unboxed; box one so `next()` returns the
            // wrapper for every collection alike (a set's element is already a
            // reference). `next()` is typed `BoxedElem`, which expects this.
            let element = box_iterated_element(heap, source, index);
            if let Some(HeapObject::Iterator { index, last, .. }) = heap.get_mut(receiver) {
                *last = Some(*index);
                *index += 1;
            }
            Ok(Some(element))
        }
        // ListIterator: the backward cursor and index queries. `nextIndex` is
        // the cursor; `previousIndex` is one less (-1 at the start).
        "hasPrevious" => Ok(Some(JValue::Int(i32::from(index != 0)))),
        "nextIndex" => Ok(Some(JValue::Int(i32::try_from(index).unwrap_or(i32::MAX)))),
        "previousIndex" => Ok(Some(JValue::Int(
            i32::try_from(index).unwrap_or(i32::MAX) - 1,
        ))),
        "previous" => {
            check_comodification(heap, source, expected_len)?;
            if index == 0 {
                return Err(throw("java.util.NoSuchElementException"));
            }
            let target = index - 1;
            let element = box_iterated_element(heap, source, target);
            if let Some(HeapObject::Iterator { index, last, .. }) = heap.get_mut(receiver) {
                *index = target;
                *last = Some(target);
            }
            Ok(Some(element))
        }
        // `set(e)` overwrites the element the last `next`/`previous` returned.
        "set" => {
            let Some(position) = last else {
                return Err(throw("java.lang.IllegalStateException"));
            };
            check_comodification(heap, source, expected_len)?;
            let value = args.first().copied().unwrap_or(JValue::NULL);
            let (target, at) = match sublist_target(heap, source) {
                Some((backing, from, _)) => (backing, from + position),
                None => (source, position),
            };
            if let Some(values) = heap.list_values_mut(target)
                && let Some(slot) = values.get_mut(at)
            {
                *slot = value;
            }
            // `set` does not change the size, so no re-sync is needed.
            Ok(None)
        }
        // `add(e)` inserts before the cursor; the cursor advances past it, and
        // there is no element to `set`/`remove` afterward.
        "add" => {
            check_comodification(heap, source, expected_len)?;
            let value = args.first().copied().unwrap_or(JValue::NULL);
            match sublist_target(heap, source) {
                Some((backing, from, len)) => {
                    let at = from + index.min(len);
                    if let Some(values) = heap.list_values_mut(backing) {
                        let at = at.min(values.len());
                        values.insert(at, value);
                    }
                    sublist_resize(heap, source, 1);
                }
                None => {
                    if let Some(values) = heap.list_values_mut(source) {
                        let at = index.min(values.len());
                        values.insert(at, value);
                    }
                }
            }
            let len = iterated_len(heap, source);
            if let Some(HeapObject::Iterator {
                index,
                last,
                expected_len,
                ..
            }) = heap.get_mut(receiver)
            {
                *index += 1;
                *last = None;
                *expected_len = len;
            }
            Ok(None)
        }
        "remove" => {
            let Some(position) = last else {
                return Err(throw("java.lang.IllegalStateException"));
            };
            // A stale iterator cannot remove either (the JDK checks here too).
            check_comodification(heap, source, expected_len)?;
            iterated_remove(heap, source, position);
            // The cursor steps back onto the hole so the next element is not
            // skipped, and `remove()` cannot be called twice in a row. This is
            // the ONE legal modification during iteration, so the iterator
            // re-syncs its expectation rather than tripping over itself.
            let len = iterated_len(heap, source);
            if let Some(HeapObject::Iterator {
                index,
                last,
                expected_len,
                ..
            }) = heap.get_mut(receiver)
            {
                *index = position;
                *last = None;
                *expected_len = len;
            }
            Ok(None)
        }
        other => Err(VmError::UnknownIntrinsic(format!("Iterator.{other}"))),
    }
}

#[allow(clippy::too_many_lines)] // one arm per list/queue/deque method
fn list_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let list_len = heap
        .list_values(receiver)
        .map_or_else(|| unreachable!("receiver kind checked by caller"), Vec::len);
    // `Arrays.asList` is a view ON an array, and its `get` indexes that array
    // directly — so an out-of-range index is an ArrayIndexOutOfBoundsException,
    // not the List one. The class is observable in a catch clause.
    let backed_by_array = matches!(heap.get(receiver), Some(HeapObject::ArrayBackedList(_)));
    // The JDK words this TWO ways, and which one a program sees depends on the
    // implementation: `ArrayList` reaches `Objects.checkIndex` and says "Index
    // N out of bounds for length L", while `LinkedList` writes its own
    // "Index: N, Size: L" — and `ArrayList.add(index, …)` uses that second
    // form too, because `rangeCheckForAdd` predates the shared check.
    let linked = matches!(heap.get(receiver), Some(HeapObject::LinkedList(_)));
    let bounds_message = move |index: i32, for_add: bool| -> String {
        if linked || for_add {
            format!("Index: {index}, Size: {list_len}")
        } else {
            format!("Index {index} out of bounds for length {list_len}")
        }
    };
    let check = |index: i32, limit: usize| -> Result<usize, VmError> {
        usize::try_from(index)
            .ok()
            .filter(|i| *i < limit)
            .ok_or_else(|| {
                let class = if backed_by_array {
                    "java.lang.ArrayIndexOutOfBoundsException"
                } else {
                    "java.lang.IndexOutOfBoundsException"
                };
                throw(format!("{class}: {}", bounds_message(index, false)))
            })
    };
    // A `Deque`/`Queue` end operation that must not run on an empty list:
    // `getFirst`/`removeFirst`/`element` throw `NoSuchElementException`, while
    // `peek`/`poll` return `null` — the caller picks by passing the throw flag.
    let end_value = |heap: &mut Heap, from_front: bool, remove: bool, throw_empty: bool| {
        let Some(values) = heap.list_values_mut(receiver) else {
            unreachable!()
        };
        if values.is_empty() {
            return if throw_empty {
                Err(throw("java.util.NoSuchElementException"))
            } else {
                Ok(Some(JValue::NULL))
            };
        }
        let at = if from_front { 0 } else { values.len() - 1 };
        let value = if remove {
            values.remove(at)
        } else {
            values[at]
        };
        Ok(Some(value))
    };
    match (method, descriptor, args) {
        // `iterator()` and `listIterator()` share the cursor object; the extra
        // ListIterator methods (`previous`/`set`/…) act on the same fields.
        // `listIterator(int)` starts the cursor at an index — the standard way
        // to walk a list backwards (`list.listIterator(list.size())`).
        // `descendingIterator()` — the same cursor, started at the END and
        // walking toward the front. A LinkedList's, an ArrayDeque's and a
        // TreeSet's all behave this way, and `remove()` works through it, so
        // the direction is the whole difference.
        ("descendingIterator", _, []) => {
            let expected_len = iterated_len(heap, receiver);
            let iterator = heap.alloc(HeapObject::Iterator {
                source: receiver,
                index: expected_len,
                last: None,
                expected_len,
                writes: IteratorWrites::All,
                list: false,
                descending: true,
            });
            Ok(Some(JValue::Ref(Some(iterator))))
        }
        ("iterator" | "listIterator", _, [] | [JValue::Int(_)]) => {
            let expected_len = iterated_len(heap, receiver);
            let index = match args {
                [JValue::Int(at)] => usize::try_from(*at)
                    .ok()
                    .filter(|at| *at <= expected_len)
                    .ok_or_else(|| {
                        throw(format!(
                            "java.lang.IndexOutOfBoundsException: Index: {at}, Size: {expected_len}"
                        ))
                    })?,
                _ => 0,
            };
            // An `Arrays.asList` view is FIXED-SIZE, not immutable, and the two
            // cursors it hands out differ: `iterator()` is JDK 9's `ArrayItr`
            // (no `remove` at all), while `listIterator()` is `AbstractList`'s,
            // whose `set` writes through to the array.
            let writes = if matches!(heap.get(receiver), Some(HeapObject::ArrayBackedList(_))) {
                if method == "iterator" {
                    IteratorWrites::ArrayCursor
                } else {
                    IteratorWrites::FixedSize
                }
            } else {
                IteratorWrites::All
            };
            let iterator = heap.alloc(HeapObject::Iterator {
                source: receiver,
                index,
                last: None,
                expected_len,
                writes,
                list: method != "iterator",
                descending: false,
            });
            Ok(Some(JValue::Ref(Some(iterator))))
        }
        ("size", _, []) => Ok(Some(JValue::Int(
            i32::try_from(list_len).unwrap_or(i32::MAX),
        ))),
        ("isEmpty", _, []) => Ok(Some(JValue::Int(i32::from(list_len == 0)))),
        // `toArray()` answers an `Object[]`, so a primitive element is boxed
        // on the way out — the same rule `Stream.toArray` follows.
        ("toArray", _, []) => {
            let values: Vec<JValue> = heap
                .list_values(receiver)
                .map_or_else(Vec::new, Clone::clone)
                .into_iter()
                .map(|element| match element {
                    JValue::Ref(_) => element,
                    primitive => JValue::Ref(Some(heap.box_wrapper(
                        match primitive {
                            JValue::Long(_) => "java/lang/Long",
                            JValue::Double(_) => "java/lang/Double",
                            JValue::Float(_) => "java/lang/Float",
                            _ => "java/lang/Integer",
                        },
                        primitive,
                    ))),
                })
                .collect();
            let array = heap.alloc(HeapObject::RefArray(
                String::from("java/lang/Object"),
                values,
            ));
            Ok(Some(JValue::Ref(Some(array))))
        }
        // `add`/`offer`/`addLast`/`offerLast` all append; `addFirst`/`offerFirst`
        // and `push` prepend. The `offer*` forms and `Queue.add` return a
        // boolean; `addFirst`/`addLast`/`push` are void — the descriptor says
        // which, and returning a value for a void method would unbalance the
        // stack.
        (
            "add" | "offer" | "addLast" | "offerLast",
            "(Ljava/lang/Object;)Z" | "(Ljava/lang/Object;)V",
            [value],
        ) => {
            let value = *value;
            if let Some(values) = heap.list_values_mut(receiver) {
                values.push(value);
            }
            Ok(boolean_return_if(descriptor))
        }
        ("addFirst" | "offerFirst" | "push", _, [value]) => {
            let value = *value;
            if let Some(values) = heap.list_values_mut(receiver) {
                values.insert(0, value);
            }
            Ok(boolean_return_if(descriptor))
        }
        ("add", "(ILjava/lang/Object;)V", [JValue::Int(index), value]) => {
            // Insertion allows index == size.
            let at = usize::try_from(*index)
                .ok()
                .filter(|i| *i <= list_len)
                .ok_or_else(|| {
                    throw(format!(
                        "java.lang.IndexOutOfBoundsException: {}",
                        bounds_message(*index, true)
                    ))
                })?;
            let value = *value;
            if let Some(values) = heap.list_values_mut(receiver) {
                values.insert(at, value);
            }
            Ok(None)
        }
        ("get", _, [JValue::Int(index)]) => {
            let at = check(*index, list_len)?;
            Ok(Some(
                heap.list_values(receiver).map_or(JValue::NULL, |v| v[at]),
            ))
        }
        ("set", _, [JValue::Int(index), value]) => {
            let at = check(*index, list_len)?;
            let value = *value;
            let Some(values) = heap.list_values_mut(receiver) else {
                unreachable!()
            };
            let previous = values[at];
            values[at] = value;
            Ok(Some(previous))
        }
        ("remove", _, [JValue::Int(index)]) => {
            let at = check(*index, list_len)?;
            let Some(values) = heap.list_values_mut(receiver) else {
                unreachable!()
            };
            Ok(Some(values.remove(at)))
        }
        // Positional ends. `peek*`/`poll*` are null-on-empty; `getFirst`/
        // `getLast`/`removeFirst`/`removeLast`/`element`/`pop`/`remove()` throw.
        ("peek" | "peekFirst", _, []) => end_value(heap, true, false, false),
        ("peekLast", _, []) => end_value(heap, false, false, false),
        ("poll" | "pollFirst", _, []) => end_value(heap, true, true, false),
        ("pollLast", _, []) => end_value(heap, false, true, false),
        ("getFirst" | "element", _, []) => end_value(heap, true, false, true),
        ("getLast", _, []) => end_value(heap, false, false, true),
        ("removeFirst" | "pop" | "remove", _, []) => end_value(heap, true, true, true),
        ("removeLast", _, []) => end_value(heap, false, true, true),
        ("clear", _, []) => {
            if let Some(values) = heap.list_values_mut(receiver) {
                values.clear();
            }
            Ok(None)
        }
        ("addAll", "(Ljava/util/Collection;)Z", [JValue::Ref(other)]) => {
            let other = other.ok_or_else(|| throw("java.lang.NullPointerException"))?;
            let incoming = match heap.list_values(other) {
                Some(values) => values.clone(),
                _ => return Err(throw("java.lang.ClassCastException: not a list")),
            };
            let changed = !incoming.is_empty();
            if let Some(values) = heap.list_values_mut(receiver) {
                values.extend(incoming);
            }
            Ok(Some(JValue::Int(i32::from(changed))))
        }
        ("addAll", "(ILjava/util/Collection;)Z", [JValue::Int(index), JValue::Ref(other)]) => {
            let other = other.ok_or_else(|| throw("java.lang.NullPointerException"))?;
            let at = usize::try_from(*index)
                .ok()
                .filter(|i| *i <= list_len)
                .ok_or_else(|| {
                    throw(format!(
                        "java.lang.IndexOutOfBoundsException: Index {index} out of bounds \
                         for length {list_len}"
                    ))
                })?;
            let incoming = match heap.list_values(other) {
                Some(values) => values.clone(),
                _ => return Err(throw("java.lang.ClassCastException: not a list")),
            };
            let changed = !incoming.is_empty();
            if let Some(values) = heap.list_values_mut(receiver) {
                for (offset, value) in incoming.into_iter().enumerate() {
                    values.insert(at + offset, value);
                }
            }
            Ok(Some(JValue::Int(i32::from(changed))))
        }
        // Capacity hints: real methods, observable-free here.
        ("ensureCapacity", _, [JValue::Int(_)]) | ("trimToSize", _, []) => Ok(None),
        _ => Err(VmError::UnknownIntrinsic(format!(
            "List.{method}{descriptor}"
        ))),
    }
}

/// `java.io.File` methods over the virtual filesystem.
/// `File.list()` and `File.listFiles()` — the NAMES a directory holds, or the
/// files themselves. Both are `null` when the receiver is not a directory,
/// including when it does not exist, which is the check a program that skips
/// one meets as a `NullPointerException`.
fn file_listing(heap: &mut Heap, vfs: &VirtualFileSystem, path: &str, method: &str) -> JValue {
    let Ok(children) = vfs.list_dir(path) else {
        return JValue::Ref(None);
    };
    let names = children
        .iter()
        .map(|child| child.rsplit('/').next().unwrap_or_default());
    let (elements, descriptor) = if method == "list" {
        (
            names
                .map(|name| JValue::Ref(Some(heap.alloc_string(name))))
                .collect::<Vec<_>>(),
            "[Ljava/lang/String;",
        )
    } else {
        // A listed file is `new File(this, name)` — its path is built from the
        // receiver's own, so a relative listing stays relative and `getPath`
        // prints what a JDK prints.
        let paths: Vec<String> = names
            .map(|name| {
                if path.ends_with('/') {
                    format!("{path}{name}")
                } else {
                    format!("{path}/{name}")
                }
            })
            .collect();
        (
            paths
                .into_iter()
                .map(|child| JValue::Ref(Some(heap.alloc(HeapObject::File(child)))))
                .collect::<Vec<_>>(),
            "[Ljava/io/File;",
        )
    };
    let reference = heap.alloc(HeapObject::RefArray(String::from(descriptor), elements));
    JValue::Ref(Some(reference))
}

/// The `File` argument of `renameTo`/`compareTo`, as its path.
fn file_argument(heap: &Heap, value: Option<&JValue>) -> Option<String> {
    match value {
        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
            Some(HeapObject::File(path)) => Some(path.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// A path's directory part, `None` when it has none. The root has no parent
/// and neither does a bare name, which is what stops a walk up the tree.
fn parent_path(path: &str) -> Option<String> {
    if path == "/" {
        return None;
    }
    match path.rfind('/') {
        None => None,
        Some(0) => Some(String::from("/")),
        Some(cut) => Some(path[..cut].to_owned()),
    }
}

/// A `File`'s ABSTRACT pathname: what the constructor keeps of the string it
/// was given. Repeated separators collapse and a trailing one is dropped, but
/// `.` and `..` are left alone — `new File("./a").getPath()` is "./a", and
/// only the canonical form resolves it. Every `File` in the heap holds this
/// form, so `getPath`, `equals` and `compareTo` agree about what it is.
#[must_use]
pub fn abstract_path(path: &str) -> String {
    let rooted = path.starts_with('/');
    let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return if rooted {
            String::from("/")
        } else {
            String::new()
        };
    }
    let joined = parts.join("/");
    if rooted { format!("/{joined}") } else { joined }
}

/// `UnixFileSystem.resolve`: how `new File(parent, child)` joins the two.
/// A null parent is no parent at all (the child stands alone), an EMPTY one
/// is the root, and an empty child is the parent itself — so
/// `new File("d", "/")` really is "d/", trailing separator and all.
#[must_use]
pub fn resolve_file(parent: Option<&str>, child: &str) -> String {
    let Some(parent) = parent else {
        return child.to_owned();
    };
    let parent = if parent.is_empty() { "/" } else { parent };
    if child.is_empty() {
        return parent.to_owned();
    }
    if parent == "/" {
        return format!("/{}", child.trim_start_matches('/'));
    }
    if child.starts_with('/') {
        return format!("{parent}{child}");
    }
    format!("{parent}/{child}")
}

/// Where a relative path sits: caturra's filesystem is rooted at `/` and has
/// no working directory, so it hangs off the root.
fn absolute_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    }
}

fn file_method(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    receiver: HeapRef,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let path = match heap.get(receiver) {
        Some(HeapObject::File(path)) => path.clone(),
        _ => unreachable!("receiver kind checked by caller"),
    };
    let boolean = |b: bool| Ok(Some(JValue::Int(i32::from(b))));
    match method {
        "exists" => boolean(vfs.exists(&path)),
        "isFile" => boolean(vfs.is_file(&path)),
        "isDirectory" => boolean(vfs.is_directory(&path)),
        "delete" => boolean(vfs.remove(&path).is_ok()),
        // `mkdir` makes ONE directory: a JDK answers false when the parent is
        // missing, and the VFS creates parents implicitly (which is what a
        // program writing a file wants), so the check has to be here.
        "mkdir" => {
            let parent = parent_path(&path);
            let ready = parent.is_none_or(|dir| vfs.is_directory(&dir));
            boolean(ready && !vfs.exists(&path) && vfs.mkdir(&path).is_ok())
        }
        "mkdirs" => boolean(!vfs.exists(&path) && vfs.mkdir(&path).is_ok()),
        "createNewFile" => {
            if vfs.exists(&path) {
                boolean(false)
            } else {
                boolean(vfs.write_file(&path, Vec::new()).is_ok())
            }
        }
        // Java returns long; caturra surfaces int (virtual files are small).
        "length" => Ok(Some(JValue::Int(
            i32::try_from(vfs.len(&path)).unwrap_or(i32::MAX),
        ))),
        // The last component of the ABSTRACT path, not of the canonical one:
        // `new File("a/..").getName()` is "..", which resolving the path first
        // would answer as "".
        "getName" => {
            let name = path.rsplit('/').next().unwrap_or_default();
            let reference = heap.alloc_string(name);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // caturra's filesystem is rooted at `/` and has no working directory,
        // so a relative path's absolute form hangs off the root — the reading
        // `Path.toAbsolutePath` already takes. The canonical form is that with
        // `.` and `..` resolved, which is what `normalize` does.
        "getAbsolutePath" | "getCanonicalPath" => {
            let text = absolute_path(&path);
            let text = if method == "getCanonicalPath" {
                VirtualFileSystem::normalize(&text)
            } else {
                text
            };
            let reference = heap.alloc_string(&text);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        "getAbsoluteFile" | "getCanonicalFile" => {
            let text = absolute_path(&path);
            let text = if method == "getCanonicalFile" {
                VirtualFileSystem::normalize(&text)
            } else {
                text
            };
            Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::File(text))))))
        }
        // `null` when there is no directory part — for a bare name AND for the
        // root itself, which is the pair a walk up the tree terminates on.
        "getParent" => match parent_path(&path) {
            Some(parent) => Ok(Some(JValue::Ref(Some(heap.alloc_string(&parent))))),
            None => Ok(Some(JValue::Ref(None))),
        },
        "getParentFile" => match parent_path(&path) {
            Some(parent) => Ok(Some(JValue::Ref(Some(
                heap.alloc(HeapObject::File(parent)),
            )))),
            None => Ok(Some(JValue::Ref(None))),
        },
        "toPath" => Ok(Some(JValue::Ref(Some(
            heap.alloc(HeapObject::Path(path.clone())),
        )))),
        "isAbsolute" => boolean(path.starts_with('/')),
        // A dotfile, the Unix convention the JDK reads it as.
        "isHidden" => boolean(
            path.rsplit('/')
                .next()
                .is_some_and(|name| name.starts_with('.') && name != "." && name != ".."),
        ),
        // Renaming OVERWRITES an existing destination file on a Unix
        // filesystem rather than failing, which is the surprising half.
        "renameTo" => {
            let Some(target) = file_argument(heap, args.first()) else {
                return boolean(false);
            };
            boolean(vfs.rename(&path, &target).is_ok())
        }
        "compareTo" => {
            let Some(target) = file_argument(heap, args.first()) else {
                return Err(throw("java.lang.NullPointerException"));
            };
            let (mine, theirs): (Vec<u16>, Vec<u16>) = (
                path.encode_utf16().collect(),
                target.encode_utf16().collect(),
            );
            Ok(Some(JValue::Int(compare_utf16(&mine, &theirs))))
        }
        "list" | "listFiles" => Ok(Some(file_listing(heap, vfs, &path, method))),
        // getPath / toString return the path as written.
        "getPath" | "toString" => {
            let reference = heap.alloc_string(&path);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // `UnixFileSystem.hashCode`: the path's hash, xor a constant. A File
        // is VALUE-based (`equals` is handled by the caller, which has the
        // argument), so two Files naming one path hash alike — what lets a
        // program keep them in a Set.
        "hashCode" => Ok(Some(JValue::Int(java_string_hash(&path) ^ 0x0012_d591))),
        _ => Err(VmError::UnknownIntrinsic(format!("File.{method}"))),
    }
}

/// `java.nio.file.Path` methods. `getFileName`/`getParent` return a `Path`;
/// `getParent` is null when the path has no directory part.
/// An `Int`/`Long` summary's `toString`. The JDK writes the average with `%f`
/// (six decimals) and leaves the identity min/max of an empty summary showing.
/// Shared with the collection renderer: written twice, a summary inside a map
/// printed as `object@2a` while the same object printed directly was right.
#[must_use]
pub(crate) fn summary_text(
    count: i64,
    sum: i64,
    min: i64,
    max: i64,
    kind: crate::value::SummaryKind,
) -> String {
    #[allow(clippy::cast_precision_loss)] // the JDK divides in double too
    let average = if count == 0 {
        0.0
    } else {
        sum as f64 / count as f64
    };
    let name = if kind == crate::value::SummaryKind::Long {
        "LongSummaryStatistics"
    } else {
        "IntSummaryStatistics"
    };
    format!("{name}{{count={count}, sum={sum}, min={min}, average={average:.6}, max={max}}}")
}

/// A `DoubleSummaryStatistics`'s `toString`: every number with `%f`, including
/// the `Infinity`/`-Infinity` an empty summary keeps, which `%f` spells out.
#[must_use]
pub(crate) fn double_summary_text(count: i64, sum: f64, min: f64, max: f64) -> String {
    #[allow(clippy::cast_precision_loss)]
    let average = if count == 0 { 0.0 } else { sum / count as f64 };
    let show = |value: f64| {
        if value.is_finite() {
            format!("{value:.6}")
        } else if value.is_nan() {
            String::from("NaN")
        } else if value > 0.0 {
            String::from("Infinity")
        } else {
            String::from("-Infinity")
        }
    };
    format!(
        "DoubleSummaryStatistics{{count={count}, sum={}, min={}, average={}, max={}}}",
        show(sum),
        show(min),
        show(average),
        show(max)
    )
}

/// What a `Collector` prints: `Object`'s shape, with the class a JDK really
/// builds. Written once, so the direct call and a collection's rendering agree.
fn collector_text(reference: HeapRef) -> String {
    format!(
        "java.util.stream.Collectors$CollectorImpl@{:x}",
        identity_hash(reference)
    )
}

/// The charsets a program names, in the JDK's canonical spelling. `None` for a
/// name no JDK would accept either, which is what makes the failure honest.
#[must_use]
pub fn canonical_charset(name: &str) -> Option<&'static str> {
    let folded = name.to_ascii_uppercase();
    Some(match folded.as_str() {
        "UTF-8" | "UTF8" | "UNICODE-1-1-UTF-8" => "UTF-8",
        "US-ASCII" | "ASCII" | "ANSI_X3.4-1968" | "ISO646-US" => "US-ASCII",
        "ISO-8859-1" | "ISO8859-1" | "LATIN1" | "ISO_8859-1" | "L1" | "8859_1" => "ISO-8859-1",
        "UTF-16" | "UTF16" => "UTF-16",
        "UTF-16BE" | "UTF16BE" | "UNICODEBIGUNMARKED" => "UTF-16BE",
        "UTF-16LE" | "UTF16LE" | "UNICODELITTLEUNMARKED" => "UTF-16LE",
        _ => return None,
    })
}

/// Encode UTF-16 units the way `String.getBytes(charset)` does. An unmappable
/// character is `?` in the byte charsets, exactly as the JDK's encoder
/// replaces it; `UTF-16` writes the big-endian BOM first, which is the one
/// place the three UTF-16 spellings differ.
#[must_use]
pub fn encode_charset(units: &[u16], charset: &str) -> Vec<i8> {
    let mut bytes: Vec<i8> = Vec::new();
    match charset {
        "US-ASCII" => {
            for unit in units {
                bytes.push(if *unit < 0x80 {
                    u8::try_from(*unit).unwrap_or(b'?').cast_signed()
                } else {
                    b'?'.cast_signed()
                });
            }
        }
        "ISO-8859-1" => {
            for unit in units {
                bytes.push(if *unit < 0x100 {
                    u8::try_from(*unit).unwrap_or(b'?').cast_signed()
                } else {
                    b'?'.cast_signed()
                });
            }
        }
        "UTF-16" | "UTF-16BE" | "UTF-16LE" => {
            let little = charset == "UTF-16LE";
            if charset == "UTF-16" {
                bytes.push(0xFEu8.cast_signed());
                bytes.push(0xFFu8.cast_signed());
            }
            for unit in units {
                let (high, low) = ((unit >> 8) as u8, (unit & 0xFF) as u8);
                if little {
                    bytes.push(low.cast_signed());
                    bytes.push(high.cast_signed());
                } else {
                    bytes.push(high.cast_signed());
                    bytes.push(low.cast_signed());
                }
            }
        }
        // UTF-8, and the default. A lone surrogate is unmappable, and the
        // JDK's encoder writes `?` for it.
        _ => {
            for decoded in char::decode_utf16(units.iter().copied()) {
                match decoded {
                    Ok(character) => {
                        let mut buffer = [0u8; 4];
                        for byte in character.encode_utf8(&mut buffer).as_bytes() {
                            bytes.push(byte.cast_signed());
                        }
                    }
                    Err(_) => bytes.push(b'?'.cast_signed()),
                }
            }
        }
    }
    bytes
}

/// Decode bytes the way `new String(bytes, charset)` does. Malformed input is
/// the replacement character U+FFFD, as the JDK's decoder gives — never an
/// exception, which is the part a program relies on.
#[must_use]
pub fn decode_charset(bytes: &[i8], charset: &str) -> Vec<u16> {
    let raw: Vec<u8> = bytes.iter().map(|b| b.cast_unsigned()).collect();
    match charset {
        "US-ASCII" => raw
            .iter()
            .map(|b| if *b < 0x80 { u16::from(*b) } else { 0xFFFD })
            .collect(),
        "ISO-8859-1" => raw.iter().map(|b| u16::from(*b)).collect(),
        "UTF-16" | "UTF-16BE" | "UTF-16LE" => {
            let mut little = charset == "UTF-16LE";
            let mut at = 0;
            // `UTF-16` reads the BOM when there is one, and is big-endian
            // without it.
            if charset == "UTF-16" && raw.len() >= 2 {
                match (raw[0], raw[1]) {
                    (0xFE, 0xFF) => at = 2,
                    (0xFF, 0xFE) => {
                        little = true;
                        at = 2;
                    }
                    _ => {}
                }
            }
            let mut units = Vec::new();
            while at + 1 < raw.len() {
                let (a, b) = (u16::from(raw[at]), u16::from(raw[at + 1]));
                units.push(if little { (b << 8) | a } else { (a << 8) | b });
                at += 2;
            }
            if at < raw.len() {
                units.push(0xFFFD);
            }
            units
        }
        _ => {
            let mut text = String::with_capacity(raw.len());
            let mut rest = &raw[..];
            loop {
                match std::str::from_utf8(rest) {
                    Ok(valid) => {
                        text.push_str(valid);
                        break;
                    }
                    Err(error) => {
                        let (valid, remainder) = rest.split_at(error.valid_up_to());
                        text.push_str(std::str::from_utf8(valid).unwrap_or(""));
                        text.push('\u{FFFD}');
                        match error.error_len() {
                            Some(skipped) => rest = &remainder[skipped..],
                            None => break,
                        }
                    }
                }
            }
            text.encode_utf16().collect()
        }
    }
}

/// A `java.util.regex.Matcher`'s own methods — the walk a program writes as
/// `while (m.find()) { … m.group(1) … }`.
///
/// A matcher REMEMBERS its last match: `group`, `start` and `end` all read it,
/// and asking before there is one is Java's `IllegalStateException`, not a
/// silent answer.
/// A `java.util.regex.Matcher`'s own methods — the walk a program writes as
/// `while (m.find()) { … m.group(1) … }`, and everything around it: the region
/// it is confined to, how that region's edges behave, the append/tail
/// rewriting loop, and what the last attempt learned.
///
/// A matcher REMEMBERS its last match: `group`, `start` and `end` all read it,
/// and asking before there is one is Java's `IllegalStateException`, not a
/// silent answer.
#[allow(clippy::too_many_lines)] // one arm per Matcher method
fn matcher_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let state = matcher_state(heap, receiver)?;
    let source = match heap.get(state.pattern) {
        Some(HeapObject::Pattern { source, flags }) => fold_regex_flags(source, *flags),
        _ => return Err(throw("java.lang.IllegalStateException: no pattern")),
    };
    let regex = compile_regex(&source)?;
    let bounds = state.bounds();
    let input = state.input.clone();
    let no_match = || throw("java.lang.IllegalStateException: No match found");
    let spans = state.last.clone();
    let group_span = |index: usize| -> Result<Option<(usize, usize)>, VmError> {
        let spans = spans.clone().ok_or_else(no_match)?;
        spans.get(index).copied().ok_or_else(|| {
            throw(format!(
                "java.lang.IndexOutOfBoundsException: No group {index}"
            ))
        })
    };
    let group_index = |args: &[JValue]| -> Result<usize, VmError> {
        match args.first() {
            Some(JValue::Int(index)) => usize::try_from(*index).map_err(|_| {
                throw(format!(
                    "java.lang.IndexOutOfBoundsException: No group {index}"
                ))
            }),
            // `group("name")` — the group a `(?<name>…)` stands for. Whether
            // there IS a match is asked FIRST, as a JDK asks it: naming a
            // group that does not exist on a matcher that has not matched is
            // "No match found", not "No group with name".
            Some(JValue::Ref(Some(name))) => {
                if spans.is_none() {
                    return Err(no_match());
                }
                let name = heap.string_text(*name).unwrap_or_default();
                regex.group_named(&name).ok_or_else(|| {
                    throw(format!(
                        "java.lang.IllegalArgumentException: No group with name <{name}>"
                    ))
                })
            }
            // No argument at all is the WHOLE match, which is group zero.
            _ => Ok(0),
        }
    };
    match method {
        // `find()` resumes where the last match ended; a zero-width match
        // advances one unit, or the loop would never end. `find(at)` RESETS
        // the matcher and starts there, as a JDK's does.
        "find" => {
            let from = match args.first() {
                Some(JValue::Int(start)) => {
                    let start = usize::try_from(*start).map_err(|_| {
                        throw("java.lang.IndexOutOfBoundsException: Illegal start index")
                    })?;
                    if start > input.len() {
                        return Err(throw(
                            "java.lang.IndexOutOfBoundsException: Illegal start index",
                        ));
                    }
                    start
                }
                _ => state.at.max(bounds.start),
            };
            if from > bounds.end {
                store_match(heap, receiver, bounds.end + 1, None, true, false);
                return Ok(Some(JValue::Int(0)));
            }
            let attempt = regex.find_in(&input, from, bounds);
            let Some(found) = attempt.matched else {
                store_match(
                    heap,
                    receiver,
                    bounds.end + 1,
                    None,
                    attempt.hit_end,
                    attempt.require_end,
                );
                return Ok(Some(JValue::Int(0)));
            };
            let next = if found.end == found.start {
                found.end + 1
            } else {
                found.end
            };
            store_match(
                heap,
                receiver,
                next,
                Some(found.groups.clone()),
                attempt.hit_end,
                attempt.require_end,
            );
            Ok(Some(JValue::Int(1)))
        }
        // `matches()` must fill the REGION; `lookingAt()` only anchors its
        // start.
        "matches" | "lookingAt" => {
            let attempt = if method == "matches" {
                regex.matches_in(&input, bounds)
            } else {
                regex.looking_at(&input, bounds)
            };
            let matched = attempt.matched.is_some();
            store_match(
                heap,
                receiver,
                state.at,
                attempt.matched.map(|found| found.groups),
                attempt.hit_end,
                attempt.require_end,
            );
            Ok(Some(JValue::Int(i32::from(matched))))
        }
        "group" => {
            let index = group_index(args)?;
            match group_span(index)? {
                Some((start, end)) => Ok(Some(JValue::Ref(Some(
                    heap.alloc_string_units(&input[start..end]),
                )))),
                // A group that took part in no match is `null`, not "".
                None => Ok(Some(JValue::NULL)),
            }
        }
        "start" | "end" => {
            let index = group_index(args)?;
            let Some((start, end)) = group_span(index)? else {
                return Ok(Some(JValue::Int(-1)));
            };
            let value = if method == "start" { start } else { end };
            Ok(Some(JValue::Int(i32::try_from(value).unwrap_or(-1))))
        }
        "groupCount" => Ok(Some(JValue::Int(
            i32::try_from(regex.group_count()).unwrap_or(0),
        ))),
        // What the last attempt learned: whether it ran out of input, and
        // whether its answer leaned on the end.
        "hitEnd" => Ok(Some(JValue::Int(i32::from(state.hit_end)))),
        "requireEnd" => Ok(Some(JValue::Int(i32::from(state.require_end)))),
        "pattern" => Ok(Some(JValue::Ref(Some(state.pattern)))),
        // `usePattern` keeps the position and drops the match, as a JDK's does.
        "usePattern" => {
            let Some(JValue::Ref(Some(pattern))) = args.first() else {
                return Err(throw(
                    "java.lang.IllegalArgumentException: Pattern cannot be null",
                ));
            };
            if !matches!(heap.get(*pattern), Some(HeapObject::Pattern { .. })) {
                return Err(throw("java.lang.ClassCastException: not a Pattern"));
            }
            if let Some(HeapObject::Matcher {
                pattern: slot,
                last,
                ..
            }) = heap.get_mut(receiver)
            {
                *slot = *pattern;
                *last = None;
            }
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        // The REGION, and the two ways its edges can behave.
        "region" => {
            let (from, to) = match (args.first(), args.get(1)) {
                (Some(JValue::Int(from)), Some(JValue::Int(to))) => (*from, *to),
                _ => return Err(throw("java.lang.VerifyError: expected two ints")),
            };
            // The JDK's three messages, in its order: the bare word for an
            // edge outside the input, "start > end" for an inverted region.
            let len = i32::try_from(input.len()).unwrap_or(i32::MAX);
            if from < 0 || from > len {
                return Err(throw("java.lang.IndexOutOfBoundsException: start"));
            }
            if to < 0 || to > len {
                return Err(throw("java.lang.IndexOutOfBoundsException: end"));
            }
            if from > to {
                return Err(throw("java.lang.IndexOutOfBoundsException: start > end"));
            }
            let (from, to) = (
                usize::try_from(from).unwrap_or(0),
                usize::try_from(to).unwrap_or(0),
            );
            if let Some(HeapObject::Matcher {
                region,
                at,
                last,
                appended,
                ..
            }) = heap.get_mut(receiver)
            {
                *region = (from, to);
                *at = from;
                *last = None;
                *appended = from;
            }
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        "regionStart" => Ok(Some(JValue::Int(
            i32::try_from(state.region.0).unwrap_or(0),
        ))),
        "regionEnd" => Ok(Some(JValue::Int(
            i32::try_from(state.region.1).unwrap_or(0),
        ))),
        "hasAnchoringBounds" => Ok(Some(JValue::Int(i32::from(state.anchoring)))),
        "hasTransparentBounds" => Ok(Some(JValue::Int(i32::from(state.transparent)))),
        "useAnchoringBounds" | "useTransparentBounds" => {
            let on = matches!(args.first(), Some(JValue::Int(value)) if *value != 0);
            if let Some(HeapObject::Matcher {
                anchoring,
                transparent,
                ..
            }) = heap.get_mut(receiver)
            {
                if method == "useAnchoringBounds" {
                    *anchoring = on;
                } else {
                    *transparent = on;
                }
            }
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        "reset" => {
            // `reset(text)` swaps the input too; both forms put the matcher
            // back at the beginning, over the WHOLE input.
            if let Some(value) = args.first() {
                let text = string_units(heap, value)?;
                if let Some(HeapObject::Matcher { input: slot, .. }) = heap.get_mut(receiver) {
                    *slot = text;
                }
            }
            matcher_reset(heap, receiver);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        // The string replacements are the JDK's own loop: `reset()`, then
        // find/appendReplacement until there is nothing left (or once), then
        // `appendTail`. Writing them as a one-shot rewrite instead left the
        // matcher at the BEGINNING, where a JDK leaves it wherever the scan
        // stopped: after `replaceFirst` the next `find` answers the SECOND
        // match, and after `replaceAll` it answers nothing at all.
        "replaceAll" | "replaceFirst" => {
            let replacement = string_units(heap, args.first().unwrap_or(&JValue::NULL))?;
            // `reset()` restores the whole input as the region, and the two
            // bound modes survive it.
            let mut full = crate::regex::Bounds {
                start: 0,
                end: input.len(),
                anchoring: state.anchoring,
                transparent: state.transparent,
                since: None,
            };
            let mut out: Vec<u16> = Vec::new();
            let (mut at, mut appended) = (0usize, 0usize);
            let mut last: Option<Vec<Option<(usize, usize)>>>;
            let (mut hit, mut require);
            loop {
                let attempt = regex.find_in(&input, at, full);
                hit = attempt.hit_end;
                require = attempt.require_end;
                let Some(one) = attempt.matched else {
                    at = full.end + 1;
                    last = None;
                    break;
                };
                out.extend_from_slice(&input[appended..one.start]);
                out.extend(expand_replacement(&input, &one, &replacement, &regex)?);
                appended = one.end;
                at = if one.end == one.start {
                    one.end + 1
                } else {
                    one.end
                };
                full.since = Some(one.end);
                last = Some(one.groups);
                if method == "replaceFirst" {
                    break;
                }
            }
            out.extend_from_slice(&input[appended.min(input.len())..]);
            if let Some(HeapObject::Matcher {
                region,
                at: slot,
                last: last_slot,
                appended: appended_slot,
                hit_end,
                require_end,
                ..
            }) = heap.get_mut(receiver)
            {
                *region = (0, input.len());
                *slot = at;
                *last_slot = last;
                *appended_slot = appended;
                *hit_end = hit;
                *require_end = require;
            }
            let text = String::from_utf16_lossy(&out);
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        // `appendReplacement(sb, replacement)` copies the text since the last
        // append, then the expanded replacement — the loop a program writes
        // when it rewrites some matches and keeps others.
        "appendReplacement" => {
            let Some(JValue::Ref(Some(builder))) = args.first() else {
                return Err(throw("java.lang.NullPointerException"));
            };
            let builder = *builder;
            let replacement = string_units(heap, args.get(1).unwrap_or(&JValue::NULL))?;
            // A JDK words THIS one differently from `group`'s: the append
            // loop reports "No match available".
            let unavailable = || throw("java.lang.IllegalStateException: No match available");
            let groups = spans.clone().ok_or_else(unavailable)?;
            let Some((start, end)) = groups.first().copied().flatten() else {
                return Err(unavailable());
            };
            let mut text: Vec<u16> = input[state.appended..start].to_vec();
            let one = crate::regex::Match {
                start,
                end,
                groups: groups.clone(),
            };
            text.extend(expand_replacement(&input, &one, &replacement, &regex)?);
            append_units(heap, builder, &text)?;
            if let Some(HeapObject::Matcher { appended, .. }) = heap.get_mut(receiver) {
                *appended = end;
            }
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        "appendTail" => {
            let Some(JValue::Ref(Some(builder))) = args.first() else {
                return Err(throw("java.lang.NullPointerException"));
            };
            let builder = *builder;
            let rest = input[state.appended.min(input.len())..].to_vec();
            append_units(heap, builder, &rest)?;
            Ok(Some(JValue::Ref(Some(builder))))
        }
        // A frozen copy of the current match, which outlives the next `find`.
        "toMatchResult" => {
            let groups = spans.clone().ok_or_else(no_match)?;
            Ok(Some(JValue::Ref(Some(heap.alloc(
                HeapObject::MatchResult {
                    input: input.clone(),
                    groups,
                },
            )))))
        }
        // `results()` — the matches still ahead, as a LAZY stream over this
        // matcher: each element is one `find`, so the pipeline consumes only
        // what it asks for and the matcher is left where it stopped.
        "results" => Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::Stream {
            source: crate::value::StreamSource::Matches { matcher: receiver },
            ops: Vec::new(),
        }))))),
        // A JDK's `Matcher.toString` prints its own state — pattern, region
        // and last match — and a program that prints a matcher sees it,
        // whether it prints one directly or a collection of them.
        "toString" => {
            let text = matcher_text(heap, receiver).unwrap_or_default();
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Matcher.{method}"))),
    }
}

/// A `MatchResult`'s four questions, answered from a frozen match.
fn match_result_method(
    heap: &mut Heap,
    receiver: HeapRef,
    input: &[u16],
    groups: &[Option<(usize, usize)>],
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let index = match args.first() {
        Some(JValue::Int(index)) => usize::try_from(*index).map_err(|_| {
            throw(format!(
                "java.lang.IndexOutOfBoundsException: No group {index}"
            ))
        })?,
        _ => 0,
    };
    let span = || -> Result<Option<(usize, usize)>, VmError> {
        groups.get(index).copied().ok_or_else(|| {
            throw(format!(
                "java.lang.IndexOutOfBoundsException: No group {index}"
            ))
        })
    };
    match method {
        "group" => match span()? {
            Some((start, end)) => Ok(Some(JValue::Ref(Some(
                heap.alloc_string_units(&input[start..end]),
            )))),
            None => Ok(Some(JValue::NULL)),
        },
        "start" | "end" => {
            let Some((start, end)) = span()? else {
                return Ok(Some(JValue::Int(-1)));
            };
            let value = if method == "start" { start } else { end };
            Ok(Some(JValue::Int(i32::try_from(value).unwrap_or(-1))))
        }
        "groupCount" => Ok(Some(JValue::Int(
            i32::try_from(groups.len().saturating_sub(1)).unwrap_or(0),
        ))),
        // A frozen match is an inner class of Matcher in a JDK and overrides
        // nothing, so printing one shows that class and an identity hash —
        // not the text it matched.
        "toString" => {
            let text = format!(
                "java.util.regex.Matcher$ImmutableMatchResult@{:x}",
                identity_hash(receiver)
            );
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("MatchResult.{method}"))),
    }
}

/// What a `Pattern`'s predicate answers for one input: `asPredicate` looks for
/// the pattern anywhere, `asMatchPredicate` demands the whole string.
pub(crate) fn regex_predicate(
    heap: &Heap,
    pattern: HeapRef,
    whole: bool,
    text: &[u16],
) -> Result<bool, VmError> {
    let source = match heap.get(pattern) {
        Some(HeapObject::Pattern { source, flags }) => fold_regex_flags(source, *flags),
        _ => return Err(throw("java.lang.IllegalStateException: no pattern")),
    };
    let regex = compile_regex(&source)?;
    Ok(if whole {
        regex.matches_whole(text)
    } else {
        regex.find_at(text, 0).is_some()
    })
}

/// Put a matcher back where `reset()` puts it: at the beginning, with the
/// whole input as its region and no match remembered.
pub(crate) fn matcher_reset(heap: &mut Heap, receiver: HeapRef) {
    if let Some(HeapObject::Matcher {
        input,
        at,
        last,
        region,
        appended,
        hit_end,
        require_end,
        ..
    }) = heap.get_mut(receiver)
    {
        *region = (0, input.len());
        *at = 0;
        *last = None;
        *appended = 0;
        *hit_end = false;
        *require_end = false;
    }
}

/// What a `Matcher` prints: its pattern, its region and its last match, in the
/// JDK's own layout. Shared by `toString` and by the renderer a COLLECTION of
/// matchers goes through — printed one way and not the other, the two drifted.
pub(crate) fn matcher_text(heap: &Heap, receiver: HeapRef) -> Option<String> {
    let state = matcher_state(heap, receiver).ok()?;
    let source = match heap.get(state.pattern) {
        Some(HeapObject::Pattern { source, flags }) => fold_regex_flags(source, *flags),
        _ => return None,
    };
    let matched = match state
        .last
        .and_then(|groups| groups.first().copied().flatten())
    {
        Some((start, end)) => String::from_utf16_lossy(&state.input[start..end]),
        None => String::new(),
    };
    Some(format!(
        "java.util.regex.Matcher[pattern={} region={},{} lastmatch={matched}]",
        String::from_utf16_lossy(&source),
        state.region.0,
        state.region.1,
    ))
}

/// A matcher's own state, read out in one go.
#[allow(clippy::struct_excessive_bools)] // a JDK's Matcher has exactly these switches
struct MatcherState {
    pattern: HeapRef,
    input: Vec<u16>,
    at: usize,
    last: Option<Vec<Option<(usize, usize)>>>,
    region: (usize, usize),
    anchoring: bool,
    transparent: bool,
    hit_end: bool,
    require_end: bool,
    appended: usize,
}

impl MatcherState {
    fn bounds(&self) -> crate::regex::Bounds {
        crate::regex::Bounds {
            start: self.region.0,
            end: self.region.1,
            anchoring: self.anchoring,
            transparent: self.transparent,
            // `\G` matches where the PREVIOUS match ended, which is not
            // always where the next search begins: after an empty match the
            // search moves on by one and `\G` does not.
            since: self
                .last
                .as_ref()
                .and_then(|groups| groups.first().copied().flatten())
                .map(|(_, end)| end),
        }
    }
}

fn matcher_state(heap: &Heap, receiver: HeapRef) -> Result<MatcherState, VmError> {
    match heap.get(receiver) {
        Some(HeapObject::Matcher {
            pattern,
            input,
            at,
            last,
            region,
            anchoring,
            transparent,
            hit_end,
            require_end,
            appended,
        }) => Ok(MatcherState {
            pattern: *pattern,
            input: input.clone(),
            at: *at,
            last: last.clone(),
            region: *region,
            anchoring: *anchoring,
            transparent: *transparent,
            hit_end: *hit_end,
            require_end: *require_end,
            appended: *appended,
        }),
        _ => Err(throw("java.lang.ClassCastException: not a Matcher")),
    }
}

/// Record what an attempt found and where the next one starts.
fn store_match(
    heap: &mut Heap,
    receiver: HeapRef,
    next: usize,
    spans: Option<Vec<Option<(usize, usize)>>>,
    hit: bool,
    require: bool,
) {
    if let Some(HeapObject::Matcher {
        at,
        last,
        hit_end,
        require_end,
        ..
    }) = heap.get_mut(receiver)
    {
        *at = next;
        *last = spans;
        *hit_end = hit;
        *require_end = require;
    }
}

/// Append units to a `StringBuilder` (the `StringBuffer` a JDK takes is the
/// same object here).
fn append_units(heap: &mut Heap, builder: HeapRef, units: &[u16]) -> Result<(), VmError> {
    match heap.get_mut(builder) {
        Some(HeapObject::StringBuilder(text)) => {
            text.extend_from_slice(units);
            Ok(())
        }
        _ => Err(throw("java.lang.ClassCastException: not a StringBuilder")),
    }
}

/// The UTF-16 units of a `String` argument (null throws NPE).
fn string_units(heap: &Heap, value: &JValue) -> Result<Vec<u16>, VmError> {
    match value {
        JValue::Ref(Some(reference)) => match heap.get(*reference) {
            // A `CharSequence` is a String or a builder, and both hold the
            // same units.
            Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                Ok(units.clone())
            }
            _ => Err(throw("java.lang.ClassCastException: not a CharSequence")),
        },
        _ => Err(throw("java.lang.NullPointerException")),
    }
}

/// `Pattern.compile(regex, flags)` — the flags a JDK gives as an int, folded
/// into the inline `(?ims)` prefix the engine reads. The four the engine
/// models are the four a program writes; anything else is ignored, as a flag
/// that changes nothing may be.
/// The flags whose EFFECT this engine models. `CANON_EQ` (canonical
/// equivalence) and `UNICODE_CHARACTER_CLASS` (Unicode-aware `\w`, `\d` and
/// `\b`) change what matches for ordinary text, and accepting them silently
/// would answer a different question from the one the program asked — so
/// `compile` says so instead, where a JDK would have compiled.
fn check_regex_flags(flags: i32) -> Result<(), VmError> {
    for (bit, name) in [(0x80, "CANON_EQ"), (0x100, "UNICODE_CHARACTER_CLASS")] {
        if flags & bit != 0 {
            return Err(VmError::UnknownIntrinsic(format!(
                "Pattern.compile with Pattern.{name}"
            )));
        }
    }
    Ok(())
}

fn fold_regex_flags(source: &[u16], flags: i32) -> Vec<u16> {
    // `LITERAL` says the pattern is TEXT, not a pattern — the same thing
    // `Pattern.quote` produces, which the engine already reads.
    if flags & 0x10 != 0 {
        let mut out: Vec<u16> = "\\Q".encode_utf16().collect();
        out.extend_from_slice(source);
        out.extend("\\E".encode_utf16());
        return out;
    }
    let mut inline = String::new();
    for (bit, letter) in [(0x02, 'i'), (0x08, 'm'), (0x20, 's'), (0x04, 'x')] {
        if flags & bit != 0 {
            inline.push(letter);
        }
    }
    if inline.is_empty() {
        return source.to_vec();
    }
    let mut out: Vec<u16> = format!("(?{inline})").encode_utf16().collect();
    out.extend_from_slice(source);
    out
}

/// The `byte[]` an argument names, as signed bytes.
fn byte_array_values(heap: &Heap, value: &JValue) -> Result<Vec<i8>, VmError> {
    match value {
        JValue::Ref(Some(reference)) => match heap.get(*reference) {
            Some(HeapObject::ByteArray(bytes)) => Ok(bytes.clone()),
            _ => Err(throw("java.lang.ClassCastException: not a byte[]")),
        },
        _ => Err(throw("java.lang.NullPointerException")),
    }
}

/// The charset an optional argument names — a `Charset` object or its NAME —
/// defaulting to UTF-8, which is the default charset here as it is on any
/// modern JDK.
fn charset_argument(heap: &Heap, value: Option<&JValue>) -> Result<String, VmError> {
    let Some(JValue::Ref(Some(reference))) = value else {
        return Ok(String::from("UTF-8"));
    };
    match heap.get(*reference) {
        Some(HeapObject::Charset(name)) => Ok(name.clone()),
        Some(HeapObject::JavaString(units)) => {
            let written = String::from_utf16_lossy(units);
            canonical_charset(&written)
                .map(ToOwned::to_owned)
                .ok_or_else(|| throw(format!("java.io.UnsupportedEncodingException: {written}")))
        }
        _ => Err(throw("java.lang.NullPointerException")),
    }
}

/// The NAME ELEMENTS of a path — what `getNameCount` counts and `getName`
/// indexes. A leading `/` is the ROOT, not an element, and an empty path has
/// one element (the empty name), which is what a JDK's `Path.of("")` reports.
fn path_names(path: &str) -> Vec<&str> {
    let body = path.strip_prefix('/').unwrap_or(path);
    if body.is_empty() {
        // An absolute path with nothing after the root has NO name elements;
        // a relative empty one has a single empty name.
        return if path.starts_with('/') {
            Vec::new()
        } else {
            vec![""]
        };
    }
    body.split('/').collect()
}

/// A path built from name elements, keeping the root if there was one.
fn path_from(absolute: bool, names: &[&str]) -> String {
    let joined = names.join("/");
    if absolute {
        format!("/{joined}")
    } else {
        joined
    }
}

#[allow(clippy::too_many_lines)] // one arm per Path method
fn path_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let path = match heap.get(receiver) {
        Some(HeapObject::Path(path)) => path.clone(),
        _ => unreachable!("receiver kind checked by caller"),
    };
    let absolute = path.starts_with('/');
    let names = path_names(&path);
    // A `Path` argument, or a `String` that names one — every method that
    // takes a path takes both, and a JDK converts the string the same way
    // `Path.of` would.
    let other = |heap: &Heap, value: Option<&JValue>| -> Result<String, VmError> {
        match value {
            Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                Some(HeapObject::Path(text)) => Ok(text.clone()),
                Some(HeapObject::JavaString(units)) => Ok(String::from_utf16_lossy(units)),
                _ => Err(throw("java.lang.ClassCastException: not a Path")),
            },
            _ => Err(throw("java.lang.NullPointerException")),
        }
    };
    let path_value = |heap: &mut Heap, text: String| {
        Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::Path(text))))))
    };
    let index = |args: &[JValue], at: usize| -> i32 {
        match args.get(at) {
            Some(JValue::Int(value)) => *value,
            _ => 0,
        }
    };
    match method {
        "toString" => Ok(Some(JValue::Ref(Some(heap.alloc_string(&path))))),
        "getFileName" => match names.last() {
            Some(last) => path_value(heap, (*last).to_owned()),
            None => Ok(Some(JValue::NULL)),
        },
        "getParent" => {
            if names.len() < 2 {
                // A single name has no parent unless there is a root above it.
                return if absolute && names.len() == 1 {
                    path_value(heap, String::from("/"))
                } else {
                    Ok(Some(JValue::NULL))
                };
            }
            path_value(heap, path_from(absolute, &names[..names.len() - 1]))
        }
        "getRoot" => {
            if absolute {
                path_value(heap, String::from("/"))
            } else {
                Ok(Some(JValue::NULL))
            }
        }
        "getNameCount" => Ok(Some(JValue::Int(i32::try_from(names.len()).unwrap_or(0)))),
        "getName" => {
            let at = index(args, 0);
            let Some(name) = usize::try_from(at).ok().and_then(|at| names.get(at)) else {
                return Err(throw(format!(
                    "java.lang.IllegalArgumentException: Invalid index: {at}"
                )));
            };
            path_value(heap, (*name).to_owned())
        }
        "isAbsolute" => Ok(Some(JValue::Int(i32::from(absolute)))),
        // `.` drops out and `..` cancels the name before it — but only when
        // there IS one to cancel, and never past a root.
        "normalize" => {
            let mut kept: Vec<&str> = Vec::new();
            for name in &names {
                match *name {
                    "." => {}
                    ".." => {
                        if matches!(kept.last(), Some(last) if *last != "..") {
                            kept.pop();
                        } else if !absolute {
                            kept.push("..");
                        }
                    }
                    other => kept.push(other),
                }
            }
            path_value(heap, path_from(absolute, &kept))
        }
        // `resolve` against an ABSOLUTE (or empty) argument answers the
        // argument itself, which is the JDK's rule.
        "resolve" | "resolveSibling" => {
            let other = other(heap, args.first())?;
            let base = if method == "resolve" {
                path.clone()
            } else if names.len() > 1 || absolute {
                path_from(absolute, &names[..names.len().saturating_sub(1)])
            } else {
                String::new()
            };
            if other.starts_with('/') || base.is_empty() {
                return path_value(heap, other);
            }
            if other.is_empty() {
                return path_value(heap, base);
            }
            path_value(heap, format!("{}/{other}", base.trim_end_matches('/')))
        }
        // `a/b`.relativize(`a/b/c`) is `c` — the walk from one to the other.
        "relativize" => {
            let other_text = other(heap, args.first())?;
            let other_absolute = other_text.starts_with('/');
            if other_absolute != absolute {
                return Err(throw(
                    "java.lang.IllegalArgumentException: 'other' is different type of Path",
                ));
            }
            let other_names = path_names(&other_text);
            let shared = names
                .iter()
                .zip(other_names.iter())
                .take_while(|(a, b)| a == b)
                .count();
            let mut walk: Vec<&str> = vec![".."; names.len() - shared];
            walk.extend(other_names[shared..].iter().copied());
            path_value(heap, walk.join("/"))
        }
        // `startsWith`/`endsWith` compare whole NAME ELEMENTS: `a/bc` does not
        // start with `a/b`, though the text does.
        "startsWith" | "endsWith" => {
            let other_text = other(heap, args.first())?;
            let other_absolute = other_text.starts_with('/');
            let other_names = path_names(&other_text);
            let matches = if method == "startsWith" {
                other_absolute == absolute
                    && other_names.len() <= names.len()
                    && names[..other_names.len()] == other_names[..]
            } else {
                // An ABSOLUTE argument must match the whole path, root and all.
                (!other_absolute || (absolute && other_names.len() == names.len()))
                    && other_names.len() <= names.len()
                    && names[names.len() - other_names.len()..] == other_names[..]
            };
            Ok(Some(JValue::Int(i32::from(matches))))
        }
        "subpath" => {
            let (from, to) = (index(args, 0), index(args, 1));
            let (Ok(from), Ok(to)) = (usize::try_from(from), usize::try_from(to)) else {
                return Err(throw("java.lang.IllegalArgumentException"));
            };
            if from >= to || to > names.len() {
                return Err(throw("java.lang.IllegalArgumentException"));
            }
            path_value(heap, names[from..to].join("/"))
        }
        // caturra's filesystem has no working directory beyond the root, so an
        // absolute path is itself and a relative one hangs off `/`.
        "toAbsolutePath" => {
            if absolute {
                path_value(heap, path)
            } else {
                path_value(heap, format!("/{path}"))
            }
        }
        "toFile" => Ok(Some(JValue::Ref(Some(
            heap.alloc(HeapObject::File(abstract_path(&path))),
        )))),
        "compareTo" => {
            let other = other(heap, args.first())?;
            Ok(Some(JValue::Int(match path.cmp(&other) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })))
        }
        // Two paths are equal when their TEXT is, which is what a JDK's
        // UnixPath compares; `hashCode` follows it.
        "equals" => {
            let equal = match args.first() {
                Some(JValue::Ref(Some(reference))) => {
                    matches!(heap.get(*reference), Some(HeapObject::Path(text)) if *text == path)
                }
                _ => false,
            };
            Ok(Some(JValue::Int(i32::from(equal))))
        }
        // A JDK's UnixPath hashes its BYTES, which for the paths a program
        // writes is the string hash.
        "hashCode" => Ok(Some(JValue::Int(java_string_hash(&path)))),
        _ => Err(VmError::UnknownIntrinsic(format!("Path.{method}"))),
    }
}

/// `java.io.PrintWriter` methods: formatting matches `PrintStream`, but
/// output appends to the writer's file in the virtual filesystem.
/// The single character a `write(int)`/`append(char)` code denotes — its low 16
/// bits, one UTF-16 code unit.
fn char_from_code(code: i32) -> String {
    let unit = u32::try_from(code & 0xFFFF).unwrap_or(0);
    char::from_u32(unit).map(String::from).unwrap_or_default()
}

fn writer_method(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    receiver: HeapRef,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let path = match heap.get(receiver) {
        Some(HeapObject::Writer { path }) => path.clone(),
        _ => unreachable!("receiver kind checked by caller"),
    };
    match method {
        "printf" | "format" => {
            let template = match args.first() {
                Some(JValue::Ref(Some(reference))) => {
                    heap.string_text(*reference).unwrap_or_default()
                }
                Some(JValue::Ref(None)) => {
                    return Err(throw("java.lang.NullPointerException: format is null"));
                }
                _ => String::new(),
            };
            let format_args = crate::format::args_from_descriptor(heap, descriptor, &args[1..])?;
            let text = crate::format::java_format(heap, &template, &format_args)?;
            vfs.append_file(&path, &units_to_utf8(&text))
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            // `format` returns the writer for chaining; `printf` is void.
            Ok((method == "format").then_some(JValue::Ref(Some(receiver))))
        }
        // `write(String)` writes the whole string; `write(int)` writes a single
        // character (its low 16 bits).
        "write" => {
            let text = if descriptor.starts_with("(I)") {
                match args.first() {
                    Some(JValue::Int(code)) => char_from_code(*code),
                    _ => String::new(),
                }
            } else {
                match args.first() {
                    Some(JValue::Ref(Some(reference))) => {
                        heap.string_text(*reference).unwrap_or_default()
                    }
                    _ => String::new(),
                }
            };
            vfs.append_file(&path, text.as_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(None)
        }
        // `append(char)` writes the character; `append(CharSequence)` the text
        // (a null appends the four characters "null"). Returns the writer.
        "append" => {
            let text = if descriptor.starts_with("(C)") {
                match args.first() {
                    Some(JValue::Int(code)) => char_from_code(*code),
                    _ => String::new(),
                }
            } else {
                match args.first() {
                    Some(JValue::Ref(Some(reference))) => {
                        heap.string_text(*reference).unwrap_or_default()
                    }
                    Some(JValue::Ref(None)) => String::from("null"),
                    _ => String::new(),
                }
            };
            vfs.append_file(&path, text.as_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        "print" | "println" => {
            let mut text = print_argument_text(heap, descriptor, args)?;
            if method == "println" {
                text.push('\n');
            }
            vfs.append_file(&path, text.as_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(None)
        }
        // Write-through means close/flush have nothing left to do.
        "close" | "flush" => Ok(None),
        _ => Err(VmError::UnknownIntrinsic(format!("PrintWriter.{method}"))),
    }
}

/// Java's `java.util.Random` LCG, used for `Math.random()`.
#[derive(Debug)]
pub struct JavaRng {
    seed: u64,
}

impl JavaRng {
    const MULTIPLIER: u64 = 0x5_DEEC_E66D;
    const MASK: u64 = (1 << 48) - 1;

    #[must_use]
    pub fn new(seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or(0x5EED_1234_5678);
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next(&mut self, bits: u32) -> u64 {
        self.seed = self.seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(0xB) & Self::MASK;
        self.seed >> (48 - bits)
    }

    /// `Random.nextDouble()`: uniform in `[0, 1)`.
    #[allow(clippy::cast_precision_loss)] // both values are < 2^53
    pub fn next_double(&mut self) -> f64 {
        let high = self.next(26) << 27;
        let low = self.next(27);
        ((high + low) as f64) / ((1u64 << 53) as f64)
    }
}

/// Intrinsic static methods, routed per class. Every method a student
/// can find in the Java 11 documentation either works or reports an
/// honest "not supported" reason at compile time.
fn is_wrapper_class(class: &str) -> bool {
    matches!(
        class,
        "java/lang/Integer"
            | "java/lang/Double"
            | "java/lang/Long"
            | "java/lang/Float"
            | "java/lang/Short"
            | "java/lang/Byte"
            | "java/lang/Character"
            | "java/lang/Boolean"
    )
}

/// Render a boxed primitive the way its wrapper's `toString` does.
pub(crate) fn boxed_to_string(class_name: &str, value: JValue) -> String {
    match (class_name, value) {
        ("java/lang/Double", JValue::Double(v)) => java_double_to_string(v),
        ("java/lang/Float", JValue::Float(v)) => java_float_to_string(v),
        ("java/lang/Long", JValue::Long(v)) => v.to_string(),
        ("java/lang/Boolean", JValue::Int(v)) => (v != 0).to_string(),
        ("java/lang/Character", JValue::Int(v)) => char::from_u32(u32::try_from(v).unwrap_or(0))
            .unwrap_or('\u{FFFD}')
            .to_string(),
        (_, JValue::Int(v)) => v.to_string(),
        (_, other) => format!("{other:?}"),
    }
}

/// `System.__box`: box a value that is really an unboxed primitive, and pass a
/// reference through unchanged. See the compiler's `emit_box_any`.
fn box_any(heap: &mut Heap, value: Option<JValue>) -> JValue {
    let primitive = match value {
        None => return JValue::Ref(None),
        Some(reference @ JValue::Ref(_)) => return reference,
        Some(primitive) => primitive,
    };
    let class_name = match primitive {
        JValue::Long(_) => "java/lang/Long",
        JValue::Double(_) => "java/lang/Double",
        JValue::Float(_) => "java/lang/Float",
        _ => "java/lang/Integer",
    };
    JValue::Ref(Some(heap.alloc(HeapObject::Boxed {
        class_name: std::rc::Rc::from(class_name),
        value: primitive,
    })))
}

/// Instance methods on a boxed wrapper: the unboxing accessors
/// (`intValue`, ...) and the Object methods (`toString`, `equals`,
/// `hashCode`, `compareTo`).
#[allow(clippy::too_many_lines)] // one arm per wrapper method
fn boxed_virtual(
    heap: &mut Heap,
    class_name: &str,
    value: JValue,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    // Unboxing accessors convert the stored primitive to the requested
    // numeric type.
    let as_double = |v: JValue| match v {
        JValue::Int(n) => f64::from(n),
        JValue::Long(n) => {
            #[allow(clippy::cast_precision_loss)]
            {
                n as f64
            }
        }
        JValue::Float(n) => f64::from(n),
        JValue::Double(n) => n,
        JValue::Ref(_) => 0.0,
    };
    let as_long = |v: JValue| match v {
        JValue::Int(n) => i64::from(n),
        JValue::Long(n) => n,
        #[allow(clippy::cast_possible_truncation)]
        JValue::Float(n) => n as i64,
        #[allow(clippy::cast_possible_truncation)]
        JValue::Double(n) => n as i64,
        JValue::Ref(_) => 0,
    };
    #[allow(clippy::cast_possible_truncation)]
    let as_int = |v: JValue| match v {
        JValue::Int(n) => n,
        JValue::Long(n) => n as i32,
        JValue::Float(n) => n as i32,
        JValue::Double(n) => n as i32,
        JValue::Ref(_) => 0,
    };
    match method {
        "intValue" | "shortValue" | "byteValue" | "charValue" => {
            Ok(Some(JValue::Int(as_int(value))))
        }
        "longValue" => Ok(Some(JValue::Long(as_long(value)))),
        "doubleValue" => Ok(Some(JValue::Double(as_double(value)))),
        #[allow(clippy::cast_possible_truncation)]
        "floatValue" => Ok(Some(JValue::Float(as_double(value) as f32))),
        "booleanValue" => Ok(Some(value)),
        // The INSTANCE forms a floating wrapper declares. `Double.isNaN(d)`
        // was modelled and `d.isNaN()` was not, though the second is the
        // spelling a program reaches for on a value it already has.
        "isNaN" => Ok(Some(JValue::Int(i32::from(as_double(value).is_nan())))),
        "isInfinite" => Ok(Some(JValue::Int(i32::from(as_double(value).is_infinite())))),
        "toString" => {
            // A `Character` is its UNIT, which may be an unpaired surrogate no
            // Rust `String` can hold. `String.valueOf(Object)` and a concat
            // whose operand is statically `Object` both lower to this
            // `toString`, so rendering it as text replaced the character.
            if let ("java/lang/Character", JValue::Int(unit)) = (class_name, value) {
                let unit = u16::try_from(unit).unwrap_or(u16::MAX);
                let reference = heap.alloc(HeapObject::JavaString(vec![unit]));
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            let text = boxed_to_string(class_name, value);
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        // Each wrapper's own `hashCode`, keyed by class: `Boolean` is
        // 1231/1237 (NOT its 0/1 value), `Long`/`Double`/`Float` fold their
        // bits, and `Integer`/`Short`/`Byte`/`Character` hash to their value.
        "hashCode" => {
            let hash = if class_name == "java/lang/Boolean" {
                if value == JValue::Int(0) { 1237 } else { 1231 }
            } else {
                match value {
                    JValue::Int(n) => n,
                    JValue::Long(n) => fold_to_int(n),
                    JValue::Double(n) => java_double_hash(n),
                    JValue::Float(n) => float_to_int_bits(n),
                    JValue::Ref(_) => 0,
                }
            };
            Ok(Some(JValue::Int(hash)))
        }
        "equals" => {
            // Equal iff the other operand is a wrapper of the same class and
            // value, or a raw unboxed primitive of equal value (caturra stores
            // primitives unboxed, so an `Object` arg may arrive unboxed — Java
            // would autobox it, `Integer.equals(5)` is true).
            let equal = match args.first() {
                Some(JValue::Ref(Some(other))) => match heap.get(*other) {
                    Some(HeapObject::Boxed {
                        class_name: other_class,
                        value: other_value,
                    }) => &**other_class == class_name && values_bit_equal(value, *other_value),
                    _ => false,
                },
                Some(
                    other @ (JValue::Int(_)
                    | JValue::Long(_)
                    | JValue::Double(_)
                    | JValue::Float(_)),
                ) => values_bit_equal(value, *other),
                _ => false,
            };
            Ok(Some(JValue::Int(i32::from(equal))))
        }
        "compareTo" => {
            // A wrapper's `compareTo` takes its OWN type — the JDK's is
            // `compareTo(Integer)`, and reaching it through `Comparable` casts
            // the argument. Comparing an Integer with a Long (or a String, or
            // any other object) is a ClassCastException, not a number.
            let other = match args.first() {
                Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::Boxed {
                        value,
                        class_name: other_class,
                    }) if other_class.as_ref() == class_name => *value,
                    other => {
                        let named = match other {
                            Some(
                                HeapObject::Boxed { class_name, .. }
                                | HeapObject::Instance { class_name, .. },
                            ) => class_name.replace('/', "."),
                            Some(HeapObject::JavaString(_)) => String::from("java.lang.String"),
                            _ => String::from("java.lang.Object"),
                        };
                        return Err(throw(format!(
                            "java.lang.ClassCastException: class {named} cannot be cast to \
                             class {}",
                            class_name.replace('/', ".")
                        )));
                    }
                },
                Some(JValue::Ref(None)) => return Err(throw("java.lang.NullPointerException")),
                // An UNBOXED primitive argument: caturra keeps wrapper values
                // unboxed where it can, so this is the ordinary same-type call.
                Some(other) => *other,
                None => JValue::Int(0),
            };
            let ordering = match (value, other) {
                (JValue::Double(a), JValue::Double(b)) => a.total_cmp(&b),
                (JValue::Float(a), JValue::Float(b)) => a.total_cmp(&b),
                (JValue::Long(a), JValue::Long(b)) => a.cmp(&b),
                _ => as_long(value).cmp(&as_long(other)),
            };
            Ok(Some(JValue::Int(match ordering {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })))
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{class_name}.{method} on a boxed value"
        ))),
    }
}

fn values_bit_equal(a: JValue, b: JValue) -> bool {
    match (a, b) {
        (JValue::Int(x), JValue::Int(y)) => x == y,
        (JValue::Long(x), JValue::Long(y)) => x == y,
        (JValue::Double(x), JValue::Double(y)) => x.to_bits() == y.to_bits(),
        (JValue::Float(x), JValue::Float(y)) => x.to_bits() == y.to_bits(),
        _ => false,
    }
}

/// Read an image pixel file written by the host: width and height as
/// little-endian u32, then `width * height` RGB triples. `None` when the file is
/// absent or truncated — the caller then reports "no image".
fn image_bytes<'a>(
    heap: &Heap,
    vfs: &'a VirtualFileSystem,
    args: &[JValue],
) -> Option<(u32, u32, &'a [u8])> {
    let Some(JValue::Ref(Some(reference))) = args.first() else {
        return None;
    };
    let path = heap.string_text(*reference)?;
    let bytes = vfs.read_file(&path).ok()?;
    if bytes.len() < 8 {
        return None;
    }
    let width = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
    let height = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
    let count = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(3)?;
    let rgb = bytes.get(8..8 + count)?;
    Some((width, height, rgb))
}

/// Sound samples, natively — the audio half of `system_image`, and for the same
/// reason only more so.
///
/// Samples cross the VFS as raw little-endian signed 16-bit PCM: exactly the
/// bytes the audio decoded from, `2 * len` of them. `beat.wav` is 4.9 MILLION
/// samples, so the text this used to be (`count s0 s1 …`) was 24 MB that the
/// *modelled* `Scanner` parsed one interpreted `nextInt()` at a time, and that
/// `playSound` built one interpreted `StringBuilder.append` at a time. Nobody
/// noticed, because a missing asset used to hand back silence and so no level
/// ever read a real sound.
///
/// An absent sound reads back empty, which is what lets `SoundLoader.read` throw
/// `FILE_NOT_FOUND` instead of inventing silence.
fn system_sound(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    method: &str,
    args: &[JValue],
) -> Option<JValue> {
    let Some(JValue::Ref(Some(reference))) = args.first() else {
        return None;
    };
    let path = heap.string_text(*reference)?;
    if method == "__soundSamples" {
        let samples = vfs.read_file(&path).map_or_else(
            |_| Vec::new(),
            |bytes| {
                bytes
                    .chunks_exact(2)
                    .map(|pair| f64::from(i16::from_le_bytes([pair[0], pair[1]])) / 32768.0)
                    .collect()
            },
        );
        return Some(JValue::Ref(Some(
            heap.alloc(HeapObject::DoubleArray(samples)),
        )));
    }
    // __writeSound(path, samples[])
    let Some(JValue::Ref(Some(array))) = args.get(1) else {
        return None;
    };
    let Some(HeapObject::DoubleArray(samples)) = heap.get(*array) else {
        return None;
    };
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "clamped to [-1, 1] and scaled, so it is in i16 range"
        )]
        let quantized = (clamped * 32767.0).round() as i16;
        bytes.extend_from_slice(&quantized.to_le_bytes());
    }
    let _ = vfs.write_file(&path, bytes);
    None
}

/// Image pixels, natively (`org.code.media.Image`). A 400x400 image is 160k
/// pixels; ferrying that through `PrintWriter`/`Scanner` text costs about a
/// minute in the interpreter, so the whole buffer crosses the VFS as bytes and is
/// packed/unpacked here in one call. `__imageDims`/`__imagePixels` read what the
/// host preloaded; `__writeImage` sends an edited image back out to be drawn.
fn system_image(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let ints = |heap: &mut Heap, values: Vec<i32>| {
        Ok(Some(JValue::Ref(Some(
            heap.alloc(HeapObject::IntArray(IntKind::Int, values)),
        ))))
    };
    match method {
        "__imageDims" => {
            let dims = match image_bytes(heap, vfs, args) {
                Some((width, height, _)) => vec![
                    i32::try_from(width).unwrap_or(0),
                    i32::try_from(height).unwrap_or(0),
                ],
                None => Vec::new(),
            };
            ints(heap, dims)
        }
        "__imagePixels" => {
            let pixels = match image_bytes(heap, vfs, args) {
                Some((width, height, rgb)) => (0..(width as usize).saturating_mul(height as usize))
                    .map(|i| {
                        let (r, g, b) = (rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]);
                        (i32::from(r) << 16) | (i32::from(g) << 8) | i32::from(b)
                    })
                    .collect(),
                None => Vec::new(),
            };
            ints(heap, pixels)
        }
        // __writeImage(path, width, height, packed[])
        _ => {
            let Some(JValue::Ref(Some(reference))) = args.first() else {
                return Ok(None);
            };
            let Some(path) = heap.string_text(*reference) else {
                return Ok(None);
            };
            let (Some(JValue::Int(width)), Some(JValue::Int(height))) = (args.get(1), args.get(2))
            else {
                return Ok(None);
            };
            let Some(JValue::Ref(Some(array))) = args.get(3) else {
                return Ok(None);
            };
            let Some(HeapObject::IntArray(_, packed)) = heap.get(*array) else {
                return Ok(None);
            };
            let (width, height) = (
                u32::try_from(*width).unwrap_or(0),
                u32::try_from(*height).unwrap_or(0),
            );
            let count = (width as usize).saturating_mul(height as usize);
            let mut bytes = Vec::with_capacity(8 + count * 3);
            bytes.extend_from_slice(&width.to_le_bytes());
            bytes.extend_from_slice(&height.to_le_bytes());
            for i in 0..count {
                let value = packed.get(i).copied().unwrap_or(0);
                bytes.push(u8::try_from((value >> 16) & 0xff).unwrap_or(0));
                bytes.push(u8::try_from((value >> 8) & 0xff).unwrap_or(0));
                bytes.push(u8::try_from(value & 0xff).unwrap_or(0));
            }
            let _ = vfs.write_file(&path, bytes);
            Ok(None)
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // one arm per library class
pub fn invoke_static(
    heap: &mut Heap,
    rng: &mut JavaRng,
    console: &mut dyn ConsoleIo,
    vfs: &mut VirtualFileSystem,
    class: &str,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    // Autoboxing: the compiler emits `Wrapper.valueOf(prim)LWrapper;`
    // (a wrapper return) to box. User `Integer.valueOf(7)` is compiled
    // with an `int` return and falls through unboxed.
    if method == "valueOf"
        && args.len() == 1
        && is_wrapper_class(class)
        && descriptor.ends_with(&format!("L{class};"))
        && !matches!(args[0], JValue::Ref(_))
    {
        let reference = heap.box_wrapper(class, args[0]);
        return Ok(Some(JValue::Ref(Some(reference))));
    }
    // `new Integer(5)` must mint a DISTINCT object every time (JLS §15.9.4:
    // a class instance creation expression always creates a new object), so
    // `new Integer(5) == new Integer(5)` is FALSE while
    // `Integer.valueOf(5) == Integer.valueOf(5)` is true. The compiler emits
    // this caturra-internal name for the deprecated constructors precisely so
    // the autoboxing cache above is bypassed.
    if method == "__newWrapper" && args.len() == 1 && is_wrapper_class(class) {
        let reference = heap.alloc(HeapObject::Boxed {
            class_name: std::rc::Rc::from(class),
            value: args[0],
        });
        return Ok(Some(JValue::Ref(Some(reference))));
    }
    match class {
        "java/lang/Math" => math_static(rng, method, args),
        "java/lang/Integer" => integer_static(heap, method, args),
        "java/lang/Double" => double_static(heap, method, args),
        "java/lang/Character" => character_static(heap, method, args),
        "java/lang/Boolean" => boolean_static(heap, method, args),
        "java/lang/Long" => long_static(heap, method, args),
        "java/lang/Float" => float_static(heap, method, args),
        "java/lang/Short" => small_int_static(heap, "Short", method, args),
        "java/lang/Byte" => small_int_static(heap, "Byte", method, args),
        // `java.time`'s factories. `__of` is how the compiler asks for an
        // enum CONSTANT (`DayOfWeek.MONDAY`) — a name the program cannot
        // write, and the interning happens on the way out.
        "java/time/LocalDate" => match method {
            "of" => {
                let (year, month, day) = match args {
                    [JValue::Int(y), JValue::Int(m), JValue::Int(d)] => (*y, *m, *d),
                    _ => {
                        return Err(throw(
                            "java.lang.VerifyError: LocalDate.of takes three ints",
                        ));
                    }
                };
                match crate::time::Date::of(year, month, day) {
                    Ok(date) => Ok(Some(JValue::Ref(Some(
                        heap.intern_temporal(Temporal::Date(date)),
                    )))),
                    Err(message) => Err(date_time_exception(&message)),
                }
            }
            // What "today" is depends on a ZONE, which the host owns: the
            // browser reads its own IANA data, and a host without one is UTC.
            "now" => {
                let offset = i64::from(console.zone_offset_seconds()) * 1000;
                let local = console.now_millis().saturating_add(offset);
                let date = crate::time::Date::from_epoch_day(local.div_euclid(86_400_000));
                Ok(Some(JValue::Ref(Some(
                    heap.intern_temporal(Temporal::Date(date)),
                ))))
            }
            // `ofYearDay(year, day)` — the n-th day of that year.
            "ofYearDay" => {
                let (year, day) = match args {
                    [JValue::Int(year), JValue::Int(day)] => (*year, i64::from(*day)),
                    _ => return Err(throw("java.lang.VerifyError: ofYearDay takes two ints")),
                };
                let start = match crate::time::Date::of(year, 1, 1) {
                    Ok(start) => start,
                    Err(message) => return Err(date_time_exception(&message)),
                };
                let length = i64::from(start.length_of_year());
                if !(1..=length).contains(&day) {
                    return Err(date_time_exception(&format!(
                        "Invalid value for DayOfYear (valid values 1 - 365/366): {day}"
                    )));
                }
                Ok(Some(JValue::Ref(Some(heap.intern_temporal(
                    Temporal::Date(start.plus_days(day - 1)),
                )))))
            }
            "ofEpochDay" => {
                let day = match args.first() {
                    Some(JValue::Long(day)) => *day,
                    Some(JValue::Int(day)) => i64::from(*day),
                    _ => 0,
                };
                let date = crate::time::Date::from_epoch_day(day);
                Ok(Some(JValue::Ref(Some(
                    heap.intern_temporal(Temporal::Date(date)),
                ))))
            }
            "parse" => {
                let text = match args.first() {
                    Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                    Some(JValue::Ref(None)) => {
                        return Err(throw("java.lang.NullPointerException: text"));
                    }
                    _ => None,
                }
                .unwrap_or_default();
                // `parse(text, formatter)` reads it the formatter's way.
                if let Some(JValue::Ref(Some(formatter))) = args.get(1) {
                    let formatter = *formatter;
                    return parse_with_formatter(heap, class, &text, formatter);
                }
                match crate::time::parse_date(&text) {
                    Ok(date) => Ok(Some(JValue::Ref(Some(
                        heap.intern_temporal(Temporal::Date(date)),
                    )))),
                    // A parse failure is its own exception type, not a
                    // DateTimeException — a program may catch either.
                    Err(message) => Err(VmError::UncaughtException(format!(
                        "java.time.format.DateTimeParseException: {message}"
                    ))),
                }
            }
            // `from(temporal)` — the date half of whatever it is handed.
            "from" => {
                let Some(JValue::Ref(Some(reference))) = args.first() else {
                    return Err(throw("java.lang.NullPointerException"));
                };
                let Some(HeapObject::Temporal(value)) = heap.get(*reference) else {
                    return Err(throw("java.lang.ClassCastException: not a temporal"));
                };
                match *value {
                    Temporal::Date(date) => Ok(Some(JValue::Ref(Some(
                        heap.intern_temporal(Temporal::Date(date)),
                    )))),
                    Temporal::DateTime(when) => Ok(Some(JValue::Ref(Some(
                        heap.intern_temporal(Temporal::Date(when.date)),
                    )))),
                    other => Err(date_time_exception(&format!(
                        "Unable to obtain LocalDate from TemporalAccessor: {}",
                        other.text()
                    ))),
                }
            }
            _ => Err(VmError::UnknownIntrinsic(format!(
                "java/time/LocalDate.{method}"
            ))),
        },
        "java/time/LocalTime" => {
            let field = |at: usize| match args.get(at) {
                Some(JValue::Int(value)) => *value,
                _ => 0,
            };
            match method {
                "of" => match crate::time::Time::of(field(0), field(1), field(2), field(3)) {
                    Ok(time) => Ok(Some(JValue::Ref(Some(
                        heap.intern_temporal(Temporal::Time(time)),
                    )))),
                    Err(message) => Err(date_time_exception(&message)),
                },
                "ofSecondOfDay" => {
                    let seconds = match args.first() {
                        Some(JValue::Long(value)) => *value,
                        Some(JValue::Int(value)) => i64::from(*value),
                        _ => 0,
                    };
                    if !(0..86_400).contains(&seconds) {
                        return Err(date_time_exception(&format!(
                            "Invalid value for SecondOfDay (valid values 0 - 86399): {seconds}"
                        )));
                    }
                    let time = crate::time::Time {
                        nano_of_day: seconds * crate::time::NANOS_PER_SECOND,
                    };
                    Ok(Some(JValue::Ref(Some(
                        heap.intern_temporal(Temporal::Time(time)),
                    ))))
                }
                // `MIDNIGHT`/`NOON`/`MIN`/`MAX`, asked for the way the
                // compiler asks for any library constant.
                "__of" => {
                    let time = crate::time::Time {
                        nano_of_day: match field(0) {
                            1 => 12 * crate::time::NANOS_PER_HOUR,
                            2 => crate::time::NANOS_PER_DAY - 1,
                            _ => 0,
                        },
                    };
                    Ok(Some(JValue::Ref(Some(
                        heap.intern_temporal(Temporal::Time(time)),
                    ))))
                }
                "parse" => {
                    let text = match args.first() {
                        Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                        Some(JValue::Ref(None)) => {
                            return Err(throw("java.lang.NullPointerException: text"));
                        }
                        _ => None,
                    }
                    .unwrap_or_default();
                    if let Some(JValue::Ref(Some(formatter))) = args.get(1) {
                        let formatter = *formatter;
                        return parse_with_formatter(heap, class, &text, formatter);
                    }
                    match crate::time::parse_time(&text) {
                        Ok(time) => Ok(Some(JValue::Ref(Some(
                            heap.intern_temporal(Temporal::Time(time)),
                        )))),
                        Err(message) => Err(VmError::UncaughtException(format!(
                            "java.time.format.DateTimeParseException: {message}"
                        ))),
                    }
                }
                "now" => {
                    let offset = i64::from(console.zone_offset_seconds()) * 1000;
                    let local = console.now_millis().saturating_add(offset);
                    let time = crate::time::Time {
                        nano_of_day: local.rem_euclid(86_400_000) * 1_000_000,
                    };
                    Ok(Some(JValue::Ref(Some(
                        heap.intern_temporal(Temporal::Time(time)),
                    ))))
                }
                "ofNanoOfDay" => {
                    let nanos = match args.first() {
                        Some(JValue::Long(value)) => *value,
                        Some(JValue::Int(value)) => i64::from(*value),
                        _ => 0,
                    };
                    if !(0..crate::time::NANOS_PER_DAY).contains(&nanos) {
                        return Err(date_time_exception(&format!(
                            "Invalid value for NanoOfDay (valid values 0 - 86399999999999): \
                             {nanos}"
                        )));
                    }
                    Ok(Some(JValue::Ref(Some(heap.intern_temporal(
                        Temporal::Time(crate::time::Time { nano_of_day: nanos }),
                    )))))
                }
                "from" => temporal_from(heap, args, false),
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "java/time/LocalTime.{method}"
                ))),
            }
        }
        "java/time/LocalDateTime" => local_date_time_static(heap, console, method, args),
        // `DateTimeFormatter.ofPattern(...)` and the ISO constants. The
        // pattern is checked HERE, because that is where a JDK throws for a
        // bad one — not at the first use.
        "java/time/format/DateTimeFormatter" => match method {
            "ofPattern" => {
                let pattern = match args.first() {
                    Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                    _ => return Err(throw("java.lang.NullPointerException: pattern")),
                }
                .unwrap_or_default();
                if let Err(message) = crate::time::parse_pattern(&pattern) {
                    return Err(throw(&*format!(
                        "java.lang.IllegalArgumentException: {message}"
                    )));
                }
                let formatter = heap.alloc(HeapObject::DateFormat(
                    crate::value::DateFormatKind::Pattern(pattern),
                ));
                Ok(Some(JValue::Ref(Some(formatter))))
            }
            // `__of` is how the compiler asks for a CONSTANT; all of the ISO
            // ones print what the value's own `toString` prints.
            // The three zone-less shapes are a date, a time and both;
            // `ISO_DATE`, `ISO_TIME` and `ISO_DATE_TIME` are those same three
            // for a value that carries no zone.
            "__of" => {
                let which = match args.first() {
                    Some(JValue::Int(index)) => (*index % 3).clamp(0, 2),
                    _ => 0,
                };
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let formatter = heap.alloc(HeapObject::DateFormat(
                    crate::value::DateFormatKind::Iso(which as u8),
                ));
                Ok(Some(JValue::Ref(Some(formatter))))
            }
            _ => Err(VmError::UnknownIntrinsic(format!(
                "java/time/format/DateTimeFormatter.{method}"
            ))),
        },
        "java/time/Duration" => {
            let count = match args.first() {
                Some(JValue::Long(value)) => *value,
                Some(JValue::Int(value)) => i64::from(*value),
                _ => 0,
            };
            let made = |heap: &mut Heap, amount: crate::time::Duration| {
                Ok(Some(JValue::Ref(Some(
                    heap.intern_temporal(Temporal::Duration(amount)),
                ))))
            };
            match method {
                "ofDays" => made(
                    heap,
                    crate::time::Duration::of_nanos(
                        count.saturating_mul(crate::time::NANOS_PER_DAY),
                    ),
                ),
                "ofHours" => made(
                    heap,
                    crate::time::Duration::of_nanos(
                        count.saturating_mul(crate::time::NANOS_PER_HOUR),
                    ),
                ),
                "ofMinutes" => made(
                    heap,
                    crate::time::Duration::of_nanos(
                        count.saturating_mul(crate::time::NANOS_PER_MINUTE),
                    ),
                ),
                "ofMillis" => made(
                    heap,
                    crate::time::Duration::of_nanos(count.saturating_mul(1_000_000)),
                ),
                "ofNanos" => made(heap, crate::time::Duration::of_nanos(count)),
                // `ofSeconds(seconds)` and `ofSeconds(seconds, nanoAdjust)`.
                "ofSeconds" => {
                    let adjust = match args.get(1) {
                        Some(JValue::Long(value)) => *value,
                        Some(JValue::Int(value)) => i64::from(*value),
                        _ => 0,
                    };
                    made(
                        heap,
                        crate::time::Duration::of_nanos(
                            count.saturating_mul(crate::time::NANOS_PER_SECOND) + adjust,
                        ),
                    )
                }
                // `__of` is the compiler's way of asking for `ZERO`.
                "__of" => made(heap, crate::time::Duration::of_nanos(0)),
                // `of(amount, unit)` — only a unit with a fixed length, since
                // a `Duration` is a number of seconds and not a calendar.
                "of" => {
                    let Some(unit) = unit_argument(heap, args.get(1)) else {
                        return Err(throw("java.lang.ClassCastException: not a ChronoUnit"));
                    };
                    let Some(nanos) = crate::time::unit_nanos(unit)
                        .filter(|_| unit == 7 || !crate::time::unit_is_estimated(unit))
                    else {
                        // The JDK names no unit here either.
                        return Err(VmError::UncaughtException(String::from(
                            "java.time.temporal.UnsupportedTemporalTypeException: \
                             Unit must not have an estimated duration",
                        )));
                    };
                    made(
                        heap,
                        crate::time::Duration::of_nanos(count.saturating_mul(nanos)),
                    )
                }
                "parse" => {
                    let text = match args.first() {
                        Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                        _ => None,
                    }
                    .unwrap_or_default();
                    match crate::time::parse_duration(&text) {
                        Ok(amount) => made(heap, amount),
                        Err(message) => Err(VmError::UncaughtException(format!(
                            "java.time.format.DateTimeParseException: {message}"
                        ))),
                    }
                }
                "between" => {
                    let read = |at: usize| match args.get(at) {
                        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                            Some(HeapObject::Temporal(value)) => Some(*value),
                            _ => None,
                        },
                        _ => None,
                    };
                    let instant = |value: Temporal| match value {
                        Temporal::Time(time) => Some(time.nano_of_day),
                        Temporal::Date(date) => Some(
                            date.to_epoch_day()
                                .saturating_mul(crate::time::NANOS_PER_DAY),
                        ),
                        Temporal::DateTime(when) => Some(
                            when.date
                                .to_epoch_day()
                                .saturating_mul(crate::time::NANOS_PER_DAY)
                                + when.time.nano_of_day,
                        ),
                        _ => None,
                    };
                    let (Some(start), Some(end)) = (read(0), read(1)) else {
                        return Err(throw("java.lang.ClassCastException: not a temporal"));
                    };
                    let (Some(from), Some(to)) = (instant(start), instant(end)) else {
                        return Err(throw("java.lang.ClassCastException: not a temporal"));
                    };
                    made(heap, crate::time::Duration::of_nanos(to - from))
                }
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "java/time/Duration.{method}"
                ))),
            }
        }
        "java/time/Period" => {
            let field = |at: usize| match args.get(at) {
                Some(JValue::Int(value)) => *value,
                _ => 0,
            };
            let made = |heap: &mut Heap, period: crate::time::Period| {
                Ok(Some(JValue::Ref(Some(
                    heap.intern_temporal(Temporal::Period(period)),
                ))))
            };
            match method {
                "of" => made(
                    heap,
                    crate::time::Period {
                        years: field(0),
                        months: field(1),
                        days: field(2),
                    },
                ),
                "parse" => {
                    let text = match args.first() {
                        Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                        _ => None,
                    }
                    .unwrap_or_default();
                    match crate::time::parse_period(&text) {
                        Ok(period) => made(heap, period),
                        Err(message) => Err(VmError::UncaughtException(format!(
                            "java.time.format.DateTimeParseException: {message}"
                        ))),
                    }
                }
                "ofYears" | "ofMonths" | "ofWeeks" | "ofDays" | "__of" => {
                    let count = field(0);
                    let period = match method {
                        "ofYears" => crate::time::Period {
                            years: count,
                            months: 0,
                            days: 0,
                        },
                        "ofMonths" => crate::time::Period {
                            years: 0,
                            months: count,
                            days: 0,
                        },
                        "ofWeeks" => crate::time::Period {
                            years: 0,
                            months: 0,
                            days: count.saturating_mul(7),
                        },
                        // `__of` is `ZERO`, which is `P0D`.
                        "__of" => crate::time::Period {
                            years: 0,
                            months: 0,
                            days: 0,
                        },
                        _ => crate::time::Period {
                            years: 0,
                            months: 0,
                            days: count,
                        },
                    };
                    made(heap, period)
                }
                "between" => {
                    let read = |at: usize| match args.get(at) {
                        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                            Some(HeapObject::Temporal(Temporal::Date(date))) => Some(*date),
                            _ => None,
                        },
                        _ => None,
                    };
                    let (Some(start), Some(end)) = (read(0), read(1)) else {
                        return Err(throw("java.lang.ClassCastException: not a LocalDate"));
                    };
                    made(heap, crate::time::Period::between(start, end))
                }
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "java/time/Period.{method}"
                ))),
            }
        }
        "java/time/temporal/TemporalAdjusters" => {
            use crate::time::AdjusterKind;
            let day = match args.first() {
                Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::Temporal(Temporal::DayOfWeek(day))) => *day,
                    _ => 1,
                },
                _ => 1,
            };
            // `dayOfWeekInMonth` takes the count FIRST, so its day is the
            // second argument.
            let (kind, day, ordinal) = match method {
                "firstDayOfMonth" => (AdjusterKind::FirstDayOfMonth, 1, 0),
                "lastDayOfMonth" => (AdjusterKind::LastDayOfMonth, 1, 0),
                "firstDayOfNextMonth" => (AdjusterKind::FirstDayOfNextMonth, 1, 0),
                "firstDayOfYear" => (AdjusterKind::FirstDayOfYear, 1, 0),
                "lastDayOfYear" => (AdjusterKind::LastDayOfYear, 1, 0),
                "firstDayOfNextYear" => (AdjusterKind::FirstDayOfNextYear, 1, 0),
                "firstInMonth" => (AdjusterKind::FirstInMonth, day, 1),
                "lastInMonth" => (AdjusterKind::LastInMonth, day, -1),
                "next" => (AdjusterKind::Next, day, 0),
                "nextOrSame" => (AdjusterKind::NextOrSame, day, 0),
                "previous" => (AdjusterKind::Previous, day, 0),
                "previousOrSame" => (AdjusterKind::PreviousOrSame, day, 0),
                "dayOfWeekInMonth" => {
                    let ordinal = match args.first() {
                        Some(JValue::Int(value)) => *value,
                        _ => 1,
                    };
                    let day = match args.get(1) {
                        Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                            Some(HeapObject::Temporal(Temporal::DayOfWeek(day))) => *day,
                            _ => 1,
                        },
                        _ => 1,
                    };
                    (AdjusterKind::DayOfWeekInMonth, day, ordinal)
                }
                _ => {
                    return Err(VmError::UnknownIntrinsic(format!(
                        "java/time/temporal/TemporalAdjusters.{method}"
                    )));
                }
            };
            let made = Temporal::Adjuster(crate::time::Adjuster { kind, day, ordinal });
            Ok(Some(JValue::Ref(Some(heap.intern_temporal(made)))))
        }
        // `IsoEra.BCE` and `IsoEra.CE`, by ordinal — and the two an enum
        // always answers.
        "java/time/chrono/IsoEra" => {
            let made = |heap: &mut Heap, ordinal: u8| {
                JValue::Ref(Some(heap.intern_temporal(Temporal::Era(ordinal))))
            };
            match method {
                "values" => {
                    let constants = vec![made(heap, 0), made(heap, 1)];
                    let array = heap.alloc(HeapObject::RefArray(String::from(class), constants));
                    Ok(Some(JValue::Ref(Some(array))))
                }
                "valueOf" => {
                    let name = match args.first() {
                        Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                        _ => None,
                    }
                    .unwrap_or_default();
                    match name.as_str() {
                        "BCE" => Ok(Some(made(heap, 0))),
                        "CE" => Ok(Some(made(heap, 1))),
                        _ => Err(throw(&*format!(
                            "java.lang.IllegalArgumentException: No enum constant \
                             java.time.chrono.IsoEra.{name}"
                        ))),
                    }
                }
                _ => {
                    let ordinal = match args.first() {
                        Some(JValue::Int(value)) => *value,
                        _ => 0,
                    };
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let ordinal = ordinal.clamp(0, 1) as u8;
                    Ok(Some(made(heap, ordinal)))
                }
            }
        }
        "java/time/temporal/ChronoUnit" | "java/time/temporal/ChronoField" => {
            let is_unit = class.ends_with("ChronoUnit");
            let names: &[&str] = if is_unit {
                &crate::time::UNIT_CONSTANTS
            } else {
                &FIELD_CONSTANTS
            };
            let made = |heap: &mut Heap, ordinal: u8| {
                let value = if is_unit {
                    Temporal::Unit(ordinal)
                } else {
                    Temporal::Field(ordinal)
                };
                JValue::Ref(Some(heap.intern_temporal(value)))
            };
            match method {
                // `values()` — a fresh array each call, holding the interned
                // constants in ordinal order.
                "values" => {
                    let mut constants = Vec::new();
                    for ordinal in 0..names.len() {
                        let ordinal = u8::try_from(ordinal).unwrap_or(0);
                        constants.push(made(heap, ordinal));
                    }
                    let array = heap.alloc(HeapObject::RefArray(class.to_owned(), constants));
                    Ok(Some(JValue::Ref(Some(array))))
                }
                "valueOf" => {
                    let name = match args.first() {
                        Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                        _ => None,
                    }
                    .unwrap_or_default();
                    match names.iter().position(|known| *known == name) {
                        Some(ordinal) => Ok(Some(made(heap, u8::try_from(ordinal).unwrap_or(0)))),
                        None => Err(throw(&*format!(
                            "java.lang.IllegalArgumentException: No enum constant {}.{name}",
                            class.replace('/', ".")
                        ))),
                    }
                }
                // A constant, named by its ordinal — how the compiler emits
                // `ChronoUnit.DAYS` and `ChronoField.YEAR`.
                _ => {
                    let ordinal = match args.first() {
                        Some(JValue::Int(value)) => *value,
                        _ => 0,
                    };
                    let last = i32::try_from(names.len()).unwrap_or(1) - 1;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let ordinal = ordinal.clamp(0, last) as u8;
                    Ok(Some(made(heap, ordinal)))
                }
            }
        }
        "java/time/DayOfWeek" | "java/time/Month" => {
            let day_of_week = class.ends_with("DayOfWeek");
            let limit = if day_of_week { 7 } else { 12 };
            let ordinal = match args.first() {
                Some(JValue::Int(value)) => *value,
                _ => 1,
            };
            match method {
                // `values()` — a fresh array each call (a JDK clones its own),
                // holding the interned constants in ordinal order.
                "values" => {
                    let mut constants = Vec::new();
                    for ordinal in 1..=limit {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let ordinal = ordinal as u8;
                        let value = if day_of_week {
                            Temporal::DayOfWeek(ordinal)
                        } else {
                            Temporal::Month(ordinal)
                        };
                        constants.push(JValue::Ref(Some(heap.intern_temporal(value))));
                    }
                    let array = heap.alloc(HeapObject::RefArray(class.to_owned(), constants));
                    Ok(Some(JValue::Ref(Some(array))))
                }
                // `from(temporal)` — the day or the month a date falls on.
                "from" => {
                    let Some(JValue::Ref(Some(reference))) = args.first() else {
                        return Err(throw("java.lang.NullPointerException"));
                    };
                    let Some(HeapObject::Temporal(value)) = heap.get(*reference) else {
                        return Err(throw("java.lang.ClassCastException: not a temporal"));
                    };
                    let date = match *value {
                        Temporal::Date(date) => date,
                        Temporal::DateTime(when) => when.date,
                        other => {
                            return Err(date_time_exception(&format!(
                                "Unable to obtain {} from TemporalAccessor: {}",
                                if day_of_week { "DayOfWeek" } else { "Month" },
                                other.text()
                            )));
                        }
                    };
                    let made = if day_of_week {
                        Temporal::DayOfWeek(date.day_of_week())
                    } else {
                        Temporal::Month(date.month)
                    };
                    Ok(Some(JValue::Ref(Some(heap.intern_temporal(made)))))
                }
                "valueOf" => {
                    let name = match args.first() {
                        Some(JValue::Ref(Some(reference))) => heap.string_text(*reference),
                        _ => None,
                    }
                    .unwrap_or_default();
                    let found = (1..=limit).find(|ordinal| {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let ordinal = *ordinal as u8;
                        let known = if day_of_week {
                            crate::time::day_name(ordinal)
                        } else {
                            crate::time::month_name(ordinal)
                        };
                        known == name
                    });
                    match found {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        Some(ordinal) => {
                            let ordinal = ordinal as u8;
                            let value = if day_of_week {
                                Temporal::DayOfWeek(ordinal)
                            } else {
                                Temporal::Month(ordinal)
                            };
                            Ok(Some(JValue::Ref(Some(heap.intern_temporal(value)))))
                        }
                        None => Err(throw(&*format!(
                            "java.lang.IllegalArgumentException: No enum constant {}.{name}",
                            class.replace('/', ".")
                        ))),
                    }
                }
                "__of" | "of" => {
                    if !(1..=limit).contains(&ordinal) {
                        let field = if day_of_week {
                            "DayOfWeek"
                        } else {
                            "MonthOfYear"
                        };
                        return Err(date_time_exception(&format!(
                            "Invalid value for {field} (valid values 1 - {limit}): {ordinal}"
                        )));
                    }
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let ordinal = ordinal as u8;
                    let value = if day_of_week {
                        Temporal::DayOfWeek(ordinal)
                    } else {
                        Temporal::Month(ordinal)
                    };
                    Ok(Some(JValue::Ref(Some(heap.intern_temporal(value)))))
                }
                _ => Err(VmError::UnknownIntrinsic(format!("{class}.{method}"))),
            }
        }
        "java/lang/System" => match method {
            // The interpreter intercepts arraycopy (its element check needs the
            // class hierarchy); this path only runs if that one is bypassed.
            "arraycopy" => system_arraycopy(heap, args, &|_, _, _| true),
            // The JVM's is system-dependent; caturra always runs where a line
            // ends with a newline.
            "lineSeparator" => Ok(Some(JValue::Ref(Some(heap.alloc_string("\n"))))),
            "currentTimeMillis" => Ok(Some(JValue::Long(console.now_millis()))),
            "nanoTime" => Ok(Some(JValue::Long(
                console.now_millis().wrapping_mul(1_000_000),
            ))),
            // Terminate the program: unwind the whole stack uncatchably (no
            // catch/finally runs, matching the real JVM) with the status code.
            "exit" => {
                let code = match args.first() {
                    Some(JValue::Int(code)) => *code,
                    _ => 0,
                };
                Err(VmError::SystemExit(code))
            }
            // Internal: box whatever is on the stack. A collection whose
            // static element type is a wildcard or a type variable reads out as
            // `Object`, but the collection stores its primitives UNBOXED — so
            // the read has to be boxed before anything treats it as a
            // reference. The compiler cannot emit a `valueOf` here because it
            // does not know which primitive (that is exactly what the wildcard
            // hid); the VM does, because it has the value.
            "__box" => Ok(Some(box_any(heap, args.first().copied()))),
            // Standard-out capture for org.code.validation's SystemOutTestRunner.
            "__captureStart" => {
                console.begin_capture();
                Ok(None)
            }
            "__captureEnd" => {
                let messages = console.take_capture();
                let refs: Vec<JValue> = messages
                    .iter()
                    .map(|m| JValue::Ref(Some(heap.alloc_string(m))))
                    .collect();
                Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::RefArray(
                    String::from("[Ljava/lang/String;"),
                    refs,
                ))))))
            }
            // Swing event pump: render the tree (arg 0), block for the next
            // event, return its payload String — or null to end the loop.
            "__uiAwait" => {
                let tree = match args.first() {
                    Some(JValue::Ref(Some(reference))) => {
                        heap.string_text(*reference).unwrap_or_default()
                    }
                    _ => String::new(),
                };
                match console.ui_await_event(&tree) {
                    Some(payload) => Ok(Some(JValue::Ref(Some(heap.alloc_string(&payload))))),
                    None => Ok(Some(JValue::Ref(None))),
                }
            }
            // Blocking JOptionPane dialog: kind (arg 0) + message (arg 1);
            // returns the response String, or null when dismissed.
            "__uiDialog" => {
                let arg_str = |i: usize| match args.get(i) {
                    Some(JValue::Ref(Some(reference))) => {
                        heap.string_text(*reference).unwrap_or_default()
                    }
                    _ => String::new(),
                };
                let (kind, message) = (arg_str(0), arg_str(1));
                match console.ui_dialog(&kind, &message) {
                    Some(response) => Ok(Some(JValue::Ref(Some(heap.alloc_string(&response))))),
                    None => Ok(Some(JValue::Ref(None))),
                }
            }
            "__imageDims" | "__imagePixels" | "__writeImage" => {
                system_image(heap, vfs, method, args)
            }
            "__soundSamples" | "__writeSound" => Ok(system_sound(heap, vfs, method, args)),
            _ => Err(VmError::UnknownIntrinsic(format!("System.{method}"))),
        },
        "java/lang/String" => string_static(heap, method, descriptor, args),
        // `Path.of` / `Paths.get` — wrap a path string as a Path.
        // `Path.of(first, more...)` / `Paths.get(first, more...)`: the
        // segments are JOINED with the separator, and an empty one contributes
        // nothing — `Path.of("a", "", "b")` is `a/b`, and so is
        // `Path.of("a/", "b")`.
        "java/nio/file/Path" | "java/nio/file/Paths" => {
            let first = arg_string(heap, &args[0])?;
            let mut segments: Vec<String> = vec![first];
            if let Some(JValue::Ref(Some(rest))) = args.get(1)
                && let Some(HeapObject::RefArray(_, values)) = heap.get(*rest)
            {
                let values = values.clone();
                for value in &values {
                    segments.push(arg_string(heap, value)?);
                }
            }
            let leading = segments.first().is_some_and(|s| s.starts_with('/'));
            let parts: Vec<&str> = segments
                .iter()
                .flat_map(|segment| segment.split('/'))
                .filter(|part| !part.is_empty())
                .collect();
            let mut text = parts.join("/");
            if leading {
                text.insert(0, '/');
            }
            Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::Path(text))))))
        }
        "java/nio/file/Files" => files_static(heap, vfs, method, args),
        // `Pattern.compile(regex[, flags])` / `Pattern.matches(regex, text)` /
        // `Pattern.quote(text)`. A pattern object holds its SOURCE, with the
        // flags folded in as the inline prefix the engine already reads, and
        // the compiled form is rebuilt per use exactly as `String.matches`
        // rebuilds it.
        "java/util/regex/Pattern" => match method {
            "compile" => {
                let source = string_units(heap, &args[0])?;
                let flags = match args.get(1) {
                    Some(JValue::Int(flags)) => *flags,
                    _ => 0,
                };
                // Compiled once HERE so a malformed pattern fails at
                // `compile`, where a JDK reports it, rather than at the first
                // `find`.
                check_regex_flags(flags)?;
                compile_regex(&fold_regex_flags(&source, flags))?;
                Ok(Some(JValue::Ref(Some(
                    heap.alloc(HeapObject::Pattern { source, flags }),
                ))))
            }
            "matches" => {
                let source = string_units(heap, &args[0])?;
                let input = string_units(heap, &args[1])?;
                let regex = compile_regex(&source)?;
                Ok(Some(JValue::Int(i32::from(regex.matches_whole(&input)))))
            }
            "quote" => {
                let text = arg_string(heap, &args[0])?;
                let quoted = format!("\\Q{text}\\E");
                Ok(Some(JValue::Ref(Some(heap.alloc_string(&quoted)))))
            }
            _ => Err(VmError::UnknownIntrinsic(format!("Pattern.{method}"))),
        },
        // `Matcher.quoteReplacement(text)` — the escaping that makes a
        // replacement literal, so a `$` in it is a dollar and not a group.
        "java/util/regex/Matcher" => match method {
            "quoteReplacement" => {
                let text = arg_string(heap, &args[0])?;
                if !text.contains('\\') && !text.contains('$') {
                    return Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))));
                }
                let mut quoted = String::with_capacity(text.len() * 2);
                for ch in text.chars() {
                    if ch == '\\' || ch == '$' {
                        quoted.push('\\');
                    }
                    quoted.push(ch);
                }
                Ok(Some(JValue::Ref(Some(heap.alloc_string(&quoted)))))
            }
            _ => Err(VmError::UnknownIntrinsic(format!("Matcher.{method}"))),
        },
        // `Charset.forName(name)` — and the `StandardCharsets` constants, which
        // the compiler lowers to the same call. An unknown name is the JDK's
        // `UnsupportedCharsetException`, whose message is the name itself.
        "java/nio/charset/Charset" => match method {
            "defaultCharset" => Ok(Some(JValue::Ref(Some(
                heap.alloc(HeapObject::Charset(String::from("UTF-8"))),
            )))),
            "forName" | "__standard" => {
                let written = arg_string(heap, &args[0])?;
                let Some(name) = canonical_charset(&written) else {
                    return Err(throw(format!(
                        "java.nio.charset.UnsupportedCharsetException: {written}"
                    )));
                };
                Ok(Some(JValue::Ref(Some(
                    heap.alloc(HeapObject::Charset(name.to_owned())),
                ))))
            }
            _ => Err(VmError::UnknownIntrinsic(format!("Charset.{method}"))),
        },
        _ => Err(VmError::UnknownIntrinsic(format!("{class}.{method}"))),
    }
}

/// The text of a `String` argument (null throws NPE).
fn arg_string(heap: &Heap, value: &JValue) -> Result<String, VmError> {
    match value {
        JValue::Ref(Some(reference)) => Ok(heap.string_text(*reference).unwrap_or_default()),
        JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
        _ => Err(throw("java.lang.VerifyError: expected a String argument")),
    }
}

/// The path string behind a `java.nio.file.Path` argument.
fn path_arg(heap: &Heap, value: &JValue) -> Result<String, VmError> {
    match value {
        JValue::Ref(Some(reference)) => match heap.get(*reference) {
            Some(HeapObject::Path(path)) => Ok(path.clone()),
            _ => Err(throw("java.lang.ClassCastException: not a Path")),
        },
        JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
        _ => Err(throw("java.lang.VerifyError: expected a Path argument")),
    }
}

/// `java.nio.file.Files` static methods, over the virtual filesystem.
fn files_static(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let path = path_arg(heap, &args[0])?;
    let not_found = || throw(format!("java.nio.file.NoSuchFileException: {path}"));
    match method {
        "readString" => {
            let content = vfs.read_file(&path).map_err(|_| not_found())?.to_vec();
            let text = String::from_utf8_lossy(&content).into_owned();
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        "readAllLines" => {
            let content = vfs.read_file(&path).map_err(|_| not_found())?.to_vec();
            let text = String::from_utf8_lossy(&content).into_owned();
            let lines: Vec<JValue> = text
                .lines()
                .map(|line| JValue::Ref(Some(heap.alloc_string(line))))
                .collect();
            Ok(Some(JValue::Ref(Some(
                heap.alloc(HeapObject::ArrayList(lines)),
            ))))
        }
        // `Files.lines(path)` is `readAllLines` as a stream. A JDK's is lazy and
        // closeable; this one reads at once, which a program that counts,
        // filters or collects the lines cannot tell apart.
        "lines" => {
            let content = vfs.read_file(&path).map_err(|_| not_found())?.to_vec();
            let text = String::from_utf8_lossy(&content).into_owned();
            let lines: Vec<JValue> = text
                .lines()
                .map(|line| JValue::Ref(Some(heap.alloc_string(line))))
                .collect();
            Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::Stream {
                source: crate::value::StreamSource::Fixed(lines),
                ops: Vec::new(),
            })))))
        }
        "writeString" => {
            let text = arg_string(heap, &args[1])?;
            vfs.write_file(&path, text.into_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(Some(args[0]))
        }
        "write" => {
            let lines = match args.get(1) {
                Some(JValue::Ref(Some(reference))) => heap.list_values(*reference).cloned(),
                _ => None,
            }
            .ok_or_else(|| throw("java.lang.NullPointerException"))?;
            let mut text = String::new();
            for line in &lines {
                if let JValue::Ref(Some(reference)) = line {
                    text.push_str(&heap.string_text(*reference).unwrap_or_default());
                }
                text.push('\n');
            }
            vfs.write_file(&path, text.into_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(Some(args[0]))
        }
        "exists" | "isRegularFile" => Ok(Some(JValue::Int(i32::from(vfs.exists(&path))))),
        "notExists" => Ok(Some(JValue::Int(i32::from(!vfs.exists(&path))))),
        "isDirectory" => Ok(Some(JValue::Int(i32::from(vfs.is_directory(&path))))),
        "delete" => {
            vfs.remove(&path).map_err(|_| not_found())?;
            Ok(None)
        }
        // ...and the form that answers instead of throwing.
        "deleteIfExists" => Ok(Some(JValue::Int(i32::from(vfs.remove(&path).is_ok())))),
        "createFile" => {
            vfs.write_file(&path, Vec::new())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(Some(args[0]))
        }
        "createDirectory" => {
            let _ = vfs.mkdir(&path);
            Ok(Some(args[0]))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Files.{method}"))),
    }
}

/// The Java type name for a field descriptor: `I` -> `int`,
/// `Ljava/lang/String;` -> `java.lang.String`, `[I` -> `int[]`.
#[must_use]
pub fn type_name_of_descriptor(descriptor: &str) -> String {
    let base = descriptor.trim_start_matches('[');
    let dims = descriptor.len() - base.len();
    let mut name = match base {
        "I" => String::from("int"),
        "J" => String::from("long"),
        "D" => String::from("double"),
        "F" => String::from("float"),
        "Z" => String::from("boolean"),
        "C" => String::from("char"),
        "S" => String::from("short"),
        "B" => String::from("byte"),
        "V" => String::from("void"),
        other if other.starts_with('L') && other.ends_with(';') => {
            other[1..other.len() - 1].replace('/', ".")
        }
        other => other.to_owned(),
    };
    for _ in 0..dims {
        name.push_str("[]");
    }
    name
}

/// Java's canonical `Field.toString()`:
/// `<modifiers> <type> <DeclaringClass>.<name>`.
#[must_use]
pub fn field_to_string(declaring: &str, name: &str, descriptor: &str, access: u16) -> String {
    use caturra_classfile::FieldAccessFlags as F;
    let mut out = String::new();
    for (flag, word) in [
        (F::PUBLIC, "public"),
        (F::PRIVATE, "private"),
        (F::PROTECTED, "protected"),
        (F::STATIC, "static"),
        (F::FINAL, "final"),
        (F::TRANSIENT, "transient"),
        (F::VOLATILE, "volatile"),
    ] {
        if access & flag != 0 {
            out.push_str(word);
            out.push(' ');
        }
    }
    out.push_str(&type_name_of_descriptor(descriptor));
    out.push(' ');
    out.push_str(declaring);
    out.push('.');
    out.push_str(name);
    out
}

/// The Java type names of a method descriptor's parameters:
/// `(Ljava/lang/String;I)V` -> `["java.lang.String", "int"]`.
#[must_use]
pub fn param_type_names(descriptor: &str) -> Vec<String> {
    let mut names = Vec::new();
    let inner = match (descriptor.find('('), descriptor.find(')')) {
        (Some(open), Some(close)) if open < close => &descriptor[open + 1..close],
        _ => return names,
    };
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        while i < bytes.len() && bytes[i] == b'[' {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'L' {
            while i < bytes.len() && bytes[i] != b';' {
                i += 1;
            }
            i += 1; // consume ';'
        } else {
            i += 1; // a single primitive char
        }
        names.push(type_name_of_descriptor(&inner[start..i]));
    }
    names
}

/// Java's canonical `Constructor.toString()`:
/// `<modifiers> <DeclaringClass>(<param types>)`.
#[must_use]
pub fn constructor_to_string(declaring: &str, descriptor: &str, access: u16) -> String {
    use caturra_classfile::MethodAccessFlags as M;
    let mut out = String::new();
    for (flag, word) in [
        (M::PUBLIC, "public"),
        (M::PRIVATE, "private"),
        (M::PROTECTED, "protected"),
    ] {
        if access & flag != 0 {
            out.push_str(word);
            out.push(' ');
        }
    }
    out.push_str(declaring);
    out.push('(');
    out.push_str(&param_type_names(descriptor).join(","));
    out.push(')');
    out
}

/// Java's `Math.max`/`min` double semantics: NaN wins, `+0.0 > -0.0`.
#[allow(clippy::float_cmp)]
fn java_double_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        // +0.0 beats -0.0.
        return if a.is_sign_positive() { a } else { b };
    }
    if a > b { a } else { b }
}

#[allow(clippy::float_cmp)]
fn java_double_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_negative() { a } else { b };
    }
    if a < b { a } else { b }
}

/// `Math.floorDiv` with Java's toward-negative-infinity semantics.
fn java_floor_div(a: i32, b: i32) -> Result<i32, VmError> {
    if b == 0 {
        return Err(throw("java.lang.ArithmeticException: / by zero"));
    }
    let quotient = a.wrapping_div(b);
    if (a ^ b) < 0 && quotient.wrapping_mul(b) != a {
        Ok(quotient - 1)
    } else {
        Ok(quotient)
    }
}

/// The `long` form of [`java_floor_div`].
fn java_floor_div_long(a: i64, b: i64) -> Result<i64, VmError> {
    if b == 0 {
        return Err(throw("java.lang.ArithmeticException: / by zero"));
    }
    let quotient = a.wrapping_div(b);
    if (a ^ b) < 0 && quotient.wrapping_mul(b) != a {
        Ok(quotient - 1)
    } else {
        Ok(quotient)
    }
}

fn overflow() -> VmError {
    throw("java.lang.ArithmeticException: integer overflow")
}

fn long_overflow() -> VmError {
    throw("java.lang.ArithmeticException: long overflow")
}

/// `Math.round(double)` — half-up toward positive infinity, computed on the
/// bit pattern exactly as JDK 11 does. The naive `(a + 0.5).floor()` rounds
/// `0.49999999999999994` UP, because adding `0.5` to it in `double` overflows
/// to `1.0`; this does not. The out-of-`[0,63]` branch is the `(long) a` cast,
/// which Rust's saturating float→int matches (NaN→0, ±∞→`i64::MIN`/`i64::MAX`).
fn java_round_double(a: f64) -> i64 {
    const SIGNIFICAND_WIDTH: i64 = 53;
    const EXP_BIAS: i64 = 1023;
    const EXP_BIT_MASK: i64 = 0x7FF0_0000_0000_0000;
    const SIGNIF_BIT_MASK: i64 = 0x000F_FFFF_FFFF_FFFF;
    let bits = a.to_bits().cast_signed();
    let biased_exp = (bits & EXP_BIT_MASK) >> (SIGNIFICAND_WIDTH - 1);
    let shift = (SIGNIFICAND_WIDTH - 2 + EXP_BIAS) - biased_exp;
    if (shift & -64) == 0 {
        let mut r = (bits & SIGNIF_BIT_MASK) | (SIGNIF_BIT_MASK + 1);
        if bits < 0 {
            r = -r;
        }
        ((r >> shift) + 1) >> 1
    } else {
        a as i64
    }
}

/// `Math.round(float)` — the 32-bit twin of [`java_round_double`], returning
/// an `int`.
fn java_round_float(a: f32) -> i32 {
    const SIGNIFICAND_WIDTH: i32 = 24;
    const EXP_BIAS: i32 = 127;
    const EXP_BIT_MASK: i32 = 0x7F80_0000;
    const SIGNIF_BIT_MASK: i32 = 0x007F_FFFF;
    let bits = a.to_bits().cast_signed();
    let biased_exp = (bits & EXP_BIT_MASK) >> (SIGNIFICAND_WIDTH - 1);
    let shift = (SIGNIFICAND_WIDTH - 2 + EXP_BIAS) - biased_exp;
    if (shift & -32) == 0 {
        let mut r = (bits & SIGNIF_BIT_MASK) | (SIGNIF_BIT_MASK + 1);
        if bits < 0 {
            r = -r;
        }
        ((r >> shift) + 1) >> 1
    } else {
        a as i32
    }
}

/// Java's `Math.pow`. Rust's `f64::powf` (glibc) is correctly rounded except on
/// exact ties, where it rounds half-away from zero while the JDK rounds half-to-
/// even — e.g. `17^13`, an odd integer sitting exactly on a double midpoint. The
/// JDK's `Math.pow` is correctly rounded for *every* integer exponent (verified
/// against a real JVM), so an integer base raised to a non-negative integer
/// power is computed exactly: the exact `u128` power cast to `f64` rounds half-
/// to-even, matching the JDK. Everything else defers to glibc, which already
/// matches the JDK bit-for-bit for non-integer / non-representable results.
#[allow(clippy::float_cmp)]
fn java_pow(a: f64, b: f64) -> f64 {
    // A NaN exponent always gives NaN (JLS/Math.pow) — unlike C99/Rust's
    // `powf`, which returns 1.0 for `pow(1.0, NaN)`. The zero-exponent cases
    // (b == ±0.0) are not NaN, so `pow(NaN, 0)` still reaches `1.0` below.
    if b.is_nan() {
        return f64::NAN;
    }
    // Java's one deviation from IEEE 754: |x| == 1 with an infinite exponent.
    if a.abs() == 1.0 && b.is_infinite() {
        return f64::NAN;
    }
    exact_integer_pow(a, b).unwrap_or_else(|| a.powf(b))
}

/// The correctly-rounded value of integer `a` raised to a non-negative integer
/// power `b`, when the exact power fits in `u128`; otherwise `None` (defer to
/// glibc). Negative exponents and |base| < 2 already match the JDK bit-for-bit,
/// so they are left to `powf`.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]
fn exact_integer_pow(a: f64, b: f64) -> Option<f64> {
    if a.fract() != 0.0 || b.fract() != 0.0 || !(1.0..=127.0).contains(&b) {
        return None;
    }
    let mag = a.abs();
    if mag < 2.0 {
        return None;
    }
    let exp = b as u32;
    let base = mag as u128;
    let mut acc: u128 = 1;
    for _ in 0..exp {
        acc = acc.checked_mul(base)?;
    }
    // `u128 as f64` rounds to nearest, ties to even — the correctly-rounded
    // result the JDK produces.
    let val = acc as f64;
    Some(if a < 0.0 && exp % 2 == 1 { -val } else { val })
}

#[allow(clippy::too_many_lines)] // one arm per documented method
#[allow(clippy::float_cmp, clippy::many_single_char_names)]
fn math_static(
    rng: &mut JavaRng,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let d = |v: f64| Ok(Some(JValue::Double(v)));
    let i = |v: i32| Ok(Some(JValue::Int(v)));
    match (method, args) {
        ("abs", [JValue::Int(v)]) => i(v.wrapping_abs()),
        ("abs", [JValue::Long(v)]) => Ok(Some(JValue::Long(v.wrapping_abs()))),
        ("abs", [JValue::Float(v)]) => Ok(Some(JValue::Float(v.abs()))),
        ("max", [JValue::Float(a), JValue::Float(b)]) => {
            Ok(Some(JValue::Float(java_float_max(*a, *b))))
        }
        ("min", [JValue::Float(a), JValue::Float(b)]) => {
            Ok(Some(JValue::Float(java_float_min(*a, *b))))
        }
        ("signum", [JValue::Float(v)]) => Ok(Some(JValue::Float(if *v == 0.0 || v.is_nan() {
            *v
        } else {
            v.signum()
        }))),
        ("max", [JValue::Long(a), JValue::Long(b)]) => Ok(Some(JValue::Long((*a).max(*b)))),
        ("min", [JValue::Long(a), JValue::Long(b)]) => Ok(Some(JValue::Long((*a).min(*b)))),
        ("toIntExact", [JValue::Long(v)]) => {
            i32::try_from(*v).map_or_else(|_| Err(overflow()), |v| Ok(Some(JValue::Int(v))))
        }
        ("multiplyHigh", [JValue::Long(a), JValue::Long(b)]) => {
            let product = i128::from(*a) * i128::from(*b);
            #[allow(clippy::cast_possible_truncation)]
            Ok(Some(JValue::Long((product >> 64) as i64)))
        }
        // The exact 64-bit product of two ints, which `int * int` would wrap.
        ("multiplyFull", [JValue::Int(a), JValue::Int(b)]) => {
            Ok(Some(JValue::Long(i64::from(*a) * i64::from(*b))))
        }
        ("scalb", [JValue::Double(value), JValue::Int(scale)]) => {
            d(java_scalb_double(*value, *scale))
        }
        ("scalb", [JValue::Float(value), JValue::Int(scale)]) => {
            Ok(Some(JValue::Float(java_scalb_float(*value, *scale))))
        }
        ("abs", [JValue::Double(v)]) => d(v.abs()),
        ("sqrt", [JValue::Double(v)]) => d(v.sqrt()),
        ("cbrt", [JValue::Double(v)]) => d(crate::floatdec::java_cbrt(*v)),
        ("pow", [JValue::Double(a), JValue::Double(b)]) => d(java_pow(*a, *b)),
        ("hypot", [JValue::Double(a), JValue::Double(b)]) => d(crate::floatdec::java_hypot(*a, *b)),
        ("max", [JValue::Int(a), JValue::Int(b)]) => i((*a).max(*b)),
        ("max", [JValue::Double(a), JValue::Double(b)]) => d(java_double_max(*a, *b)),
        ("min", [JValue::Int(a), JValue::Int(b)]) => i((*a).min(*b)),
        ("min", [JValue::Double(a), JValue::Double(b)]) => d(java_double_min(*a, *b)),
        ("random", []) => d(rng.next_double()),
        ("floor", [JValue::Double(v)]) => d(v.floor()),
        ("ceil", [JValue::Double(v)]) => d(v.ceil()),
        ("rint", [JValue::Double(v)]) => d(v.round_ties_even()),
        ("round", [JValue::Double(v)]) => Ok(Some(JValue::Long(java_round_double(*v)))),
        ("round", [JValue::Float(v)]) => i(java_round_float(*v)),
        ("sin", [JValue::Double(v)]) => d(v.sin()),
        ("cos", [JValue::Double(v)]) => d(v.cos()),
        ("tan", [JValue::Double(v)]) => d(v.tan()),
        ("asin", [JValue::Double(v)]) => d(crate::floatdec::java_asin(*v)),
        ("acos", [JValue::Double(v)]) => d(crate::floatdec::java_acos(*v)),
        ("atan", [JValue::Double(v)]) => d(crate::floatdec::java_atan(*v)),
        ("atan2", [JValue::Double(a), JValue::Double(b)]) => d(crate::floatdec::java_atan2(*a, *b)),
        ("sinh", [JValue::Double(v)]) => d(v.sinh()),
        ("cosh", [JValue::Double(v)]) => d(crate::floatdec::java_cosh(*v)),
        ("tanh", [JValue::Double(v)]) => d(v.tanh()),
        ("exp", [JValue::Double(v)]) => d(v.exp()),
        ("expm1", [JValue::Double(v)]) => d(v.exp_m1()),
        ("log", [JValue::Double(v)]) => d(v.ln()),
        ("log10", [JValue::Double(v)]) => d(v.log10()),
        ("log1p", [JValue::Double(v)]) => d(v.ln_1p()),
        ("signum", [JValue::Double(v)]) => {
            // Rust's signum maps ±0 to ±1; Java keeps ±0 and NaN.
            d(if *v == 0.0 || v.is_nan() {
                *v
            } else {
                v.signum()
            })
        }
        ("toDegrees", [JValue::Double(v)]) => d(v.to_degrees()),
        ("toRadians", [JValue::Double(v)]) => d(v.to_radians()),
        ("copySign", [JValue::Double(a), JValue::Double(b)]) => d(a.copysign(*b)),
        ("copySign", [JValue::Float(a), JValue::Float(b)]) => {
            Ok(Some(JValue::Float(a.copysign(*b))))
        }
        ("ulp", [JValue::Double(v)]) => {
            let v = v.abs();
            d(if v.is_nan() {
                f64::NAN
            } else if v.is_infinite() {
                f64::INFINITY
            } else if v == f64::MAX {
                f64::MAX - f64::from_bits(f64::MAX.to_bits() - 1)
            } else {
                v.next_up() - v
            })
        }
        ("nextUp", [JValue::Double(v)]) => d(v.next_up()),
        ("nextDown", [JValue::Double(v)]) => d(v.next_down()),
        ("nextUp", [JValue::Float(v)]) => Ok(Some(JValue::Float(v.next_up()))),
        ("nextDown", [JValue::Float(v)]) => Ok(Some(JValue::Float(v.next_down()))),
        // `ulp(float)` in float precision — `ulp(0.0f)` is Float.MIN_VALUE.
        ("ulp", [JValue::Float(v)]) => {
            let v = v.abs();
            Ok(Some(JValue::Float(if v.is_nan() {
                f32::NAN
            } else if v.is_infinite() {
                f32::INFINITY
            } else if v == f32::MAX {
                f32::MAX - f32::from_bits(f32::MAX.to_bits() - 1)
            } else {
                v.next_up() - v
            })))
        }
        // `nextAfter(float start, double direction)` returns a float.
        ("nextAfter", [JValue::Float(start), JValue::Double(direction)]) => {
            let (s, dir) = (*start, *direction);
            Ok(Some(JValue::Float(if s.is_nan() || dir.is_nan() {
                f32::NAN
            } else if f64::from(s) == dir {
                #[allow(clippy::cast_possible_truncation)]
                {
                    dir as f32
                }
            } else if dir > f64::from(s) {
                s.next_up()
            } else {
                s.next_down()
            })))
        }
        ("nextAfter", [JValue::Double(start), JValue::Double(direction)]) => {
            d(if start.is_nan() || direction.is_nan() {
                f64::NAN
            } else if start == direction {
                *direction
            } else if direction > start {
                start.next_up()
            } else {
                start.next_down()
            })
        }
        ("fma", [JValue::Double(a), JValue::Double(b), JValue::Double(c)]) => d(a.mul_add(*b, *c)),
        ("fma", [JValue::Float(a), JValue::Float(b), JValue::Float(c)]) => {
            Ok(Some(JValue::Float(a.mul_add(*b, *c))))
        }
        ("IEEEremainder", [JValue::Double(a), JValue::Double(b)]) => {
            // Per the spec's infinity cases: a NaN operand, an infinite
            // dividend, or a zero divisor gives NaN; a finite dividend with an
            // infinite divisor gives the DIVIDEND (not `a - 0*inf`, which is the
            // NaN caturra produced). Otherwise the IEEE remainder.
            d(
                if a.is_nan() || b.is_nan() || a.is_infinite() || *b == 0.0 {
                    f64::NAN
                } else if b.is_infinite() {
                    *a
                } else {
                    let quotient = (a / b).round_ties_even();
                    let remainder = a - quotient * b;
                    // IEEE 754: a ZERO result carries the DIVIDEND's sign, and
                    // `a - quotient * b` computes `+0.0` for every one of them.
                    // Losing that made `1.0 / IEEEremainder(-4.0, 2.0)` answer
                    // `Infinity` where a JDK answers `-Infinity`.
                    if remainder == 0.0 {
                        (0.0f64).copysign(*a)
                    } else {
                        remainder
                    }
                },
            )
        }
        ("getExponent", [JValue::Double(v)]) => {
            let bits = (v.to_bits() >> 52) & 0x7FF;
            i(if v.is_nan() || v.is_infinite() {
                1024
            } else if bits == 0 {
                -1023 // zero and subnormals
            } else {
                i32::try_from(bits).unwrap_or(0) - 1023
            })
        }
        // A `float`'s exponent lives in its own 8-bit field with bias 127.
        ("getExponent", [JValue::Float(v)]) => {
            let bits = (v.to_bits() >> 23) & 0xFF;
            i(if v.is_nan() || v.is_infinite() {
                128
            } else if bits == 0 {
                -127 // zero and subnormals
            } else {
                i32::try_from(bits).unwrap_or(0) - 127
            })
        }
        ("floorDiv", [JValue::Int(a), JValue::Int(b)]) => java_floor_div(*a, *b).and_then(i),
        ("floorMod", [JValue::Int(a), JValue::Int(b)]) => {
            let quotient = java_floor_div(*a, *b)?;
            i(a.wrapping_sub(quotient.wrapping_mul(*b)))
        }
        ("floorDiv", [JValue::Long(a), JValue::Long(b)]) => {
            java_floor_div_long(*a, *b).map(|q| Some(JValue::Long(q)))
        }
        ("floorMod", [JValue::Long(a), JValue::Long(b)]) => {
            let quotient = java_floor_div_long(*a, *b)?;
            Ok(Some(JValue::Long(
                a.wrapping_sub(quotient.wrapping_mul(*b)),
            )))
        }
        // `floorMod(long, int)` answers an INT: the result of a floor-mod by
        // an int always fits one.
        ("floorMod", [JValue::Long(a), JValue::Int(b)]) => {
            let quotient = java_floor_div_long(*a, i64::from(*b))?;
            let modulus = a.wrapping_sub(quotient.wrapping_mul(i64::from(*b)));
            i(i32::try_from(modulus).unwrap_or(0))
        }
        ("addExact", [JValue::Int(a), JValue::Int(b)]) => {
            a.checked_add(*b).map_or_else(|| Err(overflow()), i)
        }
        ("subtractExact", [JValue::Int(a), JValue::Int(b)]) => {
            a.checked_sub(*b).map_or_else(|| Err(overflow()), i)
        }
        ("multiplyExact", [JValue::Int(a), JValue::Int(b)]) => {
            a.checked_mul(*b).map_or_else(|| Err(overflow()), i)
        }
        ("negateExact", [JValue::Int(v)]) => v.checked_neg().map_or_else(|| Err(overflow()), i),
        ("incrementExact", [JValue::Int(v)]) => v.checked_add(1).map_or_else(|| Err(overflow()), i),
        ("decrementExact", [JValue::Int(v)]) => v.checked_sub(1).map_or_else(|| Err(overflow()), i),
        // The long overloads throw "long overflow" (the JDK's message differs
        // from the int ones' "integer overflow").
        ("addExact", [JValue::Long(a), JValue::Long(b)]) => a
            .checked_add(*b)
            .map_or_else(|| Err(long_overflow()), |v| Ok(Some(JValue::Long(v)))),
        ("subtractExact", [JValue::Long(a), JValue::Long(b)]) => a
            .checked_sub(*b)
            .map_or_else(|| Err(long_overflow()), |v| Ok(Some(JValue::Long(v)))),
        ("multiplyExact", [JValue::Long(a), JValue::Long(b)]) => a
            .checked_mul(*b)
            .map_or_else(|| Err(long_overflow()), |v| Ok(Some(JValue::Long(v)))),
        ("negateExact", [JValue::Long(v)]) => v
            .checked_neg()
            .map_or_else(|| Err(long_overflow()), |v| Ok(Some(JValue::Long(v)))),
        ("incrementExact", [JValue::Long(v)]) => v
            .checked_add(1)
            .map_or_else(|| Err(long_overflow()), |v| Ok(Some(JValue::Long(v)))),
        ("decrementExact", [JValue::Long(v)]) => v
            .checked_sub(1)
            .map_or_else(|| Err(long_overflow()), |v| Ok(Some(JValue::Long(v)))),
        _ => Err(VmError::UnknownIntrinsic(format!("Math.{method}"))),
    }
}

/// Java's `Integer.toString(int, radix)`: lowercase digits, radix
/// clamped to 10 when out of range.
fn int_to_string_radix(value: i32, radix: i32) -> String {
    let radix = if (2..=36).contains(&radix) { radix } else { 10 };
    let radix = u32::try_from(radix).expect("radix in range");
    let mut magnitude = i64::from(value).unsigned_abs();
    let mut digits = Vec::new();
    loop {
        let digit = u32::try_from(magnitude % u64::from(radix)).expect("digit < radix");
        digits.push(char::from_digit(digit, radix).expect("valid digit"));
        magnitude /= u64::from(radix);
        if magnitude == 0 {
            break;
        }
    }
    if value < 0 {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

/// `Long.toString(long, radix)`: lowercase digits, radix clamped to 10 when out
/// of range (as the JDK does).
fn long_to_string_radix(value: i64, radix: i32) -> String {
    let radix = if (2..=36).contains(&radix) { radix } else { 10 };
    let radix = u32::try_from(radix).expect("radix in range");
    let mut magnitude = value.unsigned_abs();
    let mut digits = Vec::new();
    loop {
        let digit = u32::try_from(magnitude % u64::from(radix)).expect("digit < radix");
        digits.push(char::from_digit(digit, radix).expect("valid digit"));
        magnitude /= u64::from(radix);
        if magnitude == 0 {
            break;
        }
    }
    if value < 0 {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

/// Validate a parse radix, throwing the JDK's `Character.MIN_RADIX`/`MAX_RADIX`
/// `NumberFormatException` (not the generic "For input string") when out of range.
fn checked_radix(radix: i32) -> Result<u32, VmError> {
    if radix < 2 {
        return Err(throw(format!(
            "java.lang.NumberFormatException: radix {radix} less than Character.MIN_RADIX"
        )));
    }
    if radix > 36 {
        return Err(throw(format!(
            "java.lang.NumberFormatException: radix {radix} greater than Character.MAX_RADIX"
        )));
    }
    Ok(u32::try_from(radix).expect("2..=36"))
}

/// `Integer.decode`/`Long.decode`: an optional sign, then `0x`/`0X`/`#` (hex),
/// a leading `0` (octal), or decimal. Returned as i64 for the caller to range.
fn decode_integer(text: &str) -> Result<i64, VmError> {
    // The JDK checks the length FIRST and says so in its own words, rather
    // than falling through to the "For input string" wording.
    if text.is_empty() {
        return Err(throw("java.lang.NumberFormatException: Zero length string"));
    }
    let (neg, body) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (radix, digits) =
        if let Some(hex) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
            (16, hex)
        } else if let Some(hex) = body.strip_prefix('#') {
            (16, hex)
        } else if let Some(oct) = body.strip_prefix('0').filter(|rest| !rest.is_empty()) {
            (8, oct)
        } else {
            (10, body)
        };
    // A sign AFTER the radix prefix is its own complaint, and `--1` was
    // accepted outright: the first `-` set `neg` and the second was read as
    // part of the number.
    if digits.starts_with('-') || digits.starts_with('+') {
        return Err(throw(
            "java.lang.NumberFormatException: Sign character in wrong position",
        ));
    }
    // The JDK strips the RADIX PREFIX before parsing but puts the sign back
    // when it reports, so `decode("0x")` names `""` while `decode("-1.5")`
    // names the whole `-1.5` — not the `1.5` a plain strip would leave.
    let magnitude = i64::from_str_radix(digits, radix).map_err(|_| {
        let sign = if neg { "-" } else { "" };
        throw(format!(
            "java.lang.NumberFormatException: For input string: \"{sign}{digits}\""
        ))
    })?;
    Ok(if neg { -magnitude } else { magnitude })
}

/// `Integer.parseUnsignedInt(s[, radix])`: a leading minus is its own error, a
/// value beyond `0xFFFF_FFFF` "exceeds range of unsigned int." (the JDK's own
/// wording), and the result is the bit-pattern stored signed.
fn parse_unsigned_int(text: &str, radix: u32) -> Result<Option<JValue>, VmError> {
    if text.starts_with('-') {
        return Err(throw(format!(
            "java.lang.NumberFormatException: Illegal leading minus sign \
             on unsigned string {text}."
        )));
    }
    match u64::from_str_radix(text, radix)
        .ok()
        .and_then(|v| u32::try_from(v).ok())
    {
        Some(value) => Ok(Some(JValue::Int(value.cast_signed()))),
        None if u64::from_str_radix(text, radix).is_ok() => Err(throw(format!(
            "java.lang.NumberFormatException: String value {text} exceeds range of unsigned int."
        ))),
        None => Err(number_format(text)),
    }
}

/// `Long.parseUnsignedLong(s[, radix])`: the 64-bit counterpart — a leading
/// minus is its own error; the full `u64` range is valid, stored signed.
fn parse_unsigned_long(text: &str, radix: u32) -> Result<Option<JValue>, VmError> {
    if text.starts_with('-') {
        return Err(throw(format!(
            "java.lang.NumberFormatException: Illegal leading minus sign \
             on unsigned string {text}."
        )));
    }
    u64::from_str_radix(text, radix).map_or_else(
        |_| Err(number_format(text)),
        |value| Ok(Some(JValue::Long(value.cast_signed()))),
    )
}

/// The text an integer parser works on: the digits FOLDED to ASCII so Rust's
/// ASCII-only parsers see what `Character.digit` would have given them, and
/// the string as WRITTEN, which is what the JDK quotes back. Reporting the
/// folded form said `For input string: "3"` about an input of `"\u0663"`.
struct NumberText {
    folded: String,
    raw: String,
}

impl std::ops::Deref for NumberText {
    type Target = str;

    fn deref(&self) -> &str {
        &self.folded
    }
}

impl std::fmt::Display for NumberText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.folded)
    }
}

fn parse_int_text(heap: &Heap, value: &JValue) -> Result<NumberText, VmError> {
    let raw = number_arg_text(heap, value)?;
    Ok(NumberText {
        folded: ascii_digits(&raw),
        raw,
    })
}

/// The argument text as WRITTEN, with the number parsers' shared handling of a
/// null or non-string argument.
///
/// The FLOAT parsers must use this rather than `parse_int_text`: only the
/// integer ones go through `Character.digit`, and `Double.parseDouble` reads
/// ASCII alone. Folding for them made `Double.parseDouble("\u0663")` answer
/// 3.0 where a JDK throws.
fn number_arg_text(heap: &Heap, value: &JValue) -> Result<String, VmError> {
    match value {
        JValue::Ref(Some(reference)) => heap
            .string_text(*reference)
            .ok_or_else(|| throw("java.lang.ClassCastException: not a String")),
        // JDK 11's message for a null string is literally "null".
        JValue::Ref(None) => Err(throw("java.lang.NumberFormatException: null")),
        _ => Err(throw("java.lang.VerifyError: expected a String argument")),
    }
}

fn number_format(text: &str) -> VmError {
    throw(format!(
        "java.lang.NumberFormatException: For input string: \"{text}\""
    ))
}

#[allow(clippy::too_many_lines)] // one arm per documented method
#[allow(clippy::many_single_char_names)]
fn integer_static(
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let i = |v: i32| Ok(Some(JValue::Int(v)));
    let s = |heap: &mut Heap, text: String| {
        let reference = heap.alloc_string(&text);
        Ok(Some(JValue::Ref(Some(reference))))
    };
    match (method, args) {
        ("parseInt" | "valueOf", [text @ JValue::Ref(_)]) => {
            // `valueOf` answers the WRAPPER OBJECT; only `parseX` answers the
            // primitive. Returning the primitive for both made two `valueOf`
            // results compare `==` by value.
            let parsed = (|| -> Result<Option<JValue>, VmError> {
                let text = parse_int_text(heap, text)?;
                text.parse()
                    .map_or_else(|_| Err(number_format(&text.raw)), i)
            })()?;
            if method == "valueOf"
                && let Some(value) = parsed
            {
                let reference = heap.box_wrapper("java/lang/Integer", value);
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            Ok(parsed)
        }
        ("parseInt" | "valueOf", [text @ JValue::Ref(_), JValue::Int(radix)]) => {
            // `valueOf` answers the WRAPPER OBJECT; only `parseX` answers the
            // primitive. Returning the primitive for both made two `valueOf`
            // results compare `==` by value.
            let parsed = (|| -> Result<Option<JValue>, VmError> {
                let text = parse_int_text(heap, text)?;
                let radix = checked_radix(*radix)?;
                i32::from_str_radix(&text, radix).map_or_else(|_| Err(number_format(&text.raw)), i)
            })()?;
            if method == "valueOf"
                && let Some(value) = parsed
            {
                let reference = heap.box_wrapper("java/lang/Integer", value);
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            Ok(parsed)
        }
        // `decode` answers an `Integer`, not an `int` — the compiler types it
        // as a wrapper and so does not box the result, and a raw `Int` left
        // where a reference belongs is a VerifyError the moment the value
        // reaches an `Object` (`Object o = Integer.decode("7")` then using
        // `o`, or a `Supplier<Object>` lambda returning one).
        ("decode", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            let value = decode_integer(&text)?;
            let value = i32::try_from(value).map_err(|_| number_format(&text.raw))?;
            let reference = heap.box_wrapper("java/lang/Integer", JValue::Int(value));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("parseUnsignedInt", [text @ JValue::Ref(_), JValue::Int(radix)]) => {
            let text = parse_int_text(heap, text)?;
            let radix = checked_radix(*radix)?;
            parse_unsigned_int(&text, radix)
        }
        ("parseUnsignedInt", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            parse_unsigned_int(&text, 10)
        }
        ("toString", [JValue::Int(v)]) => s(heap, v.to_string()),
        ("toString", [JValue::Int(v), JValue::Int(radix)]) => {
            s(heap, int_to_string_radix(*v, *radix))
        }
        ("toBinaryString", [JValue::Int(v)]) => s(heap, format!("{:b}", v.cast_unsigned())),
        ("toOctalString", [JValue::Int(v)]) => s(heap, format!("{:o}", v.cast_unsigned())),
        ("toHexString", [JValue::Int(v)]) => s(heap, format!("{:x}", v.cast_unsigned())),
        ("toUnsignedString", [JValue::Int(v)]) => s(heap, v.cast_unsigned().to_string()),
        ("toUnsignedString", [JValue::Int(v), JValue::Int(radix)]) => {
            let radix = if (2..=36).contains(radix) { *radix } else { 10 };
            let radix = u32::try_from(radix).expect("radix in range");
            let mut magnitude = v.cast_unsigned();
            let mut digits = Vec::new();
            loop {
                digits.push(char::from_digit(magnitude % radix, radix).expect("valid digit"));
                magnitude /= radix;
                if magnitude == 0 {
                    break;
                }
            }
            s(heap, digits.iter().rev().collect())
        }

        ("compare", [JValue::Int(a), JValue::Int(b)]) => i(match a.cmp(b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }),
        ("compareUnsigned", [JValue::Int(a), JValue::Int(b)]) => {
            i(match a.cast_unsigned().cmp(&b.cast_unsigned()) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })
        }
        ("max", [JValue::Int(a), JValue::Int(b)]) => i((*a).max(*b)),
        ("min", [JValue::Int(a), JValue::Int(b)]) => i((*a).min(*b)),
        ("sum", [JValue::Int(a), JValue::Int(b)]) => i(a.wrapping_add(*b)),
        // `Integer.valueOf(int)` answers a REFERENCE, through the autoboxing
        // cache, so `valueOf(100) == valueOf(100)` is true and `valueOf(200)`
        // is not. `hashCode` stays a plain int — they were one arm, which is
        // why valueOf handed back a bare primitive.
        ("valueOf", [JValue::Int(v)]) => {
            let reference = heap.box_wrapper("java/lang/Integer", JValue::Int(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("hashCode", [JValue::Int(v)]) => i(*v),
        ("signum", [JValue::Int(v)]) => i(v.signum()),
        ("bitCount", [JValue::Int(v)]) => i(i32::try_from(v.count_ones()).unwrap_or(0)),
        ("highestOneBit", [JValue::Int(v)]) => i(if *v == 0 {
            0
        } else {
            // Logical shift: the sign bit must not smear downward.
            ((1u32 << 31) >> v.leading_zeros()).cast_signed()
        }),
        ("lowestOneBit", [JValue::Int(v)]) => i(v.wrapping_neg() & v),
        ("numberOfLeadingZeros", [JValue::Int(v)]) => {
            i(i32::try_from(v.leading_zeros()).unwrap_or(32))
        }
        ("numberOfTrailingZeros", [JValue::Int(v)]) => {
            i(i32::try_from(v.trailing_zeros()).unwrap_or(32))
        }
        ("reverse", [JValue::Int(v)]) => i(v.reverse_bits()),
        ("reverseBytes", [JValue::Int(v)]) => i(v.swap_bytes()),
        ("rotateLeft", [JValue::Int(v), JValue::Int(n)]) => {
            i(v.rotate_left(n.cast_unsigned() % 32))
        }
        ("rotateRight", [JValue::Int(v), JValue::Int(n)]) => {
            i(v.rotate_right(n.cast_unsigned() % 32))
        }
        ("divideUnsigned", [JValue::Int(a), JValue::Int(b)]) => {
            if *b == 0 {
                return Err(throw("java.lang.ArithmeticException: / by zero"));
            }
            i((a.cast_unsigned() / b.cast_unsigned()).cast_signed())
        }
        ("remainderUnsigned", [JValue::Int(a), JValue::Int(b)]) => {
            if *b == 0 {
                return Err(throw("java.lang.ArithmeticException: / by zero"));
            }
            i((a.cast_unsigned() % b.cast_unsigned()).cast_signed())
        }
        // The int's 32 bits as an unsigned value in a long (no sign extension).
        ("toUnsignedLong", [JValue::Int(v)]) => {
            Ok(Some(JValue::Long(i64::from(v.cast_unsigned()))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Integer.{method}"))),
    }
}

/// Java's `Double.hashCode`: fold the canonical bit pattern.
pub(crate) fn java_double_hash_public(v: f64) -> i32 {
    java_double_hash(v)
}

fn java_double_hash(v: f64) -> i32 {
    let bits = if v.is_nan() {
        0x7FF8_0000_0000_0000_u64
    } else {
        v.to_bits()
    };
    (((bits ^ (bits >> 32)) & 0xFFFF_FFFF) as u32).cast_signed()
}

/// Java's `Double.toHexString`.
pub(crate) fn java_double_to_hex(v: f64) -> String {
    if v.is_nan() {
        return String::from("NaN");
    }
    if v.is_infinite() {
        return String::from(if v > 0.0 { "Infinity" } else { "-Infinity" });
    }
    let sign = if v.is_sign_negative() { "-" } else { "" };
    let bits = v.to_bits();
    let exponent = (bits >> 52) & 0x7FF;
    let mantissa = bits & 0x000F_FFFF_FFFF_FFFF;
    if exponent == 0 {
        if mantissa == 0 {
            return format!("{sign}0x0.0p0");
        }
        // Subnormal.
        let mut hex = format!("{mantissa:013x}");
        while hex.len() > 1 && hex.ends_with('0') {
            hex.pop();
        }
        return format!("{sign}0x0.{hex}p-1022");
    }
    let mut hex = format!("{mantissa:013x}");
    while hex.len() > 1 && hex.ends_with('0') {
        hex.pop();
    }
    let unbiased = i64::try_from(exponent).unwrap_or(0) - 1023;
    format!("{sign}0x1.{hex}p{unbiased}")
}

/// Strip an optional trailing Java float/double type suffix (`f`/`F`/`d`/`D`),
/// legal on any numeric floating string — `"1.0f"`, `"3.14d"`, `"0x1p4d"`.
fn strip_float_suffix(s: &str) -> &str {
    match s.chars().last() {
        Some('f' | 'F' | 'd' | 'D') if s.len() > 1 => &s[..s.len() - 1],
        _ => s,
    }
}

/// Parse a hexadecimal floating-point string (`0x1.8p1` = 3.0), the form Rust's
/// float parser rejects but Java's `parseDouble`/`parseFloat` grammar accepts.
/// The `p`/`P` binary exponent is mandatory (as in Java).
fn parse_hex_float(s: &str) -> Option<f64> {
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let rest = rest
        .strip_prefix("0x")
        .or_else(|| rest.strip_prefix("0X"))?;
    let (mantissa, exponent) = rest.split_once(['p', 'P'])?;
    let exponent: i32 = exponent.parse().ok()?;
    let (int_part, frac_part) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let mut value = 0f64;
    for c in int_part.chars() {
        value = value * 16.0 + f64::from(c.to_digit(16)?);
    }
    let mut scale = 1.0 / 16.0;
    for c in frac_part.chars() {
        value += f64::from(c.to_digit(16)?) * scale;
        scale /= 16.0;
    }
    value *= 2f64.powi(exponent);
    Some(if neg { -value } else { value })
}

/// Java's grammar admits exactly two WORD forms — `NaN` and `Infinity`, with
/// an optional sign and no type suffix. Rust's parser also takes `inf`,
/// `infinity` and `nan` in any case, and would take `NaNd` once the suffix
/// is stripped, all of which Java rejects. `None` when the text is not a
/// word form at all; `Some(None)` when it is one Java does not accept.
#[allow(clippy::option_option)] // "not a word" and "a word Java rejects" differ
fn java_float_word(core: &str) -> Option<Option<f64>> {
    let (negative, unsigned) = match core.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, core.strip_prefix('+').unwrap_or(core)),
    };
    if !unsigned
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic())
    {
        return None;
    }
    Some(match unsigned {
        "NaN" => Some(f64::NAN),
        "Infinity" => Some(if negative {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }),
        _ => None,
    })
}

/// Parse per Java's `Double.parseDouble` grammar — a trailing type suffix and
/// the hexadecimal form, both of which Rust's `str::parse` rejects.
fn parse_java_double(trimmed: &str) -> Option<f64> {
    if let Some(word) = java_float_word(trimmed) {
        return word;
    }
    let core = strip_float_suffix(trimmed);
    let unsigned = core.trim_start_matches(['+', '-']);
    if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        return parse_hex_float(core);
    }
    core.parse().ok()
}

/// `Float.parseFloat`'s grammar — the same suffix and hex forms, rounded to a
/// float.
#[allow(clippy::cast_possible_truncation)]
fn parse_java_float(trimmed: &str) -> Option<f32> {
    if let Some(word) = java_float_word(trimmed) {
        return word.map(|value| value as f32);
    }
    let core = strip_float_suffix(trimmed);
    let unsigned = core.trim_start_matches(['+', '-']);
    if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        return parse_hex_float(core).map(|value| value as f32);
    }
    core.parse().ok()
}

#[allow(clippy::float_cmp)]
fn double_static(
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let d = |v: f64| Ok(Some(JValue::Double(v)));
    let z = |v: bool| Ok(Some(JValue::Int(i32::from(v))));
    match (method, args) {
        ("parseDouble" | "valueOf", [text @ JValue::Ref(_)]) => {
            // `valueOf` answers the WRAPPER OBJECT; only `parseX` answers the
            // primitive. Returning the primitive for both made two `valueOf`
            // results compare `==` by value.
            let parsed = (|| -> Result<Option<JValue>, VmError> {
                // JDK 11 `parseDouble(null)` is a NullPointerException (it reads
                // the null string's length), not a NumberFormatException.
                if matches!(text, JValue::Ref(None)) {
                    return Err(throw("java.lang.NullPointerException"));
                }
                let text = number_arg_text(heap, text)?;
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    return Err(throw("java.lang.NumberFormatException: empty String"));
                }
                parse_java_double(trimmed).map_or_else(|| Err(number_format(&text)), d)
            })()?;
            if method == "valueOf"
                && let Some(value) = parsed
            {
                let reference = heap.box_wrapper("java/lang/Double", value);
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            Ok(parsed)
        }
        ("toString", [JValue::Double(v)]) => {
            let reference = heap.alloc_string(&java_double_to_string(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("toHexString", [JValue::Double(v)]) => {
            let reference = heap.alloc_string(&java_double_to_hex(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // `valueOf` answers the WRAPPER OBJECT — a boxed value with its own
        // identity — not the bare primitive. Returning the primitive made
        // `Double.valueOf(1.0) == Double.valueOf(1.0)` compare by value and
        // answer true, where a JDK's two distinct objects answer false.
        ("valueOf", [JValue::Double(v)]) => {
            let reference = heap.box_wrapper("java/lang/Double", JValue::Double(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("isNaN", [JValue::Double(v)]) => z(v.is_nan()),
        ("isInfinite", [JValue::Double(v)]) => z(v.is_infinite()),
        ("isFinite", [JValue::Double(v)]) => z(v.is_finite()),
        ("compare", [JValue::Double(a), JValue::Double(b)]) => {
            // Java: -0.0 < 0.0 and NaN is the largest.
            let ordering = if *a < *b {
                -1
            } else if *a > *b {
                1
            } else {
                let a_bits = if a.is_nan() {
                    0x7FF8_0000_0000_0000_u64
                } else {
                    a.to_bits()
                };
                let b_bits = if b.is_nan() {
                    0x7FF8_0000_0000_0000_u64
                } else {
                    b.to_bits()
                };
                match a_bits.cast_signed().cmp(&b_bits.cast_signed()) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }
            };
            Ok(Some(JValue::Int(ordering)))
        }
        ("max", [JValue::Double(a), JValue::Double(b)]) => d(java_double_max(*a, *b)),
        ("min", [JValue::Double(a), JValue::Double(b)]) => d(java_double_min(*a, *b)),
        ("sum", [JValue::Double(a), JValue::Double(b)]) => d(a + b),
        ("hashCode", [JValue::Double(v)]) => Ok(Some(JValue::Int(java_double_hash(*v)))),
        ("doubleToLongBits", [JValue::Double(v)]) => {
            let bits = if v.is_nan() {
                0x7FF8_0000_0000_0000_u64
            } else {
                v.to_bits()
            };
            Ok(Some(JValue::Long(bits.cast_signed())))
        }
        ("doubleToRawLongBits", [JValue::Double(v)]) => {
            Ok(Some(JValue::Long(v.to_bits().cast_signed())))
        }
        ("longBitsToDouble", [JValue::Long(v)]) => {
            Ok(Some(JValue::Double(f64::from_bits(v.cast_unsigned()))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Double.{method}"))),
    }
}

/// Java's `Character.isWhitespace` (NOT Unicode `White_Space`: excludes
/// the no-break spaces, includes the ASCII separators).
fn java_is_whitespace(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\x0B' | '\x0C' | '\r' | '\x1C'..='\x1F')
        || (java_is_space_char(c) && !matches!(c, '\u{00A0}' | '\u{2007}' | '\u{202F}'))
}

/// Java's `Character.isSpaceChar` (Unicode space separators).
fn java_is_space_char(c: char) -> bool {
    matches!(
        c,
        ' ' | '\u{00A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{2028}' | '\u{2029}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

/// Start code point of every Unicode decimal-digit (category `Nd`) run in the
/// BMP, one per script. Each run is exactly ten contiguous code points valued
/// 0..=9 (Unicode 10.0, matching JDK 11). Extracted directly from a real JVM.
const ND_STARTS: [u32; 37] = [
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
    0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10,
];

/// Every BMP character whose `Character.getNumericValue` is not simply its
/// `digit(c, 36)` value, as runs of `(first, last, value_at_first)` — Roman
/// numerals, Ethiopic and Tamil numbers, superscripts, and the fractions
/// (which answer -2: "a numeric value that is not a nonnegative integer").
/// Generated from JDK 11 itself, so it matches the Unicode version caturra
/// targets rather than a newer one.
const NUMERIC_RUNS: &[(u32, u32, i32)] = &[
    (0x00B2, 0x00B3, 2),
    (0x00B9, 0x00B9, 1),
    (0x00BC, 0x00BC, -2),
    (0x00BD, 0x00BD, -2),
    (0x00BE, 0x00BE, -2),
    (0x09F4, 0x09F4, -2),
    (0x09F5, 0x09F5, -2),
    (0x09F6, 0x09F6, -2),
    (0x09F7, 0x09F7, -2),
    (0x09F8, 0x09F8, -2),
    (0x09F9, 0x09F9, 16),
    (0x0B72, 0x0B72, -2),
    (0x0B73, 0x0B73, -2),
    (0x0B74, 0x0B74, -2),
    (0x0B75, 0x0B75, -2),
    (0x0B76, 0x0B76, -2),
    (0x0B77, 0x0B77, -2),
    (0x0BF0, 0x0BF0, 10),
    (0x0BF1, 0x0BF1, 100),
    (0x0BF2, 0x0BF2, 1000),
    (0x0C78, 0x0C7B, 0),
    (0x0C7C, 0x0C7E, 1),
    (0x0D58, 0x0D58, -2),
    (0x0D59, 0x0D59, -2),
    (0x0D5A, 0x0D5A, -2),
    (0x0D5B, 0x0D5B, -2),
    (0x0D5C, 0x0D5C, -2),
    (0x0D5D, 0x0D5D, -2),
    (0x0D5E, 0x0D5E, -2),
    (0x0D70, 0x0D70, 10),
    (0x0D71, 0x0D71, 100),
    (0x0D72, 0x0D72, 1000),
    (0x0D73, 0x0D73, -2),
    (0x0D74, 0x0D74, -2),
    (0x0D75, 0x0D75, -2),
    (0x0D76, 0x0D76, -2),
    (0x0D77, 0x0D77, -2),
    (0x0D78, 0x0D78, -2),
    (0x0F2A, 0x0F2A, -2),
    (0x0F2B, 0x0F2B, -2),
    (0x0F2C, 0x0F2C, -2),
    (0x0F2D, 0x0F2D, -2),
    (0x0F2E, 0x0F2E, -2),
    (0x0F2F, 0x0F2F, -2),
    (0x0F30, 0x0F30, -2),
    (0x0F31, 0x0F31, -2),
    (0x0F32, 0x0F32, -2),
    (0x0F33, 0x0F33, -2),
    (0x1369, 0x1372, 1),
    (0x1373, 0x1373, 20),
    (0x1374, 0x1374, 30),
    (0x1375, 0x1375, 40),
    (0x1376, 0x1376, 50),
    (0x1377, 0x1377, 60),
    (0x1378, 0x1378, 70),
    (0x1379, 0x1379, 80),
    (0x137A, 0x137A, 90),
    (0x137B, 0x137B, 100),
    (0x137C, 0x137C, 10_000),
    (0x16EE, 0x16F0, 17),
    (0x17F0, 0x17F9, 0),
    (0x19DA, 0x19DA, 1),
    (0x2070, 0x2070, 0),
    (0x2074, 0x2079, 4),
    (0x2080, 0x2089, 0),
    (0x2150, 0x2150, -2),
    (0x2151, 0x2151, -2),
    (0x2152, 0x2152, -2),
    (0x2153, 0x2153, -2),
    (0x2154, 0x2154, -2),
    (0x2155, 0x2155, -2),
    (0x2156, 0x2156, -2),
    (0x2157, 0x2157, -2),
    (0x2158, 0x2158, -2),
    (0x2159, 0x2159, -2),
    (0x215A, 0x215A, -2),
    (0x215B, 0x215B, -2),
    (0x215C, 0x215C, -2),
    (0x215D, 0x215D, -2),
    (0x215E, 0x215E, -2),
    (0x215F, 0x215F, 1),
    (0x2160, 0x216B, 1),
    (0x216C, 0x216C, 50),
    (0x216D, 0x216D, 100),
    (0x216E, 0x216E, 500),
    (0x216F, 0x216F, 1000),
    (0x2170, 0x217B, 1),
    (0x217C, 0x217C, 50),
    (0x217D, 0x217D, 100),
    (0x217E, 0x217E, 500),
    (0x217F, 0x217F, 1000),
    (0x2180, 0x2180, 1000),
    (0x2181, 0x2181, 5000),
    (0x2182, 0x2182, 10_000),
    (0x2185, 0x2185, 6),
    (0x2186, 0x2186, 50),
    (0x2187, 0x2187, 50_000),
    (0x2188, 0x2188, 100_000),
    (0x2189, 0x2189, 0),
    (0x2460, 0x2473, 1),
    (0x2474, 0x2487, 1),
    (0x2488, 0x249B, 1),
    (0x24EA, 0x24EA, 0),
    (0x24EB, 0x24F4, 11),
    (0x24F5, 0x24FE, 1),
    (0x24FF, 0x24FF, 0),
    (0x2776, 0x277F, 1),
    (0x2780, 0x2789, 1),
    (0x278A, 0x2793, 1),
    (0x2CFD, 0x2CFD, -2),
    (0x3007, 0x3007, 0),
    (0x3021, 0x3029, 1),
    (0x3038, 0x3038, 10),
    (0x3039, 0x3039, 20),
    (0x303A, 0x303A, 30),
    (0x3192, 0x3195, 1),
    (0x3220, 0x3229, 1),
    (0x3248, 0x3248, 10),
    (0x3249, 0x3249, 20),
    (0x324A, 0x324A, 30),
    (0x324B, 0x324B, 40),
    (0x324C, 0x324C, 50),
    (0x324D, 0x324D, 60),
    (0x324E, 0x324E, 70),
    (0x324F, 0x324F, 80),
    (0x3251, 0x325F, 21),
    (0x3280, 0x3289, 1),
    (0x32B1, 0x32BF, 36),
    (0xA6E6, 0xA6EE, 1),
    (0xA6EF, 0xA6EF, 0),
    (0xA830, 0xA830, -2),
    (0xA831, 0xA831, -2),
    (0xA832, 0xA832, -2),
    (0xA833, 0xA833, -2),
    (0xA834, 0xA834, -2),
    (0xA835, 0xA835, -2),
    (0xF96B, 0xF96B, 3),
    (0xF973, 0xF973, 10),
    (0xF978, 0xF978, 2),
    (0xF9B2, 0xF9B2, 0),
    (0xF9D1, 0xF9D1, 6),
    (0xF9D3, 0xF9D3, 6),
    (0xF9FD, 0xF9FD, 10),
];

/// `Character.getNumericValue(c)` for a character the digit tables do not
/// cover; `None` when the character has no numeric value.
fn numeric_run_value(c: char) -> Option<i32> {
    let cp = u32::from(c);
    NUMERIC_RUNS.iter().find_map(|&(first, last, base)| {
        (cp >= first && cp <= last).then(|| {
            if base < 0 {
                base
            } else {
                base + i32::try_from(cp - first).unwrap_or(0)
            }
        })
    })
}

/// Java's `Character.toTitleCase(char)`: the uppercase mapping, except for
/// Rewrite every Unicode decimal digit as its ASCII counterpart, so the
/// integer parsers (which are ASCII-only) see what `Character.digit` would
/// have given them: `Integer.parseInt("\u0663\u0664")` is 34 on a JDK.
fn ascii_digits(text: &str) -> String {
    text.chars()
        .map(|c| {
            nd_digit_value(c)
                .and_then(|value| char::from_digit(value, 10))
                .unwrap_or(c)
        })
        .collect()
}

/// Java's Unicode decimal-digit value (category `Nd`): 0..=9, or `None`.
/// Rust's `char::to_digit` is ASCII-only, so Arabic-Indic '٠', fullwidth '０',
/// Devanagari '५' etc. all need this table.
fn nd_digit_value(c: char) -> Option<u32> {
    let cp = u32::from(c);
    // `then`, not `then_some`: the latter evaluates its argument eagerly, so
    // `cp - s` underflowed for every start above `cp` — a debug-build panic on
    // something as ordinary as `Character.isDigit('A')`. Release builds wrapped
    // instead and the range test still rejected the value, which is how a
    // panic on a common call went unnoticed.
    ND_STARTS
        .iter()
        .find_map(|&s| (cp >= s && cp <= s + 9).then(|| cp - s))
}

/// Java's `Character.toUpperCase(char)`: the *simple* (single-char) uppercase
/// mapping. Rust's `char::to_uppercase` is the *full* mapping, multi-char for
/// ~100 BMP chars. Where the full mapping is one char it equals the simple one;
/// where it is multi-char the simple mapping is either the char itself (ß,
/// ligatures) or, for 27 polytonic-Greek letters with ypogegrammeni, a specific
/// char in the 1F88.. titlecase block.
#[allow(clippy::too_many_lines)] // one arm per documented method
#[allow(clippy::many_single_char_names)]
fn character_static(
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let z = |v: bool| Ok(Some(JValue::Int(i32::from(v))));
    let c_of = |unit: &i32| char::from_u32(u32::try_from(*unit).unwrap_or(0)).unwrap_or('\u{FFFD}');
    // The raw UTF-16 unit, which the case tables are keyed by — a surrogate
    // has a mapping too (itself), and `c_of` cannot hold one.
    let unit_of = |unit: &i32| u16::try_from(*unit).unwrap_or(u16::MAX);
    // The CODE POINT, for the predicates that take one: `Character.isLetter`
    // has an `int` overload, and it is asked about the whole space.
    let point_of = |point: &i32| u32::try_from(*point).unwrap_or(u32::MAX);
    let ch_ret = |c: char| {
        Ok(Some(JValue::Int(
            i32::try_from(u32::from(c) & 0xFFFF).unwrap_or(0),
        )))
    };
    match (method, args) {
        // `reverseBytes(char)`: swap the two bytes of the UTF-16 unit.
        ("reverseBytes", [JValue::Int(unit)]) => {
            let value = u16::try_from(*unit & 0xFFFF).unwrap_or(0);
            Ok(Some(JValue::Int(i32::from(value.swap_bytes()))))
        }
        // `codePointCount(CharSequence, begin, end)`: surrogate PAIRS count
        // once, so it is not simply `end - begin`.
        ("codePointCount", [text, JValue::Int(begin), JValue::Int(end)]) => {
            let units = match text {
                JValue::Ref(Some(reference)) => match heap.get(*reference) {
                    Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                        units.clone()
                    }
                    _ => return Err(throw("java.lang.ClassCastException: not a CharSequence")),
                },
                _ => return Err(throw("java.lang.NullPointerException")),
            };
            code_point_count(&units, *begin, *end).map(|n| Some(JValue::Int(n)))
        }
        // The CODE POINT half. `String` already answers these about itself;
        // `Character` answers them about any `CharSequence`, with the same
        // rules underneath.
        ("codePointAt", [text, JValue::Int(at)]) => {
            let units = code_point_source(heap, text)?;
            code_point_at(&units, *at).map(|point| Some(JValue::Int(point)))
        }
        ("codePointBefore", [text, JValue::Int(at)]) => {
            let units = code_point_source(heap, text)?;
            code_point_before(&units, *at).map(|point| Some(JValue::Int(point)))
        }
        ("offsetByCodePoints", [text, JValue::Int(index), JValue::Int(offset)]) => {
            let units = code_point_source(heap, text)?;
            offset_by_code_points(&units, *index, *offset).map(|at| Some(JValue::Int(at)))
        }
        ("isValidCodePoint", [JValue::Int(v)]) => z((0..=0x10_FFFF).contains(v)),
        ("isBmpCodePoint", [JValue::Int(v)]) => z((0..=0xFFFF).contains(v)),
        ("isSupplementaryCodePoint", [JValue::Int(v)]) => z((0x1_0000..=0x10_FFFF).contains(v)),
        // The two halves of the pair a supplementary code point is written
        // as. A JDK does not check the range first: the arithmetic is the
        // whole method.
        ("highSurrogate", [JValue::Int(v)]) => Ok(Some(JValue::Int(
            0xD800 + ((v.wrapping_sub(0x1_0000) >> 10) & 0x3FF),
        ))),
        ("lowSurrogate", [JValue::Int(v)]) => Ok(Some(JValue::Int(0xDC00 + (v & 0x3FF)))),
        ("isSurrogatePair", [JValue::Int(high), JValue::Int(low)]) => {
            z((0xD800..0xDC00).contains(high) && (0xDC00..0xE000).contains(low))
        }
        // Deprecated in Java 1.1 and still there in 11: `isJavaLetter` and
        // `isJavaLetterOrDigit` are the old spellings of the identifier
        // predicates below, and `isSpace` is five ASCII characters.
        ("isSpace", [JValue::Int(v)]) => z(matches!(*v, 0x20 | 0x09 | 0x0A | 0x0C | 0x0D)),
        // These come from the JDK 11 category table rather than from Rust's
        // Unicode data, which is a NEWER version and disagreed on 4761 BMP
        // units. See `crate::unicode`.
        ("isDigit", [JValue::Int(v)]) => z(unicode::is_digit(point_of(v))),
        ("isLetterOrDigit", [JValue::Int(v)]) => {
            let unit = point_of(v);
            z(unicode::is_letter(unit) || unicode::is_digit(unit))
        }
        // `isLetter` is the L* categories; `isAlphabetic` is L* plus
        // LETTER_NUMBER (Roman numerals and the CJK number letters) and the
        // `Other_Alphabetic` marks, which is where the two part company.
        ("isLetter", [JValue::Int(v)]) => z(unicode::is_letter(point_of(v))),
        ("isAlphabetic", [JValue::Int(v)]) => z(unicode::is_alphabetic(point_of(v))),
        ("isUpperCase", [JValue::Int(v)]) => z(unicode::is_upper(point_of(v))),
        ("isLowerCase", [JValue::Int(v)]) => z(unicode::is_lower(point_of(v))),
        ("isWhitespace", [JValue::Int(v)]) => z(unicode::is_whitespace(point_of(v))),
        ("isSpaceChar", [JValue::Int(v)]) => z(unicode::is_space_char(point_of(v))),
        ("getType", [JValue::Int(v)]) => Ok(Some(JValue::Int(i32::from(unicode::category_of(
            point_of(v),
        ))))),
        // `$` and `_` are not a special case: they are a CURRENCY symbol and
        // a CONNECTING punctuation, two whole categories that start an
        // identifier. Reading the rule as "alphabetic, or one of those two
        // characters" answered wrongly for every other currency sign.
        ("isJavaIdentifierStart" | "isJavaLetter", [JValue::Int(v)]) => {
            z(unicode::is_java_identifier_start(point_of(v)))
        }
        ("isJavaIdentifierPart" | "isJavaLetterOrDigit", [JValue::Int(v)]) => {
            z(unicode::is_java_identifier_part(point_of(v)))
        }
        ("isUnicodeIdentifierStart", [JValue::Int(v)]) => {
            z(unicode::is_unicode_identifier_start(point_of(v)))
        }
        ("isUnicodeIdentifierPart", [JValue::Int(v)]) => {
            z(unicode::is_unicode_identifier_part(point_of(v)))
        }
        ("isIdentifierIgnorable", [JValue::Int(v)]) => {
            z(unicode::is_identifier_ignorable(point_of(v)))
        }
        ("isDefined", [JValue::Int(v)]) => z(unicode::is_defined(point_of(v))),
        ("isISOControl", [JValue::Int(v)]) => z(unicode::is_iso_control(point_of(v))),
        ("isMirrored", [JValue::Int(v)]) => z(unicode::is_mirrored(point_of(v))),
        ("isIdeographic", [JValue::Int(v)]) => z(unicode::is_ideographic(point_of(v))),
        ("isTitleCase", [JValue::Int(v)]) => z(unicode::is_title_case(point_of(v))),
        // A lone surrogate has no case mapping and comes back UNCHANGED.
        // `c_of` cannot hold one, so it hands over the replacement character
        // and these used to answer U+FFFD for all 2048 of them.
        ("toTitleCase" | "toUpperCase" | "toLowerCase", [JValue::Int(v)])
            if (0xD800..=0xDFFF).contains(v) =>
        {
            Ok(Some(JValue::Int(*v)))
        }
        // These take a CODE POINT in their `int` form, and the supplementary
        // scripts with a case (Deseret and four others) map like any other.
        ("toTitleCase", [JValue::Int(v)]) => Ok(Some(JValue::Int(
            i32::try_from(unicode::title_point(point_of(v))).unwrap_or(*v),
        ))),
        ("toUpperCase", [JValue::Int(v)]) => Ok(Some(JValue::Int(
            i32::try_from(unicode::upper_point(point_of(v))).unwrap_or(*v),
        ))),
        ("toLowerCase", [JValue::Int(v)]) => Ok(Some(JValue::Int(
            i32::try_from(unicode::lower_point(point_of(v))).unwrap_or(*v),
        ))),
        ("getNumericValue", [JValue::Int(v)]) => {
            let c = c_of(v);
            // The Nd decimal value (0..=9) or a Latin letter's 10..=35 —
            // then the letter-number/other-number table, which carries the
            // Roman numerals, the superscripts, and the fractions' -2.
            let value = nd_digit_value(c)
                .or_else(|| c.to_digit(36))
                .and_then(|d| i32::try_from(d).ok())
                .or_else(|| numeric_run_value(c))
                .unwrap_or(-1);
            Ok(Some(JValue::Int(value)))
        }
        ("digit", [JValue::Int(v), JValue::Int(radix)]) => {
            let c = c_of(v);
            let value = u32::try_from(*radix)
                .ok()
                .filter(|r| (2..=36).contains(r))
                .and_then(|r| {
                    nd_digit_value(c)
                        .filter(|&d| d < r)
                        .or_else(|| c.to_digit(r))
                })
                .and_then(|d| i32::try_from(d).ok())
                .unwrap_or(-1);
            Ok(Some(JValue::Int(value)))
        }
        ("forDigit", [JValue::Int(digit), JValue::Int(radix)]) => {
            let c = u32::try_from(*radix)
                .ok()
                .filter(|r| (2..=36).contains(r))
                .zip(u32::try_from(*digit).ok())
                .filter(|(r, d)| d < r)
                .and_then(|(r, d)| char::from_digit(d, r))
                .unwrap_or('\0');
            ch_ret(c)
        }
        ("compare", [JValue::Int(a), JValue::Int(b)]) => Ok(Some(JValue::Int(a - b))),

        // The UNITS, not a rendered `char`: `Character.toString('\uD83D')` is
        // a one-unit string holding that surrogate. The `int` overload takes a
        // CODE POINT, so a supplementary one becomes its surrogate PAIR.
        ("toString", [JValue::Int(v)]) => {
            let units = match u32::try_from(*v) {
                Ok(point @ 0x1_0000..=0x10_FFFF) => {
                    let offset = point - 0x1_0000;
                    vec![
                        u16::try_from(0xD800 + (offset >> 10)).unwrap_or(u16::MAX),
                        u16::try_from(0xDC00 + (offset & 0x3FF)).unwrap_or(u16::MAX),
                    ]
                }
                _ => vec![unit_of(v)],
            };
            let reference = heap.alloc(HeapObject::JavaString(units));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("hashCode", [JValue::Int(v)]) => Ok(Some(JValue::Int(*v))),
        ("valueOf", [JValue::Int(v)]) => {
            let reference = heap.box_wrapper("java/lang/Character", JValue::Int(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("isHighSurrogate", [JValue::Int(v)]) => z((0xD800..0xDC00).contains(v)),
        ("isLowSurrogate", [JValue::Int(v)]) => z((0xDC00..0xE000).contains(v)),
        ("isSurrogate", [JValue::Int(v)]) => z((0xD800..0xE000).contains(v)),
        ("charCount", [JValue::Int(v)]) => Ok(Some(JValue::Int(if *v >= 0x10000 { 2 } else { 1 }))),
        // `toChars(codePoint)` — one unit in the BMP, a surrogate pair above
        // it. `char[]` lives in an IntArray, so an unpaired half survives.
        ("toChars", [JValue::Int(code_point)]) => {
            let units = code_point_units(*code_point)?;
            let values: Vec<i32> = units.iter().map(|u| i32::from(*u)).collect();
            let array = heap.alloc(HeapObject::IntArray(IntKind::Char, values));
            Ok(Some(JValue::Ref(Some(array))))
        }
        // `toCodePoint(high, low)` — the JDK does NOT validate the halves,
        // it just composes them.
        ("toCodePoint", [JValue::Int(high), JValue::Int(low)]) => {
            let composed = ((high - 0xD800) << 10) + (low - 0xDC00) + 0x10000;
            Ok(Some(JValue::Int(composed)))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Character.{method}"))),
    }
}

fn boolean_static(
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let z = |v: bool| Ok(Some(JValue::Int(i32::from(v))));
    match (method, args) {
        // `valueOf(String)` is `parseBoolean`'s answer (boxed on a JDK, a
        // plain boolean here) — anything but "true", in any case, is false.
        ("parseBoolean" | "valueOf", [text @ JValue::Ref(_)]) => {
            // `valueOf` answers the WRAPPER OBJECT; only `parseX` answers the
            // primitive. Returning the primitive for both made two `valueOf`
            // results compare `==` by value.
            let parsed = (|| -> Result<Option<JValue>, VmError> {
                let text = parse_int_text(heap, text)?;
                z(text.eq_ignore_ascii_case("true"))
            })()?;
            if method == "valueOf"
                && let Some(value) = parsed
            {
                let reference = heap.box_wrapper("java/lang/Boolean", value);
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            Ok(parsed)
        }
        ("toString", [JValue::Int(v)]) => {
            let reference = heap.alloc_string(if *v != 0 { "true" } else { "false" });
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("valueOf", [JValue::Int(v)]) => {
            // `Boolean.valueOf` answers one of the two CACHED objects, so two
            // calls with the same value really are identical.
            let reference = heap.box_wrapper("java/lang/Boolean", JValue::Int(i32::from(*v != 0)));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("compare", [JValue::Int(a), JValue::Int(b)]) => {
            Ok(Some(JValue::Int(if (*a != 0) == (*b != 0) {
                0
            } else if *a != 0 {
                1
            } else {
                -1
            })))
        }
        // Java's fixed Boolean hash codes.
        ("hashCode", [JValue::Int(v)]) => Ok(Some(JValue::Int(if *v != 0 { 1231 } else { 1237 }))),
        ("logicalAnd", [JValue::Int(a), JValue::Int(b)]) => z(*a != 0 && *b != 0),
        ("logicalOr", [JValue::Int(a), JValue::Int(b)]) => z(*a != 0 || *b != 0),
        ("logicalXor", [JValue::Int(a), JValue::Int(b)]) => z((*a != 0) != (*b != 0)),
        _ => Err(VmError::UnknownIntrinsic(format!("Boolean.{method}"))),
    }
}

#[allow(clippy::too_many_lines, clippy::many_single_char_names)] // one arm per documented method
fn long_static(heap: &mut Heap, method: &str, args: &[JValue]) -> Result<Option<JValue>, VmError> {
    let l = |v: i64| Ok(Some(JValue::Long(v)));
    let i = |v: i32| Ok(Some(JValue::Int(v)));
    let s = |heap: &mut Heap, text: String| {
        let reference = heap.alloc_string(&text);
        Ok(Some(JValue::Ref(Some(reference))))
    };
    match (method, args) {
        ("parseLong" | "valueOf", [text @ JValue::Ref(_)]) => {
            // `valueOf` answers the WRAPPER OBJECT; only `parseX` answers the
            // primitive. Returning the primitive for both made two `valueOf`
            // results compare `==` by value.
            let parsed = (|| -> Result<Option<JValue>, VmError> {
                let text = parse_int_text(heap, text)?;
                text.parse()
                    .map_or_else(|_| Err(number_format(&text.raw)), l)
            })()?;
            if method == "valueOf"
                && let Some(value) = parsed
            {
                let reference = heap.box_wrapper("java/lang/Long", value);
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            Ok(parsed)
        }
        ("parseLong" | "valueOf", [text @ JValue::Ref(_), JValue::Int(radix)]) => {
            // `valueOf` answers the WRAPPER OBJECT; only `parseX` answers the
            // primitive. Returning the primitive for both made two `valueOf`
            // results compare `==` by value.
            let parsed = (|| -> Result<Option<JValue>, VmError> {
                let text = parse_int_text(heap, text)?;
                let radix = checked_radix(*radix)?;
                i64::from_str_radix(&text, radix).map_or_else(|_| Err(number_format(&text.raw)), l)
            })()?;
            if method == "valueOf"
                && let Some(value) = parsed
            {
                let reference = heap.box_wrapper("java/lang/Long", value);
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            Ok(parsed)
        }
        ("parseUnsignedLong", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            parse_unsigned_long(&text, 10)
        }
        ("parseUnsignedLong", [text @ JValue::Ref(_), JValue::Int(radix)]) => {
            let text = parse_int_text(heap, text)?;
            let radix = checked_radix(*radix)?;
            parse_unsigned_long(&text, radix)
        }
        // ...and `Long.decode` answers a `Long`, for the same reason.
        ("decode", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            let value = decode_integer(&text)?;
            let reference = heap.box_wrapper("java/lang/Long", JValue::Long(value));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("valueOf", [JValue::Long(v)]) => {
            let reference = heap.box_wrapper("java/lang/Long", JValue::Long(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("toString", [JValue::Long(v)]) => s(heap, v.to_string()),
        ("toString", [JValue::Long(v), JValue::Int(radix)]) => {
            s(heap, long_to_string_radix(*v, *radix))
        }
        ("toBinaryString", [JValue::Long(v)]) => s(heap, format!("{:b}", v.cast_unsigned())),
        ("toOctalString", [JValue::Long(v)]) => s(heap, format!("{:o}", v.cast_unsigned())),
        ("toHexString", [JValue::Long(v)]) => s(heap, format!("{:x}", v.cast_unsigned())),
        ("compare", [JValue::Long(a), JValue::Long(b)]) => i(match a.cmp(b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }),
        ("max", [JValue::Long(a), JValue::Long(b)]) => l((*a).max(*b)),
        ("min", [JValue::Long(a), JValue::Long(b)]) => l((*a).min(*b)),
        ("sum", [JValue::Long(a), JValue::Long(b)]) => l(a.wrapping_add(*b)),
        ("signum", [JValue::Long(v)]) => i(i32::try_from(v.signum()).unwrap_or(0)),
        ("hashCode", [JValue::Long(v)]) => i((((v.cast_unsigned() ^ (v.cast_unsigned() >> 32))
            & 0xFFFF_FFFF) as u32)
            .cast_signed()),
        ("bitCount", [JValue::Long(v)]) => i(i32::try_from(v.count_ones()).unwrap_or(0)),
        ("numberOfLeadingZeros", [JValue::Long(v)]) => {
            i(i32::try_from(v.leading_zeros()).unwrap_or(64))
        }
        ("numberOfTrailingZeros", [JValue::Long(v)]) => {
            i(i32::try_from(v.trailing_zeros()).unwrap_or(64))
        }
        ("reverse", [JValue::Long(v)]) => l(v.reverse_bits()),
        ("reverseBytes", [JValue::Long(v)]) => l(v.swap_bytes()),
        ("highestOneBit", [JValue::Long(v)]) => l(if *v == 0 {
            0
        } else {
            1i64.wrapping_shl(v.cast_unsigned().ilog2())
        }),
        ("lowestOneBit", [JValue::Long(v)]) => l(v & v.wrapping_neg()),
        // Rotation distance is used modulo 64 (JLS / the JDK).
        ("rotateLeft", [JValue::Long(v), JValue::Int(d)]) => {
            l(v.rotate_left((d & 63).cast_unsigned()))
        }
        ("rotateRight", [JValue::Long(v), JValue::Int(d)]) => {
            l(v.rotate_right((d & 63).cast_unsigned()))
        }
        // The JDK implements the LONG unsigned pair through `BigInteger`, so a
        // zero divisor carries its message, not the `/ by zero` of the int
        // forms beside it.
        ("divideUnsigned", [JValue::Long(a), JValue::Long(b)]) => {
            if *b == 0 {
                return Err(throw(
                    "java.lang.ArithmeticException: BigInteger divide by zero",
                ));
            }
            l((a.cast_unsigned() / b.cast_unsigned()).cast_signed())
        }
        ("remainderUnsigned", [JValue::Long(a), JValue::Long(b)]) => {
            if *b == 0 {
                return Err(throw(
                    "java.lang.ArithmeticException: BigInteger divide by zero",
                ));
            }
            l((a.cast_unsigned() % b.cast_unsigned()).cast_signed())
        }
        ("compareUnsigned", [JValue::Long(a), JValue::Long(b)]) => {
            i(match a.cast_unsigned().cmp(&b.cast_unsigned()) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })
        }
        ("toUnsignedString", [JValue::Long(v)]) => s(heap, v.cast_unsigned().to_string()),
        ("toUnsignedString", [JValue::Long(v), JValue::Int(radix)]) => {
            let radix = if (2..=36).contains(radix) { *radix } else { 10 };
            let radix = u64::try_from(radix).expect("radix in range");
            let mut magnitude = v.cast_unsigned();
            let mut digits = Vec::new();
            loop {
                let digit = u32::try_from(magnitude % radix).expect("digit < radix");
                let radix32 = u32::try_from(radix).expect("radix in range");
                digits.push(char::from_digit(digit, radix32).expect("valid digit"));
                magnitude /= radix;
                if magnitude == 0 {
                    break;
                }
            }
            s(heap, digits.iter().rev().collect())
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Long.{method}"))),
    }
}

/// `Short` and `Byte` wrapper statics — identical shapes, differing
/// only in range.
fn small_int_static(
    heap: &mut Heap,
    class: &str,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let (lo, hi) = if class == "Short" {
        (i32::from(i16::MIN), i32::from(i16::MAX))
    } else {
        (i32::from(i8::MIN), i32::from(i8::MAX))
    };
    match (method, args) {
        ("parseShort" | "parseByte" | "valueOf", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            let value: i32 = text.parse().map_err(|_| number_format(&text.raw))?;
            if value < lo || value > hi {
                return Err(number_format_range(&text));
            }
            if method == "valueOf" {
                let wrapper = if class == "Short" {
                    "java/lang/Short"
                } else {
                    "java/lang/Byte"
                };
                let reference = heap.box_wrapper(wrapper, JValue::Int(value));
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            Ok(Some(JValue::Int(value)))
        }
        ("hashCode", [JValue::Int(v)]) => Ok(Some(JValue::Int(*v))),
        ("valueOf", [JValue::Int(v)]) => {
            let reference = heap.box_wrapper(
                if class == "Short" {
                    "java/lang/Short"
                } else {
                    "java/lang/Byte"
                },
                JValue::Int(*v),
            );
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("toString", [JValue::Int(v)]) => {
            let reference = heap.alloc_string(&v.to_string());
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // `Short.compare`/`Byte.compare` return `x - y` (the JDK source), NOT
        // the -1/0/1 sign that `Integer.compare` gives — the values are small
        // enough that the subtraction cannot overflow.
        ("compare", [JValue::Int(a), JValue::Int(b)]) => Ok(Some(JValue::Int(a - b))),
        // Zero-extend the low byte/short: `Byte.toUnsignedInt((byte) -1)` is
        // 255, `Short.toUnsignedInt((short) -1)` is 65535.
        ("toUnsignedInt", [JValue::Int(v)]) => {
            let mask = if class == "Short" { 0xFFFF } else { 0xFF };
            Ok(Some(JValue::Int(v & mask)))
        }
        ("toUnsignedLong", [JValue::Int(v)]) => {
            let mask = if class == "Short" { 0xFFFF } else { 0xFF };
            Ok(Some(JValue::Long(i64::from(v & mask))))
        }
        ("reverseBytes", [JValue::Int(v)]) => {
            #[allow(clippy::cast_possible_truncation)]
            let value = *v as i16;
            Ok(Some(JValue::Int(i32::from(value.swap_bytes()))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("{class}.{method}"))),
    }
}

/// javac range message for `Byte.parseByte("200")`.
fn number_format_range(text: &str) -> VmError {
    // `Short.parseShort`/`Byte.parseByte` range-check the result of
    // `Integer.parseInt(s, radix)` and throw exactly this — no class suffix.
    VmError::UncaughtException(format!(
        "java.lang.NumberFormatException: Value out of range. Value:\"{text}\" Radix:10"
    ))
}

fn float_static(heap: &mut Heap, method: &str, args: &[JValue]) -> Result<Option<JValue>, VmError> {
    let f = |v: f32| Ok(Some(JValue::Float(v)));
    let b = |v: bool| Ok(Some(JValue::Int(i32::from(v))));
    match (method, args) {
        ("parseFloat" | "valueOf", [text @ JValue::Ref(_)]) => {
            // `valueOf` answers the WRAPPER OBJECT; only `parseX` answers the
            // primitive. Returning the primitive for both made two `valueOf`
            // results compare `==` by value.
            let parsed = (|| -> Result<Option<JValue>, VmError> {
                // Like Double: null NPEs, an empty/blank string is "empty String".
                if matches!(text, JValue::Ref(None)) {
                    return Err(throw("java.lang.NullPointerException"));
                }
                let text = number_arg_text(heap, text)?;
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    return Err(throw("java.lang.NumberFormatException: empty String"));
                }
                parse_java_float(trimmed).map_or_else(|| Err(number_format(&text)), f)
            })()?;
            if method == "valueOf"
                && let Some(value) = parsed
            {
                let reference = heap.box_wrapper("java/lang/Float", value);
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            Ok(parsed)
        }
        ("valueOf", [JValue::Float(v)]) => {
            let reference = heap.box_wrapper("java/lang/Float", JValue::Float(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("toString", [JValue::Float(v)]) => {
            let reference = heap.alloc_string(&java_float_to_string(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("isNaN", [JValue::Float(v)]) => b(v.is_nan()),
        ("isInfinite", [JValue::Float(v)]) => b(v.is_infinite()),
        ("isFinite", [JValue::Float(v)]) => b(v.is_finite()),
        ("compare", [JValue::Float(a), JValue::Float(b)]) => {
            // Java total order: -0.0 < 0.0, NaN greatest.
            Ok(Some(JValue::Int(match a.total_cmp(b) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })))
        }
        ("max", [JValue::Float(a), JValue::Float(b)]) => f(java_float_max(*a, *b)),
        ("min", [JValue::Float(a), JValue::Float(b)]) => f(java_float_min(*a, *b)),
        ("sum", [JValue::Float(a), JValue::Float(b)]) => f(a + b),
        ("hashCode" | "floatToIntBits", [JValue::Float(v)]) => {
            let bits = if v.is_nan() {
                0x7FC0_0000_u32
            } else {
                v.to_bits()
            };
            Ok(Some(JValue::Int(bits.cast_signed())))
        }
        ("floatToRawIntBits", [JValue::Float(v)]) => {
            Ok(Some(JValue::Int(v.to_bits().cast_signed())))
        }
        ("intBitsToFloat", [JValue::Int(v)]) => f(f32::from_bits(v.cast_unsigned())),
        _ => Err(VmError::UnknownIntrinsic(format!("Float.{method}"))),
    }
}

/// Java `Math.max(float)`: NaN wins; +0.0 beats -0.0.
fn java_float_max(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        return f32::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_positive() || b.is_sign_positive() {
            0.0
        } else {
            -0.0
        };
    }
    if a > b { a } else { b }
}

fn java_float_min(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        return f32::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_negative() || b.is_sign_negative() {
            -0.0
        } else {
            0.0
        };
    }
    if a < b { a } else { b }
}

fn string_static(
    heap: &mut Heap,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    if method == "format" {
        let template = match args.first() {
            Some(JValue::Ref(Some(reference))) => heap.string_text(*reference).unwrap_or_default(),
            // The JDK's own NPE here carries NO message (it comes from
            // `Objects.requireNonNull(format)` without one).
            Some(JValue::Ref(None)) => {
                return Err(throw("java.lang.NullPointerException"));
            }
            _ => String::new(),
        };
        let format_args = crate::format::args_from_descriptor(heap, descriptor, &args[1..])?;
        let text = crate::format::java_format(heap, &template, &format_args)?;
        let reference = heap.alloc_string_units(&text);
        return Ok(Some(JValue::Ref(Some(reference))));
    }
    if method == "join" {
        return string_join(heap, descriptor, args);
    }
    match (method, args) {
        // `valueOf(char[], offset, count)` — the subrange overload.
        (
            "valueOf" | "copyValueOf",
            [
                JValue::Ref(Some(reference)),
                JValue::Int(offset),
                JValue::Int(count),
            ],
        ) => {
            let Some(HeapObject::IntArray(_, values)) = heap.get(*reference) else {
                return Err(VmError::UnknownIntrinsic(String::from(
                    "String.valueOf(char[], int, int) needs a char array",
                )));
            };
            let (offset_u, count_u) = (
                usize::try_from(*offset).unwrap_or(usize::MAX),
                usize::try_from(*count).unwrap_or(usize::MAX),
            );
            let end = offset_u.saturating_add(count_u);
            if *offset < 0 || *count < 0 || end > values.len() {
                return Err(throw(format!(
                    "java.lang.StringIndexOutOfBoundsException: offset {offset}, count {count}, length {}",
                    values.len()
                )));
            }
            // The UNITS verbatim, exactly as the whole-array overload below
            // already does: rendering each through `char::from_u32` destroys
            // every surrogate pair the range happens to contain.
            let units: Vec<u16> = values[offset_u..end]
                .iter()
                .map(|v| u16::try_from(*v).unwrap_or(u16::MAX))
                .collect();
            let reference = heap.alloc(HeapObject::JavaString(units));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // `String.valueOf(char[])` / `copyValueOf(char[])` copy the UNITS
        // verbatim. Rendering each through `char::from_u32` destroyed every
        // surrogate PAIR — a non-BMP character came back as two U+FFFDs — and
        // a null array must throw, not print "null" (that is the `Object`
        // overload, which a `char[]` never takes).
        ("valueOf" | "copyValueOf", [value]) if descriptor.starts_with("([C)") => {
            let units = char_array_units(heap, value)?;
            let reference = heap.alloc(HeapObject::JavaString(units));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("valueOf" | "copyValueOf", [value]) => {
            // The descriptor disambiguates int/char/boolean, which all
            // arrive as JValue::Int.
            // A `char` is ONE UTF-16 unit and stays one, even when it is an
            // unpaired surrogate: `char::from_u32` rejects those, and the
            // U+FFFD it fell back to is a different character. `"" + c`
            // lowers to this, so the loss showed up in ordinary concatenation.
            if let ("(C)Ljava/lang/String;", JValue::Int(v)) = (descriptor, value) {
                let unit = u16::try_from(*v).unwrap_or(u16::MAX);
                let reference = heap.alloc(HeapObject::JavaString(vec![unit]));
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            let text = match (descriptor, value) {
                ("(Z)Ljava/lang/String;", JValue::Int(v)) => {
                    String::from(if *v != 0 { "true" } else { "false" })
                }
                (_, JValue::Int(v)) => v.to_string(),
                (_, JValue::Long(v)) => v.to_string(),
                ("(F)Ljava/lang/String;", JValue::Float(v)) => java_float_to_string(*v),
                (_, JValue::Double(v)) => java_double_to_string(*v),
                (_, JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::IntArray(_, values)) => values
                        .iter()
                        .map(|v| {
                            char::from_u32(u32::try_from(*v).unwrap_or(0)).unwrap_or('\u{FFFD}')
                        })
                        .collect(),
                    _ => heap.string_text(*reference).unwrap_or_default(),
                },
                (_, JValue::Ref(None)) => String::from("null"),
                _ => String::new(),
            };
            let reference = heap.alloc_string(&text);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("String.{method}"))),
    }
}

/// `String.join(delimiter, ...)`. The codegen descriptor says how argument 1 is
/// shaped: a `String[]`, a `List`, or the first of individual `CharSequence`
/// arguments. A null delimiter or a null array/iterable throws; a null element
/// joins as the text `null`, as `StringJoiner` produces.
/// The elements of a set-like collection, in iteration order — what
/// `String.join(delimiter, Iterable)` walks when the argument is not a list.
fn set_like_elements(heap: &Heap, reference: HeapRef) -> Option<Vec<JValue>> {
    Some(match heap.get(reference)? {
        HeapObject::HashSet(entries) => entries
            .entries_in_order()
            .into_iter()
            .map(|(element, _)| element)
            .collect(),
        HeapObject::TreeSet { values, .. } => values.clone(),
        HeapObject::UnmodifiableSet(inner) => set_like_elements(heap, *inner)?,
        // ...and an unmodifiable LIST, which `List.of` and
        // `Collections.unmodifiableList` both answer. Without this arm the one
        // reader that falls back here — `String.join(delimiter, iterable)` —
        // threw `NullPointerException` for `String.join("-", List.of("a"))`,
        // ordinary Java that works through any other list.
        HeapObject::UnmodifiableList(inner) => heap
            .list_values(*inner)
            .cloned()
            .or_else(|| set_like_elements(heap, *inner))?,
        // A sorted view is iterable like any other collection, and a MapView
        // over one reaches here through the same door.
        HeapObject::SortedView { .. } => sorted_view_pairs(heap, reference)
            .into_iter()
            .map(|(key, _)| key)
            .collect(),
        HeapObject::MapView { map, kind, .. }
            if matches!(heap.get(*map), Some(HeapObject::SortedView { .. })) =>
        {
            let kind = *kind;
            sorted_view_pairs(heap, *map)
                .into_iter()
                .map(|(key, value)| {
                    if matches!(kind, MapViewKind::Values) {
                        value
                    } else {
                        key
                    }
                })
                .collect()
        }
        // A `subList` VIEW is iterable like any other collection — reading it
        // as a range of the backing list is what every other reader does.
        HeapObject::SubList {
            backing, from, len, ..
        } => {
            let (from, len) = (*from, *len);
            let items = heap
                .list_values(*backing)
                .cloned()
                .or_else(|| set_like_elements(heap, *backing))?;
            items.into_iter().skip(from).take(len).collect()
        }
        _ => return None,
    })
}

/// Trim units from one or both ends of a string, by a predicate over UNITS.
fn trim_units(units: &[u16], leading: bool, trailing: bool, cut: impl Fn(u16) -> bool) -> &[u16] {
    let mut start = 0;
    let mut end = units.len();
    if leading {
        while start < end && cut(units[start]) {
            start += 1;
        }
    }
    if trailing {
        while end > start && cut(units[end - 1]) {
            end -= 1;
        }
    }
    &units[start..end]
}

fn string_join(
    heap: &mut Heap,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let delimiter: Vec<u16> = match args.first() {
        Some(JValue::Ref(Some(reference))) => {
            heap.string_units(*reference).unwrap_or_default().to_vec()
        }
        Some(JValue::Ref(None)) => {
            // `Objects.requireNonNull(delimiter)` throws bare.
            return Err(throw("java.lang.NullPointerException"));
        }
        _ => Vec::new(),
    };
    let shape = descriptor
        .strip_prefix("(Ljava/lang/String;")
        .and_then(|rest| rest.strip_suffix(")Ljava/lang/String;"))
        .unwrap_or("");
    let elements: Vec<JValue> = if shape.starts_with('[') {
        match args.get(1) {
            Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                Some(HeapObject::RefArray(_, values)) => values.clone(),
                _ => return Err(throw("java.lang.NullPointerException")),
            },
            _ => return Err(throw("java.lang.NullPointerException")),
        }
    } else if shape.starts_with("Ljava/util/List;") {
        // `join(delimiter, Iterable)` takes ANY collection, so a Set or a
        // Deque reaches here too — `list_values` sees only the sequence kinds.
        match args.get(1) {
            Some(JValue::Ref(Some(reference))) => match heap.list_values(*reference) {
                Some(values) => values.clone(),
                None => set_like_elements(heap, *reference)
                    .ok_or_else(|| throw("java.lang.NullPointerException"))?,
            },
            _ => return Err(throw("java.lang.NullPointerException")),
        }
    } else {
        args.get(1..).unwrap_or(&[]).to_vec()
    };
    let mut out: Vec<u16> = Vec::new();
    for (at, value) in elements.iter().enumerate() {
        if at > 0 {
            out.extend_from_slice(&delimiter);
        }
        match value {
            JValue::Ref(Some(reference)) => match heap.string_units(*reference) {
                Some(units) => out.extend_from_slice(units),
                None => out.extend("null".encode_utf16()),
            },
            _ => out.extend("null".encode_utf16()),
        }
    }
    let reference = heap.alloc_string_units(&out);
    Ok(Some(JValue::Ref(Some(reference))))
}

/// Render a `StringBuilder.append` argument the way Java would.
/// Render an `Object`-typed value inline (`String.valueOf` semantics) for the
/// reflection handles that flow through `append(Object)`/`print(Object)`.
/// An object's identity hash: its heap reference, as a real JVM uses its
/// address. `Object.toString()` prints this same value in hex, so the two
/// agree exactly as they do on a JVM.
pub(crate) fn identity_hash(reference: HeapRef) -> i32 {
    i32::from_ne_bytes(reference.to_ne_bytes())
}

/// The JVM class descriptor of an array on the heap: `[I`, `[[I`,
/// `[Ljava/lang/String;`. A primitive array carries it in its variant; a
/// reference array remembers its own, because once the static type is gone
/// the heap is the only thing that still knows the element type.
pub(crate) fn array_class_name(heap: &Heap, reference: HeapRef) -> Option<String> {
    Some(match heap.get(reference)? {
        HeapObject::IntArray(IntKind::Int, _) => String::from("[I"),
        HeapObject::IntArray(IntKind::Boolean, _) => String::from("[Z"),
        HeapObject::IntArray(IntKind::Char, _) => String::from("[C"),
        HeapObject::DoubleArray(_) => String::from("[D"),
        HeapObject::LongArray(_) => String::from("[J"),
        HeapObject::FloatArray(_) => String::from("[F"),
        HeapObject::ShortArray(_) => String::from("[S"),
        HeapObject::ByteArray(_) => String::from("[B"),
        HeapObject::RefArray(class, _) => class.clone(),
        _ => return None,
    })
}

/// `Object.toString()` for an array, exactly as the JDK writes it:
/// `getClass().getName() + "@" + Integer.toHexString(hashCode())`, where
/// `getName()` spells a reference element with dots. An `int[]` is
/// `[I@1b6d3586`; a `String[]` is `[Ljava.lang.String;@4554617c`. Useless
/// to read, and precisely what a student sees on a real JVM.
pub(crate) fn array_to_string(heap: &Heap, reference: HeapRef) -> Option<String> {
    let name = array_class_name(heap, reference)?.replace('/', ".");
    Some(format!("{name}@{:x}", identity_hash(reference)))
}

pub(crate) fn object_display(heap: &Heap, value: JValue) -> String {
    match value {
        JValue::Ref(None) => String::from("null"),
        JValue::Ref(Some(reference)) => match heap.get(reference) {
            // A StringBuilder renders as its content (String.valueOf/append(Object)
            // call toString), same as a String.
            Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                String::from_utf16_lossy(units)
            }
            // `Class.toString()`: "class <binary name>", "interface <name>"
            // for an interface, and the bare name for a primitive (JDK). The
            // name was rendered raw, so it printed the INTERNAL
            // `class java/lang/String`, a name Java never produces.
            Some(HeapObject::Class { name }) => crate::interpreter::class_to_string(name, false),
            // An array does not override `toString`, so it gets Object's:
            // `[I@1b6d3586`. Not `Arrays.toString`'s `[1, 2, 3]`.
            Some(_) if array_class_name(heap, reference).is_some() => {
                array_to_string(heap, reference).unwrap_or_default()
            }
            Some(HeapObject::Boxed { class_name, value }) => boxed_to_string(class_name, *value),
            Some(HeapObject::Field {
                declaring,
                name,
                descriptor,
                access,
                ..
            }) => field_to_string(declaring, name, descriptor, *access),
            Some(HeapObject::Constructor {
                declaring,
                descriptor,
                access,
            }) => constructor_to_string(declaring, descriptor, *access),
            Some(HeapObject::Method {
                declaring,
                name,
                descriptor,
                ..
            }) => format!("{declaring}.{name}{descriptor}"),
            Some(HeapObject::ReflectType { raw, args }) => {
                let dotted = raw.replace('/', ".");
                if args.is_empty() {
                    dotted
                } else {
                    format!("{dotted}<{}>", args.join(", "))
                }
            }
            Some(HeapObject::StackFrame {
                declaring,
                method,
                file,
                line,
            }) => crate::interpreter::stack_frame_text(declaring, method, file.as_deref(), *line),
            Some(HeapObject::Instance { class_name, .. }) => format!("{class_name}@{reference:x}"),
            // A stream is an ordinary object for display purposes: the JDK
            // prints its pipeline class and identity hash, and printing one is
            // usually a mistake — but it must not be an internal error.
            Some(HeapObject::Stream { .. }) => {
                format!("java.util.stream.ReferencePipeline${reference:x}")
            }
            // The library objects that print as a VALUE. A collection renders
            // its elements through here, and every one of these printed as
            // `object@2a` inside a list while printing the same object
            // directly said `x` — one toString written twice, and only one of
            // them reached from a collection.
            Some(HeapObject::Path(text) | HeapObject::Charset(text) | HeapObject::File(text)) => {
                text.clone()
            }
            Some(HeapObject::Temporal(value)) => value.text(),
            Some(HeapObject::Collector(_)) => collector_text(reference),
            Some(HeapObject::SummaryStats {
                count,
                sum,
                min,
                max,
                kind,
            }) => summary_text(*count, *sum, *min, *max, *kind),
            Some(HeapObject::DoubleSummaryStats {
                count,
                sum,
                min,
                max,
            }) => double_summary_text(*count, *sum, *min, *max),
            Some(HeapObject::Pattern { source, flags }) => {
                String::from_utf16_lossy(&fold_regex_flags(source, *flags))
            }
            Some(HeapObject::Matcher { .. }) => {
                matcher_text(heap, reference).unwrap_or_else(|| format!("object@{reference:x}"))
            }
            Some(HeapObject::MatchResult { .. }) => format!(
                "java.util.regex.Matcher$ImmutableMatchResult@{:x}",
                identity_hash(reference)
            ),
            _ => format!("object@{reference:x}"),
        },
        other => format!("{other:?}"),
    }
}

fn builder_value_text(heap: &Heap, param: &str, value: &JValue) -> Result<Vec<u16>, VmError> {
    let text = match (param, value) {
        ("I", JValue::Int(v)) => v.to_string(),
        ("J", JValue::Long(v)) => v.to_string(),
        ("F", JValue::Float(v)) => java_float_to_string(*v),
        ("Z", JValue::Int(v)) => if *v != 0 { "true" } else { "false" }.to_owned(),
        ("D", JValue::Double(v)) => java_double_to_string(*v),
        // A `char` is one UTF-16 unit, even an unpaired surrogate — which
        // `char::from_u32` would reject. Keep the raw unit.
        ("C", JValue::Int(v)) => return Ok(vec![u16::try_from(*v).unwrap_or(u16::MAX)]),
        ("[C", value) => return char_array_units(heap, value),
        // Straight from the UNITS. Going through `string_text` converts by way
        // of a Rust `String`, which cannot hold an unpaired surrogate, so
        // appending such a string to a builder REPLACED the unit rather than
        // copying it — a corrupted value, not merely a rendering.
        ("Ljava/lang/String;", JValue::Ref(reference)) => match reference {
            None => String::from("null"),
            Some(reference) => match heap.get(*reference) {
                Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                    return Ok(units.clone());
                }
                _ => {
                    return Err(VmError::UnknownIntrinsic(String::from(
                        "argument is not a string object",
                    )));
                }
            },
        },
        // `insert(int, Object)` — `String.valueOf(Object)` semantics.
        // `append(Object)` never reaches here: the interpreter renders its
        // argument first, since doing so may call a user `toString()`.
        ("Ljava/lang/Object;", value) => object_display(heap, *value),
        _ => {
            return Err(VmError::UnknownIntrinsic(format!(
                "StringBuilder overload with parameter {param}"
            )));
        }
    };
    Ok(text.encode_utf16().collect())
}

/// The UTF-16 units of a `char[]` argument (`char` values live in an
/// `IntArray`, so an unpaired surrogate survives the round trip).
fn char_array_units(heap: &Heap, value: &JValue) -> Result<Vec<u16>, VmError> {
    match value {
        JValue::Ref(Some(reference)) => match heap.get(*reference) {
            Some(HeapObject::IntArray(_, values)) => Ok(values
                .iter()
                .map(|v| u16::try_from(*v).unwrap_or(u16::MAX))
                .collect()),
            _ => Err(throw("java.lang.ClassCastException: not a char[]")),
        },
        JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
        _ => Err(throw("java.lang.VerifyError: expected a char[] argument")),
    }
}

/// The parameter descriptors of a method descriptor, i.e. what sits
/// between the parentheses (`"(ILjava/lang/String;)V"` → `"ILjava/lang/String;"`).
fn descriptor_params(descriptor: &str) -> &str {
    descriptor
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'))
        .map_or("", |(params, _)| params)
}

/// Render a print/println argument the way Java would.
/// What an UNPAIRED surrogate becomes on the way to the console.
///
/// A lone surrogate is not a character and cannot be encoded, so a JDK's
/// `PrintStream` hands it to the charset encoder, which substitutes its
/// default replacement — a plain `?`. Rust's lossy conversion substitutes
/// U+FFFD instead, which is a DIFFERENT character and one a program can also
/// legitimately print, so the two must not be conflated.
const UNENCODABLE: char = '?';

/// UTF-16 units as console text: valid characters through, unpaired
/// surrogates as `?`.
fn console_text(units: &[u16]) -> String {
    char::decode_utf16(units.iter().copied())
        .map(|decoded| decoded.unwrap_or(UNENCODABLE))
        .collect()
}

fn print_argument_text(heap: &Heap, descriptor: &str, args: &[JValue]) -> Result<String, VmError> {
    // `append` has the same argument shapes as `print`, but answers the
    // stream — so its descriptors end in the stream type, not `V`.
    let descriptor = match descriptor.split_once(')') {
        Some((params, _)) => format!("{params})V"),
        None => descriptor.to_owned(),
    };
    let text = match (descriptor.as_str(), args) {
        ("()V", []) => String::new(),
        ("(I)V", [JValue::Int(v)]) => v.to_string(),
        ("(J)V", [JValue::Long(v)]) => v.to_string(),
        ("(F)V", [JValue::Float(v)]) => java_float_to_string(*v),
        ("(Z)V", [JValue::Int(v)]) => if *v != 0 { "true" } else { "false" }.to_owned(),
        ("(C)V", [JValue::Int(v)]) => {
            let unit = u32::try_from(*v).unwrap_or(u32::from(u16::MAX));
            char::from_u32(unit).map_or_else(|| String::from(UNENCODABLE), String::from)
        }
        ("(D)V", [JValue::Double(v)]) => java_double_to_string(*v),
        // `println(char[])` is a real overload: it prints the CHARACTERS.
        // (`"" + chars` does not — that is `append(Object)` and prints
        // `[C@hash`. A classic Java trap, faithfully reproduced.)
        ("([C)V", [value]) => console_text(&char_array_units(heap, value)?),
        ("(Ljava/lang/String;)V" | "(Ljava/lang/CharSequence;)V", [JValue::Ref(reference)]) => {
            match reference {
                None => String::from("null"),
                // Straight from the UNITS: a string carrying an unpaired
                // surrogate has to reach the console as `?`, and the lossy
                // conversion `string_text` performs would make it U+FFFD.
                Some(reference) => match heap.get(*reference) {
                    Some(HeapObject::JavaString(units)) => console_text(units),
                    _ => {
                        return Err(VmError::UnknownIntrinsic(String::from(
                            "println argument is not a string object",
                        )));
                    }
                },
            }
        }
        _ => {
            return Err(VmError::UnknownIntrinsic(format!(
                "PrintStream overload {descriptor} with {} args",
                args.len()
            )));
        }
    };
    Ok(text)
}

/// Format a `double` the way `Double.toString` does for common cases.
///
/// TODO(classlib): full `Double.toString` semantics (shortest digits
/// that round-trip, exact scientific-notation thresholds). Current
/// coverage: NaN/infinities, integral values gaining `.0`, and the 1e7
/// switch to scientific notation.
/// `Double.toString`, matching `OpenJDK` 11's `FloatingDecimal`.
pub(crate) fn java_double_to_string(value: f64) -> String {
    crate::floatdec::java_double_to_string(value)
}

/// `Float.toString`, matching `OpenJDK` 11's `FloatingDecimal`.
pub(crate) fn java_float_to_string(value: f32) -> String {
    crate::floatdec::java_float_to_string(value)
}

/// A comparison that needs no user code: two numbers, two characters, two
/// booleans, two Strings, or the boxed forms of those. `None` means the JDK
/// would call the program's own `compareTo` (or a `Comparator` it wrote), which
/// only the interpreter can do — see [`sorted_view_range`] for why a reader
/// that has nothing but the heap still gets a right answer for every tree keyed
/// by one of these.
fn native_order(heap: &Heap, a: JValue, b: JValue) -> Option<std::cmp::Ordering> {
    use crate::value::HeapObject;
    match (a, b) {
        (JValue::Int(x), JValue::Int(y)) => Some(x.cmp(&y)),
        (JValue::Long(x), JValue::Long(y)) => Some(x.cmp(&y)),
        (JValue::Double(x), JValue::Double(y)) => Some(crate::interpreter::java_double_order(x, y)),
        (JValue::Float(x), JValue::Float(y)) => Some(crate::interpreter::java_double_order(
            f64::from(x),
            f64::from(y),
        )),
        (JValue::Ref(Some(left)), JValue::Ref(Some(right))) => {
            match (heap.get(left), heap.get(right)) {
                (Some(HeapObject::JavaString(x)), Some(HeapObject::JavaString(y))) => {
                    Some(x.cmp(y))
                }
                (
                    Some(HeapObject::Boxed { value: x, .. }),
                    Some(HeapObject::Boxed { value: y, .. }),
                ) => native_order(heap, *x, *y),
                _ => None,
            }
        }
        _ => None,
    }
}

/// The vector a sorted view reads, as (key, value) pairs in the BACKING's own
/// ascending order. A set's pairs are its elements twice over, which is what
/// lets all three faces of a view share one resolver.
pub(crate) fn sorted_backing_pairs(heap: &Heap, backing: HeapRef) -> Vec<(JValue, JValue)> {
    use crate::value::HeapObject;
    match heap.get(backing) {
        Some(HeapObject::TreeSet { values, .. }) => values.iter().map(|v| (*v, *v)).collect(),
        Some(HeapObject::TreeMap { entries, .. }) => entries.clone(),
        _ => Vec::new(),
    }
}

/// Whether the tree a view reads orders itself with user code — a `Comparator`
/// object, or elements whose `compareTo` is the program's. Such a tree's bounds
/// can only be resolved by the interpreter.
fn orders_natively(heap: &Heap, backing: HeapRef) -> bool {
    use crate::value::HeapObject;
    let comparator = match heap.get(backing) {
        Some(HeapObject::TreeSet { comparator, .. } | HeapObject::TreeMap { comparator, .. }) => {
            *comparator
        }
        _ => return false,
    };
    comparator.is_none()
}

/// Where a view's bounds fall in a sorted vector — the ONE statement of what a
/// bound means, shared by the two callers that can ask. `order` is how this
/// tree compares, which is the only thing that differs between them: a reader
/// holding nothing but the heap passes a comparison that fails on anything
/// needing user code, and the interpreter passes the tree's real comparator.
pub(crate) fn resolve_sorted_range<E, F>(
    pairs: &[(JValue, JValue)],
    lo: Option<crate::value::SortedBound>,
    hi: Option<crate::value::SortedBound>,
    mut order: F,
) -> Result<(usize, usize), E>
where
    F: FnMut(JValue, JValue) -> Result<std::cmp::Ordering, E>,
{
    use std::cmp::Ordering;
    let mut from = 0;
    if let Some(bound) = lo {
        // Nothing is in range until something is found to be: an empty tree,
        // or one whose every key sits below the bound, is an empty view.
        from = pairs.len();
        for (at, (key, _)) in pairs.iter().enumerate() {
            let ordering = order(*key, bound.value)?;
            let inside = if bound.inclusive {
                ordering != Ordering::Less
            } else {
                ordering == Ordering::Greater
            };
            if inside {
                from = at;
                break;
            }
        }
    }
    let mut to = pairs.len();
    if let Some(bound) = hi {
        for (at, (key, _)) in pairs.iter().enumerate() {
            let ordering = order(*key, bound.value)?;
            let past = if bound.inclusive {
                ordering == Ordering::Greater
            } else {
                ordering != Ordering::Less
            };
            if past {
                to = at;
                break;
            }
        }
    }
    // A `subMap(b, a)` with the ends crossed is an IllegalArgumentException at
    // construction, so a range that comes out backwards here can only be an
    // empty one.
    Ok((from, to.max(from)))
}

/// The bounds resolved with NATIVE comparisons only. `None` when any
/// comparison the resolution needs would run the program's code.
fn native_sorted_range(
    heap: &Heap,
    backing: HeapRef,
    lo: Option<crate::value::SortedBound>,
    hi: Option<crate::value::SortedBound>,
) -> Option<(usize, usize)> {
    if !orders_natively(heap, backing) {
        return None;
    }
    let pairs = sorted_backing_pairs(heap, backing);
    resolve_sorted_range(&pairs, lo, hi, |a, b| native_order(heap, a, b).ok_or(())).ok()
}

/// The slice of the backing a sorted view presents, as `from..to`. Resolved
/// from the BOUNDS whenever the ordering is native — which keeps the view live
/// for every tree keyed by a number, a character or a String, even for a
/// reader that holds nothing but the heap. A tree ordered by the program's own
/// code falls back to the range the last call THROUGH the view resolved.
pub(crate) fn sorted_view_range(heap: &Heap, view: HeapRef) -> (usize, usize) {
    use crate::value::HeapObject;
    let Some(HeapObject::SortedView {
        backing,
        lo,
        hi,
        range,
        ..
    }) = heap.get(view)
    else {
        return (0, 0);
    };
    let (backing, lo, hi, range) = (*backing, *lo, *hi, *range);
    let len = sorted_backing_pairs(heap, backing).len();
    native_sorted_range(heap, backing, lo, hi).unwrap_or((range.0.min(len), range.1.min(len)))
}

/// The (key, value) pairs a sorted view presents, in the VIEW's order — which
/// is the backing's, reversed for a descending view.
pub(crate) fn sorted_view_pairs(heap: &Heap, view: HeapRef) -> Vec<(JValue, JValue)> {
    use crate::value::HeapObject;
    let Some(HeapObject::SortedView {
        backing,
        descending,
        ..
    }) = heap.get(view)
    else {
        return Vec::new();
    };
    let (backing, descending) = (*backing, *descending);
    let (from, to) = sorted_view_range(heap, view);
    let pairs = sorted_backing_pairs(heap, backing);
    let mut slice: Vec<(JValue, JValue)> = pairs[from.min(pairs.len())..to.min(pairs.len())].into();
    if descending {
        slice.reverse();
    }
    slice
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::BufferedConsole;

    #[test]
    fn println_string_writes_line_to_stdout() {
        let mut heap = Heap::new();
        let mut statics = IntrinsicStatics::default();
        let mut console = BufferedConsole::new();
        let mut vfs = VirtualFileSystem::new();

        let JValue::Ref(Some(out)) = statics
            .static_field(&mut heap, "java/lang/System", "out")
            .unwrap()
        else {
            panic!("expected a reference");
        };
        let text = heap.alloc_string("Hello, World!");
        invoke_virtual(
            &mut heap,
            &mut console,
            &mut vfs,
            out,
            "java/io/PrintStream",
            "println",
            "(Ljava/lang/String;)V",
            &[JValue::Ref(Some(text))],
        )
        .unwrap();
        assert_eq!(console.stdout_text(), "Hello, World!\n");
    }

    #[test]
    fn system_out_is_a_singleton() {
        let mut heap = Heap::new();
        let mut statics = IntrinsicStatics::default();
        let a = statics.static_field(&mut heap, "java/lang/System", "out");
        let b = statics.static_field(&mut heap, "java/lang/System", "out");
        assert_eq!(a, b);
    }

    #[test]
    fn err_goes_to_stderr() {
        let mut heap = Heap::new();
        let mut statics = IntrinsicStatics::default();
        let mut console = BufferedConsole::new();
        let mut vfs = VirtualFileSystem::new();
        let JValue::Ref(Some(err)) = statics
            .static_field(&mut heap, "java/lang/System", "err")
            .unwrap()
        else {
            panic!("expected a reference");
        };
        invoke_virtual(
            &mut heap,
            &mut console,
            &mut vfs,
            err,
            "java/io/PrintStream",
            "println",
            "(I)V",
            &[JValue::Int(7)],
        )
        .unwrap();
        assert_eq!(console.stderr_text(), "7\n");
        assert_eq!(console.stdout_text(), "");
    }

    #[test]
    fn stringbuilder_appends_and_converts() {
        let mut heap = Heap::new();
        let mut console = BufferedConsole::new();
        let mut vfs = VirtualFileSystem::new();
        let builder =
            heap.alloc(instantiate("java/lang/StringBuilder").expect("known intrinsic class"));
        invoke_special(
            &mut heap,
            &mut vfs,
            builder,
            "java/lang/StringBuilder",
            "<init>",
            "()V",
            &[],
        )
        .unwrap();

        let hello = heap.alloc_string("x = ");
        invoke_virtual(
            &mut heap,
            &mut console,
            &mut vfs,
            builder,
            "java/lang/StringBuilder",
            "append",
            "(Ljava/lang/String;)Ljava/lang/StringBuilder;",
            &[JValue::Ref(Some(hello))],
        )
        .unwrap();
        invoke_virtual(
            &mut heap,
            &mut console,
            &mut vfs,
            builder,
            "java/lang/StringBuilder",
            "append",
            "(I)Ljava/lang/StringBuilder;",
            &[JValue::Int(42)],
        )
        .unwrap();
        let result = invoke_virtual(
            &mut heap,
            &mut console,
            &mut vfs,
            builder,
            "java/lang/StringBuilder",
            "toString",
            "()Ljava/lang/String;",
            &[],
        )
        .unwrap();
        let Some(JValue::Ref(Some(text))) = result else {
            panic!("expected a string reference, got {result:?}");
        };
        assert_eq!(heap.string_text(text).as_deref(), Some("x = 42"));
    }

    #[test]
    fn doubles_format_like_java() {
        assert_eq!(java_double_to_string(3.5), "3.5");
        assert_eq!(java_double_to_string(2.0), "2.0");
        assert_eq!(java_double_to_string(0.0), "0.0");
        assert_eq!(java_double_to_string(-4.0), "-4.0");
        assert_eq!(java_double_to_string(10_000_000.0), "1.0E7");
        assert_eq!(java_double_to_string(f64::NAN), "NaN");
        assert_eq!(java_double_to_string(f64::INFINITY), "Infinity");
    }

    #[test]
    fn null_string_prints_null() {
        let heap = Heap::new();
        let text = print_argument_text(&heap, "(Ljava/lang/String;)V", &[JValue::NULL]).unwrap();
        assert_eq!(text, "null");
    }
}
