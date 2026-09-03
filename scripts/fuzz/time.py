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


ENUMS = ["Month", "DayOfWeek", "ChronoUnit", "ChronoField", "IsoEra"]

# Two constants of each, for the arms of a switch over it.
ENUM_ARMS = {
    "Month": ("JANUARY", "DECEMBER"),
    "DayOfWeek": ("MONDAY", "SUNDAY"),
    "ChronoUnit": ("NANOS", "FOREVER"),
    "ChronoField": ("NANO_OF_SECOND", "ERA"),
    "IsoEra": ("BCE", "CE"),
}


def position_program(rng):
    """Each `java.time` value in the POSITIONS a program puts one.

    A switch selector, an `EnumSet`, a stream's element, a `Comparable` bound.
    Every one of these was a separate refusal at some point, and each was one
    fact — "a library enum is an enum", "a library call answers a type" —
    written down in one pass and not in another. Nothing here prints an
    identity hash: a hash-ordered collection of interned constants is not
    comparable between two JVMs.
    """
    checks = []
    for simple in ENUMS:
        checks.append(
            '        for (%s v : %s.values()) { probe("switch %s " + v, () -> pick%s(v)); }'
            % (simple, simple, simple, simple)
        )
        checks.append(
            '        probe("all %s", () -> EnumSet.allOf(%s.class).size());' % (simple, simple)
        )
        checks.append(
            '        probe("none %s", () -> EnumSet.noneOf(%s.class));' % (simple, simple)
        )
        checks.append(
            '        probe("range %s", () -> EnumSet.range(%s.%s, %s.%s));'
            % (simple, simple, ENUM_ARMS[simple][0], simple, ENUM_ARMS[simple][1])
        )
        checks.append(
            '        probe("backwards %s", () -> EnumSet.range(%s.%s, %s.%s));'
            % (simple, simple, ENUM_ARMS[simple][1], simple, ENUM_ARMS[simple][0])
        )
        checks.append(
            '        probe("names %s", () -> Arrays.stream(%s.values())'
            '.map(%s::name).collect(Collectors.joining(",")));' % (simple, simple, simple)
        )
        checks.append(
            '        probe("ordinals %s", () -> Arrays.stream(%s.values())'
            '.map(v -> v.ordinal()).reduce(0, Integer::sum));' % (simple, simple)
        )
        checks.append(
            '        probe("sorted %s", () -> sortedText(new ArrayList<>('
            'Arrays.asList(%s.values()))));' % (simple, simple)
        )
        checks.append(
            '        probe("biggest %s", () -> biggest(new ArrayList<>('
            'Arrays.asList(%s.values()))));' % (simple, simple)
        )
        checks.append(
            '        probe("treeset %s", () -> new TreeSet<>('
            'Arrays.asList(%s.values())).size());' % (simple, simple)
        )
    for year, month, day in dates(rng, 24):
        checks.append(
            '        probe("date %d-%d-%d", () -> { LocalDate d = LocalDate.of(%d, %d, %d); '
            'return d.datesUntil(d.plusDays(5)).map(LocalDate::getDayOfMonth)'
            '.collect(Collectors.toList()) + " " + d.getEra() + " "'
            ' + weekend(d.getDayOfWeek()) + " " + EnumSet.of(d.getMonth(), Month.MAY) + " "'
            ' + biggest(new ArrayList<>(List.of(d, LocalDate.of(2000, 1, 1)))); });'
            % (year, month, day, year, month, day)
        )
    pickers = "\n\n".join(
        PICKER % (simple, simple, ENUM_ARMS[simple][0], ENUM_ARMS[simple][1])
        for simple in ENUMS
    )
    return POSITION_SOURCE % (pickers, "\n".join(checks))


PICKER = """    static String pick%s(%s v) {
        switch (v) {
            case %s:
                return "first";
            case %s:
                return "second";
            default:
                return "other";
        }
    }"""

POSITION_SOURCE = """import java.time.DayOfWeek;
import java.time.LocalDate;
import java.time.Month;
import java.time.chrono.IsoEra;
import java.time.temporal.ChronoField;
import java.time.temporal.ChronoUnit;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.EnumSet;
import java.util.List;
import java.util.TreeSet;
import java.util.stream.Collectors;

public class TimePositions {
    static void probe(String label, java.util.function.Supplier<Object> body) {
        try {
            System.out.println(label + " = " + body.get());
        } catch (Throwable e) {
            System.out.println(label + " ! " + e.getClass().getName() + ": " + e.getMessage());
        }
    }

    static <T extends Comparable<T>> T biggest(List<T> values) {
        T best = values.get(0);
        for (T one : values) {
            if (one.compareTo(best) > 0) {
                best = one;
            }
        }
        return best;
    }

    static String sortedText(List<? extends Comparable> values) {
        Collections.sort(values);
        return values.toString();
    }

    static String weekend(DayOfWeek day) {
        switch (day) {
            case SATURDAY:
            case SUNDAY:
                return "weekend";
            default:
                return "weekday";
        }
    }

%s

    public static void main(String[] args) {
%s
    }
}
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
        ("TimePositions", position_program(rng)),
    ):
        path = os.path.join(args.out, f"{name}.java")
        open(path, "w", encoding="utf-8").write(source)
        written += 1
    print(f"{written} programs in {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
