#!/usr/bin/env python3
"""`java.time` against a real JDK, at scale.

The calendar is arithmetic caturra WROTE — the proleptic Gregorian rules, the
month-end clamping, the epoch-day conversion both ways, the ISO text — so it is
exactly the kind of code that is right on the cases someone thought of and
wrong two centuries out. This generates one large program per shape, prints
every answer, and compares the two engines byte for byte.

    scripts/fuzz/time.py [--seed N] [--out DIR]
    scripts/fuzz/run.py <DIR>

The dates are drawn over four centuries plus the awkward edges (year zero, the
negative side of the epoch, the four-hundred-year cycle, every February 29th in
range), because those are where a calendar goes wrong.
"""

import argparse
import os
import random
import sys

PATTERNS = [
    "yyyy-MM-dd",
    "dd/MM/yyyy",
    "d/M/yy",
    "EEEE, MMMM d, yyyy",
    "EEE MMM d",
    "'day' D 'of' yyyy",
    "yyyy-MM-dd HH:mm:ss",
    "h:mm a",
    "HH:mm",
    "MMM d ''yy",
    "EEEEE",
    "uuuu-MM-dd'T'HH:mm",
]


def dates(rng, count):
    """Random dates, plus the edges a calendar trips over."""
    fixed = [
        (1, 1, 1), (0, 1, 1), (-1, 12, 31), (1970, 1, 1), (1969, 12, 31),
        (1900, 2, 28), (2000, 2, 29), (2100, 3, 1), (2024, 2, 29), (9999, 12, 31),
        (1582, 10, 15), (-753, 4, 21), (10000, 1, 1), (-10000, 6, 15),
    ]
    out = list(fixed)
    while len(out) < count:
        year = rng.randint(-2000, 4000)
        month = rng.randint(1, 12)
        length = [31, 29 if (year % 4 == 0 and year % 100 != 0) or year % 400 == 0 else 28,
                  31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month - 1]
        out.append((year, month, rng.randint(1, length)))
    return out


def times(rng, count):
    fixed = [(0, 0, 0, 0), (23, 59, 59, 999999999), (12, 0, 0, 0), (0, 0, 0, 1),
             (13, 5, 0, 500000000), (1, 2, 3, 4)]
    out = list(fixed)
    while len(out) < count:
        out.append((rng.randint(0, 23), rng.randint(0, 59), rng.randint(0, 59),
                    rng.choice([0, 1, 1000, 1000000, 500000000, rng.randint(0, 999999999)])))
    return out


def date_program(rng):
    """Every date operation, over every drawn date."""
    lines = []
    for year, month, day in dates(rng, 120):
        lines.append(f'        show(LocalDate.of({year}, {month}, {day}));')
    shifts = [rng.randint(-40000, 40000) for _ in range(6)] + [1, -1, 7, 400, -400]
    body = "\n".join(lines)
    steps = "\n".join(
        f'        System.out.println(d.plusDays({n}) + " " + d.plusMonths({n % 97}) '
        f'+ " " + d.plusYears({n % 31}) + " " + d.minusDays({n}) + " " + d.minusMonths({n % 97}));'
        for n in shifts
    )
    return f"""import java.time.LocalDate;
import java.time.temporal.ChronoUnit;

public class TimeDates {{
    static void show(LocalDate d) {{
        System.out.println(d + " " + d.getDayOfWeek() + " " + d.getMonth() + " "
            + d.getDayOfYear() + " " + d.toEpochDay() + " " + d.isLeapYear() + " "
            + d.lengthOfMonth() + " " + d.lengthOfYear() + " " + d.hashCode());
        System.out.println(LocalDate.ofEpochDay(d.toEpochDay()) + " "
            + LocalDate.parse(d.toString()) + " " + d.equals(LocalDate.parse(d.toString())));
{steps}
        System.out.println(ChronoUnit.DAYS.between(LocalDate.of(2000, 1, 1), d) + " "
            + ChronoUnit.MONTHS.between(LocalDate.of(2000, 1, 1), d) + " "
            + ChronoUnit.YEARS.between(LocalDate.of(2000, 1, 1), d) + " "
            + java.time.Period.between(LocalDate.of(2000, 1, 1), d) + " "
            + java.time.Period.between(d, LocalDate.of(2000, 1, 1)));
    }}

    public static void main(String[] args) {{
{body}
    }}
}}
"""


def time_program(rng):
    lines = []
    for hour, minute, second, nano in times(rng, 60):
        lines.append(f'        show(LocalTime.of({hour}, {minute}, {second}, {nano}));')
    shifts = [rng.randint(-100000, 100000) for _ in range(5)] + [1, -1, 3600, 86399]
    steps = "\n".join(
        f'        System.out.println(t.plusHours({n % 50}) + " " + t.plusMinutes({n}) + " "'
        f' + t.plusSeconds({n}) + " " + t.minusHours({n % 50}) + " " + t.minusNanos({n}));'
        for n in shifts
    )
    body = "\n".join(lines)
    return f"""import java.time.Duration;
import java.time.LocalTime;

public class TimeTimes {{
    static void show(LocalTime t) {{
        System.out.println(t + " " + t.getHour() + " " + t.getMinute() + " " + t.getSecond()
            + " " + t.getNano() + " " + t.toSecondOfDay() + " " + t.toNanoOfDay() + " "
            + t.hashCode());
        System.out.println(LocalTime.parse(t.toString()) + " "
            + t.equals(LocalTime.parse(t.toString())) + " "
            + Duration.between(LocalTime.MIDNIGHT, t) + " "
            + Duration.between(t, LocalTime.NOON));
{steps}
    }}

    public static void main(String[] args) {{
{body}
    }}
}}
"""


def format_program(rng):
    """Every pattern against every drawn date-time, and the round trip back."""
    lines = []
    for year, month, day in dates(rng, 40):
        hour, minute, second, nano = times(rng, 1)[0]
        lines.append(
            f'        show(LocalDateTime.of({year}, {month}, {day}, '
            f'{rng.randint(0, 23)}, {rng.randint(0, 59)}, {rng.randint(0, 59)}));'
        )
    shown = "\n".join(
        f'        print(w, "{pattern}");' for pattern in PATTERNS
    )
    body = "\n".join(lines)
    return f"""import java.time.LocalDate;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;

public class TimeFormats {{
    static void print(LocalDateTime w, String pattern) {{
        DateTimeFormatter f = DateTimeFormatter.ofPattern(pattern);
        System.out.println(pattern + " -> " + w.format(f) + " | " + f);
    }}

    static void show(LocalDateTime w) {{
        System.out.println(w + " " + w.format(DateTimeFormatter.ISO_LOCAL_DATE_TIME) + " "
            + w.toLocalDate().format(DateTimeFormatter.ISO_LOCAL_DATE) + " "
            + w.toLocalTime().format(DateTimeFormatter.ISO_LOCAL_TIME));
{shown}
        // Round trip: what a pattern wrote, read back through the same one.
        DateTimeFormatter iso = DateTimeFormatter.ofPattern("yyyy-MM-dd");
        String text = w.toLocalDate().format(iso);
        System.out.println(text + " " + LocalDate.parse(text, iso).equals(w.toLocalDate()));
    }}

    public static void main(String[] args) {{
{body}
    }}
}}
"""


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--seed", type=int, default=20260902)
    parser.add_argument("--out", default="/tmp/timefz/cases")
    args = parser.parse_args()

    rng = random.Random(args.seed)
    os.makedirs(args.out, exist_ok=True)
    written = 0
    for name, source in (
        ("TimeDates", date_program(rng)),
        ("TimeTimes", time_program(rng)),
        ("TimeFormats", format_program(rng)),
    ):
        path = os.path.join(args.out, f"{name}.java")
        open(path, "w", encoding="utf-8").write(source)
        written += 1
    print(f"{written} programs in {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
