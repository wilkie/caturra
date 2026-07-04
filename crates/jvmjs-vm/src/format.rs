//! Java's `String.format` / `printf` engine.
//!
//! Implements the `java.util.Formatter` subset reachable from jvmjs's
//! types: conversions `b B s S c C d o x X e E f g G n % h H`, flags
//! `- + 0 , ( #` and space, argument indexes (`%2$s`), width, and
//! precision — with Java's exact error types and messages.
//!
//! Floating-point conversions round `HALF_UP` like Java (Rust's float
//! formatting rounds half-even), computed over the shortest-round-trip
//! decimal digits like Java's `BigDecimal.valueOf` path, so `%.2f` of
//! `2.675` is `2.68` here and on a real JVM alike.

use crate::value::{Heap, HeapObject, JValue};
use crate::vm::VmError;

/// A formatting argument: the static Java type (from the synthesized
/// call descriptor) plus the runtime value.
#[derive(Debug, Clone, Copy)]
pub enum FormatArg {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Char(u16),
    Boolean(bool),
    /// A string reference (or null).
    Str(Option<crate::value::HeapRef>),
}

impl FormatArg {
    /// Java class name for `IllegalFormatConversionException`.
    fn java_class(self) -> &'static str {
        match self {
            FormatArg::Int(_) => "java.lang.Integer",
            FormatArg::Long(_) => "java.lang.Long",
            FormatArg::Float(_) => "java.lang.Float",
            FormatArg::Double(_) => "java.lang.Double",
            FormatArg::Char(_) => "java.lang.Character",
            FormatArg::Boolean(_) => "java.lang.Boolean",
            FormatArg::Str(_) => "java.lang.String",
        }
    }
}

fn throw(class: &str, message: &str) -> VmError {
    VmError::UncaughtException(format!("{class}: {message}"))
}

fn unknown_conversion(conversion: char) -> VmError {
    throw(
        "java.util.UnknownFormatConversionException",
        &format!("Conversion = '{conversion}'"),
    )
}

/// One parsed `%` specifier. (Flags genuinely are independent bools —
/// Java allows almost every combination.)
#[allow(clippy::struct_excessive_bools)]
struct Spec {
    /// The original source text (for error messages).
    text: String,
    arg_index: Option<usize>,
    left_justify: bool,
    plus: bool,
    space: bool,
    zero_pad: bool,
    grouping: bool,
    parentheses: bool,
    alternate: bool,
    width: Option<usize>,
    precision: Option<usize>,
    conversion: char,
}

/// Format `template` with `args`, Java-style.
pub fn java_format(heap: &Heap, template: &str, args: &[FormatArg]) -> Result<String, VmError> {
    let chars: Vec<char> = template.chars().collect();
    let mut out = String::new();
    let mut at = 0;
    let mut next_arg = 0usize;

    while at < chars.len() {
        if chars[at] != '%' {
            out.push(chars[at]);
            at += 1;
            continue;
        }
        let spec = parse_spec(&chars, &mut at)?;
        match spec.conversion {
            '%' => out.push_str(&pad(&spec, "%")),
            'n' => out.push('\n'),
            _ => {
                let index = if let Some(explicit) = spec.arg_index {
                    explicit
                } else {
                    let index = next_arg;
                    next_arg += 1;
                    index
                };
                let arg = *args.get(index).ok_or_else(|| {
                    throw(
                        "java.util.MissingFormatArgumentException",
                        &format!("Format specifier '{}'", spec.text),
                    )
                })?;
                out.push_str(&render(heap, &spec, arg)?);
            }
        }
    }
    Ok(out)
}

fn parse_spec(chars: &[char], at: &mut usize) -> Result<Spec, VmError> {
    let start = *at;
    *at += 1; // '%'
    let mut spec = Spec {
        text: String::new(),
        arg_index: None,
        left_justify: false,
        plus: false,
        space: false,
        zero_pad: false,
        grouping: false,
        parentheses: false,
        alternate: false,
        width: None,
        precision: None,
        conversion: ' ',
    };

    // Argument index: digits followed by '$'.
    let digits_start = *at;
    while chars.get(*at).is_some_and(char::is_ascii_digit) {
        *at += 1;
    }
    if *at > digits_start && chars.get(*at) == Some(&'$') {
        let index: usize = chars[digits_start..*at]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap_or(1);
        spec.arg_index = Some(index.saturating_sub(1));
        *at += 1;
    } else {
        *at = digits_start;
    }

    // Flags.
    loop {
        match chars.get(*at) {
            Some('-') => spec.left_justify = true,
            Some('+') => spec.plus = true,
            Some(' ') => spec.space = true,
            Some('0') => spec.zero_pad = true,
            Some(',') => spec.grouping = true,
            Some('(') => spec.parentheses = true,
            Some('#') => spec.alternate = true,
            _ => break,
        }
        *at += 1;
    }

    // Width.
    let width_start = *at;
    while chars.get(*at).is_some_and(char::is_ascii_digit) {
        *at += 1;
    }
    if *at > width_start {
        spec.width = chars[width_start..*at]
            .iter()
            .collect::<String>()
            .parse()
            .ok();
    }

    // Precision.
    if chars.get(*at) == Some(&'.') {
        *at += 1;
        let precision_start = *at;
        while chars.get(*at).is_some_and(char::is_ascii_digit) {
            *at += 1;
        }
        spec.precision = Some(
            chars[precision_start..*at]
                .iter()
                .collect::<String>()
                .parse()
                .unwrap_or(0),
        );
    }

    let conversion = *chars.get(*at).ok_or_else(|| unknown_conversion('%'))?;
    *at += 1;
    spec.conversion = conversion;
    spec.text = chars[start..*at].iter().collect();
    Ok(spec)
}

/// Apply width padding (spaces; the numeric zero-pad happens earlier).
fn pad(spec: &Spec, body: &str) -> String {
    let width = spec.width.unwrap_or(0);
    let len = body.chars().count();
    if len >= width {
        return body.to_owned();
    }
    let padding = " ".repeat(width - len);
    if spec.left_justify {
        format!("{body}{padding}")
    } else {
        format!("{padding}{body}")
    }
}

/// Numeric padding: honors `0` (after the sign) unless left-justified.
fn pad_numeric(spec: &Spec, sign: &str, magnitude: &str) -> String {
    let width = spec.width.unwrap_or(0);
    let len = sign.chars().count() + magnitude.chars().count();
    if spec.zero_pad && !spec.left_justify && len < width {
        let zeros = "0".repeat(width - len);
        return format!("{sign}{zeros}{magnitude}");
    }
    pad(spec, &format!("{sign}{magnitude}"))
}

/// Insert `,` groupings into an integer digit run.
fn group_digits(digits: &str) -> String {
    let chars: Vec<char> = digits.chars().collect();
    let mut out = String::new();
    for (position, c) in chars.iter().enumerate() {
        if position > 0 && (chars.len() - position).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*c);
    }
    out
}

fn conversion_mismatch(conversion: char, arg: FormatArg) -> VmError {
    throw(
        "java.util.IllegalFormatConversionException",
        &format!("{conversion} != {}", arg.java_class()),
    )
}

#[allow(clippy::too_many_lines)] // one arm per conversion
fn render(heap: &Heap, spec: &Spec, arg: FormatArg) -> Result<String, VmError> {
    let conversion = spec.conversion;
    match conversion.to_ascii_lowercase() {
        's' => {
            let mut text = match arg {
                FormatArg::Str(Some(reference)) => heap.string_text(reference).unwrap_or_default(),
                FormatArg::Str(None) => String::from("null"),
                FormatArg::Int(v) => v.to_string(),
                FormatArg::Long(v) => v.to_string(),
                FormatArg::Float(v) => crate::intrinsics::java_float_to_string(v),
                FormatArg::Double(v) => crate::intrinsics::java_double_to_string(v),
                FormatArg::Char(u) => char::from_u32(u32::from(u))
                    .unwrap_or('\u{FFFD}')
                    .to_string(),
                FormatArg::Boolean(b) => b.to_string(),
            };
            if let Some(precision) = spec.precision {
                text = text.chars().take(precision).collect();
            }
            if conversion == 'S' {
                text = text.to_uppercase();
            }
            Ok(pad(spec, &text))
        }
        'b' => {
            let value = match arg {
                FormatArg::Boolean(b) => b,
                FormatArg::Str(None) => false,
                // Java: any non-null non-Boolean argument is true.
                _ => true,
            };
            let mut text = value.to_string();
            if conversion == 'B' {
                text = text.to_uppercase();
            }
            Ok(pad(spec, &text))
        }
        'h' => {
            let hash = match arg {
                FormatArg::Str(None) => return Ok(pad(spec, "null")),
                FormatArg::Str(Some(reference)) => match heap.get(reference) {
                    Some(HeapObject::JavaString(units)) => {
                        let mut hash: i32 = 0;
                        for unit in units {
                            hash = hash.wrapping_mul(31).wrapping_add(i32::from(*unit));
                        }
                        hash
                    }
                    _ => reference.cast_signed(),
                },
                FormatArg::Int(v) => v,
                FormatArg::Long(v) => (((v.cast_unsigned() ^ (v.cast_unsigned() >> 32))
                    & 0xFFFF_FFFF) as u32)
                    .cast_signed(),
                FormatArg::Float(v) => {
                    let bits = if v.is_nan() {
                        0x7FC0_0000_u32
                    } else {
                        v.to_bits()
                    };
                    bits.cast_signed()
                }
                FormatArg::Double(v) => crate::intrinsics::java_double_hash_public(v),
                FormatArg::Char(u) => i32::from(u),
                FormatArg::Boolean(b) => {
                    if b {
                        1231
                    } else {
                        1237
                    }
                }
            };
            let mut text = format!("{:x}", hash.cast_unsigned());
            if conversion == 'H' {
                text = text.to_uppercase();
            }
            Ok(pad(spec, &text))
        }
        'c' => {
            let unit = match arg {
                FormatArg::Char(u) => u32::from(u),
                FormatArg::Int(v) => u32::try_from(v).map_err(|_| {
                    throw(
                        "java.util.IllegalFormatException",
                        &format!("Code point = {v}"),
                    )
                })?,
                other => return Err(conversion_mismatch(conversion, other)),
            };
            let mut text = char::from_u32(unit).unwrap_or('\u{FFFD}').to_string();
            if conversion == 'C' {
                text = text.to_uppercase();
            }
            Ok(pad(spec, &text))
        }
        'd' => {
            let value = match arg {
                FormatArg::Int(v) => i64::from(v),
                FormatArg::Long(v) => v,
                other => return Err(conversion_mismatch(conversion, other)),
            };
            let negative = value < 0;
            let mut magnitude = value.unsigned_abs().to_string();
            if spec.grouping {
                magnitude = group_digits(&magnitude);
            }
            if negative && spec.parentheses {
                return Ok(pad(spec, &format!("({magnitude})")));
            }
            let sign = if negative {
                "-"
            } else if spec.plus {
                "+"
            } else if spec.space {
                " "
            } else {
                ""
            };
            Ok(pad_numeric(spec, sign, &magnitude))
        }
        'o' | 'x' => {
            let value = match arg {
                FormatArg::Int(v) => u64::from(v.cast_unsigned()),
                FormatArg::Long(v) => v.cast_unsigned(),
                other => return Err(conversion_mismatch(conversion, other)),
            };
            let mut text = match conversion.to_ascii_lowercase() {
                'o' => format!("{value:o}"),
                _ => format!("{value:x}"),
            };
            if spec.alternate {
                text = match conversion.to_ascii_lowercase() {
                    'o' => format!("0{text}"),
                    _ => format!("0x{text}"),
                };
            }
            if conversion == 'X' {
                text = text.to_uppercase();
            }
            Ok(pad_numeric(spec, "", &text))
        }
        'f' | 'e' | 'g' => {
            let value = match arg {
                FormatArg::Double(v) => v,
                // Java's Formatter widens Float via doubleValue().
                FormatArg::Float(v) => f64::from(v),
                other => return Err(conversion_mismatch(conversion, other)),
            };
            let text = format_float(spec, value);
            Ok(if conversion.is_ascii_uppercase() {
                // Uppercases the exponent marker and Infinity/NaN.
                let body = text.to_uppercase();
                pad_sign_aware(spec, value.is_sign_negative() && !value.is_nan(), &body)
            } else {
                pad_sign_aware(spec, value.is_sign_negative() && !value.is_nan(), &text)
            })
        }
        _ => Err(unknown_conversion(conversion)),
    }
}

/// Width handling for floats: the body already contains its sign, so
/// zero-padding must go after it.
fn pad_sign_aware(spec: &Spec, _negative: bool, body: &str) -> String {
    if spec.zero_pad && !spec.left_justify {
        let width = spec.width.unwrap_or(0);
        let len = body.chars().count();
        if len < width {
            let (sign, magnitude) = if let Some(rest) = body.strip_prefix('-') {
                ("-", rest)
            } else if let Some(rest) = body.strip_prefix('+') {
                ("+", rest)
            } else {
                ("", body)
            };
            let zeros = "0".repeat(width - len);
            return format!("{sign}{zeros}{magnitude}");
        }
    }
    pad(spec, body)
}

/// Format a finite/infinite/NaN double per the conversion.
fn format_float(spec: &Spec, value: f64) -> String {
    if value.is_nan() {
        return String::from("NaN");
    }
    if value.is_infinite() {
        return String::from(if value > 0.0 { "Infinity" } else { "-Infinity" });
    }

    let sign = if value.is_sign_negative() {
        "-"
    } else if spec.plus {
        "+"
    } else if spec.space {
        " "
    } else {
        ""
    };
    // Parenthesized negatives.
    let use_parens = spec.parentheses && value.is_sign_negative();
    let sign = if use_parens { "" } else { sign };

    let body = match spec.conversion.to_ascii_lowercase() {
        'f' => {
            let precision = spec.precision.unwrap_or(6);
            let mut text = fixed_digits(value.abs(), precision);
            if spec.grouping {
                let (int_part, frac_part) = text
                    .split_once('.')
                    .map_or((text.as_str(), None), |(i, f)| (i, Some(f)));
                let grouped = group_digits(int_part);
                text = match frac_part {
                    Some(frac) => format!("{grouped}.{frac}"),
                    None => grouped,
                };
            }
            text
        }
        'e' => {
            let precision = spec.precision.unwrap_or(6);
            scientific_digits(value.abs(), precision)
        }
        _ => {
            // %g: precision counts significant digits.
            let precision = match spec.precision {
                Some(0) => 1,
                Some(p) => p,
                None => 6,
            };
            general_digits(value.abs(), precision)
        }
    };

    if use_parens {
        format!("({body})")
    } else {
        format!("{sign}{body}")
    }
}

/// The decimal digits Java's Formatter rounds: the SHORTEST
/// round-trip representation (Java routes doubles through
/// `BigDecimal.valueOf`, i.e. `Double.toString`), not the exact binary
/// expansion — `%.2f` of `2.675` is `2.68` even though the double is
/// exactly 2.674999…82. Returns `(digits, point)`: the value is
/// `0.digits × 10^point`.
fn shortest_decimal(value: f64) -> (Vec<u8>, i32) {
    if value == 0.0 {
        return (vec![], 0);
    }
    let text = format!("{:e}", value.abs());
    let (mantissa, exponent) = text.split_once('e').expect("exponential form");
    let exponent: i32 = exponent.parse().expect("numeric exponent");
    let digits: Vec<u8> = mantissa
        .bytes()
        .filter(u8::is_ascii_digit)
        .map(|b| b - b'0')
        .collect();
    (digits, exponent + 1)
}

/// Round the (most-significant-first) digits `HALF_UP` at `keep` digits,
/// returning whether the leading digit gained a position (99.5 → 100).
fn round_half_up(digits: &mut Vec<u8>, keep: usize) -> bool {
    if digits.len() <= keep {
        return false;
    }
    let round_up = digits[keep] >= 5;
    digits.truncate(keep);
    if !round_up {
        return false;
    }
    for digit in digits.iter_mut().rev() {
        if *digit == 9 {
            *digit = 0;
        } else {
            *digit += 1;
            return false;
        }
    }
    digits.insert(0, 1);
    true
}

/// `%f`: fixed-point with exactly `precision` fraction digits.
fn fixed_digits(value: f64, precision: usize) -> String {
    let (mut digits, mut point) = shortest_decimal(value);
    // Total digits to keep: point + precision (fraction digits after
    // the decimal point).
    let keep = point + i32::try_from(precision).unwrap_or(0);
    if keep <= 0 {
        // Rounds to zero unless the first digit rounds up.
        let first_kept = if keep == 0 {
            digits.first().copied()
        } else {
            None
        };
        let mut out = String::from("0");
        if precision > 0 {
            out.push('.');
            for _ in 0..precision {
                out.push('0');
            }
        }
        if first_kept.is_some_and(|d| d >= 5) && precision > 0 {
            // 0.00…1 case: last digit becomes 1.
            out.pop();
            out.push('1');
        } else if first_kept.is_some_and(|d| d >= 5) {
            return String::from("1");
        }
        return out;
    }
    let keep = usize::try_from(keep).expect("positive");
    if round_half_up(&mut digits, keep) {
        point += 1;
    }
    while digits.len() < keep {
        digits.push(0);
    }
    render_fixed(&digits, point, precision)
}

fn render_fixed(digits: &[u8], point: i32, precision: usize) -> String {
    let mut out = String::new();
    if point <= 0 {
        out.push('0');
    } else {
        for index in 0..usize::try_from(point).expect("positive") {
            out.push(char::from(b'0' + digits.get(index).copied().unwrap_or(0)));
        }
    }
    if precision > 0 {
        out.push('.');
        for offset in 0..precision {
            let index = i32::try_from(offset).unwrap_or(0) + point;
            let digit = if index < 0 {
                0
            } else {
                usize::try_from(index)
                    .ok()
                    .and_then(|i| digits.get(i))
                    .copied()
                    .unwrap_or(0)
            };
            out.push(char::from(b'0' + digit));
        }
    }
    out
}

/// `%e`: scientific with `precision` fraction digits and a two-digit
/// (minimum) exponent.
fn scientific_digits(value: f64, precision: usize) -> String {
    let (mut digits, point) = shortest_decimal(value);
    if digits.is_empty() {
        let mut out = String::from("0");
        if precision > 0 {
            out.push('.');
            out.push_str(&"0".repeat(precision));
        }
        out.push_str("e+00");
        return out;
    }
    let mut exponent = point - 1;
    if round_half_up(&mut digits, precision + 1) {
        exponent += 1;
    }
    while digits.len() < precision + 1 {
        digits.push(0);
    }
    let mut out = String::new();
    out.push(char::from(b'0' + digits[0]));
    if precision > 0 {
        out.push('.');
        for digit in &digits[1..=precision] {
            out.push(char::from(b'0' + digit));
        }
    }
    let sign = if exponent < 0 { '-' } else { '+' };
    out.push('e');
    out.push(sign);
    let _ = std::fmt::Write::write_fmt(&mut out, format_args!("{:02}", exponent.abs()));
    out
}

/// `%g`: `precision` significant digits, fixed or scientific by
/// Java's exponent rule.
fn general_digits(value: f64, precision: usize) -> String {
    let (mut digits, point) = shortest_decimal(value);
    if digits.is_empty() {
        let mut out = String::from("0.");
        out.push_str(&"0".repeat(precision.saturating_sub(1).max(1)));
        return out;
    }
    let mut exponent = point - 1;
    if round_half_up(&mut digits, precision) {
        exponent += 1;
    }
    while digits.len() < precision {
        digits.push(0);
    }
    let precision_i = i32::try_from(precision).unwrap_or(6);
    if exponent >= -4 && exponent < precision_i {
        // Fixed notation with (precision - 1 - exponent) fraction digits.
        let fraction = usize::try_from(precision_i - 1 - exponent).unwrap_or(0);
        render_fixed(&digits, exponent + 1, fraction)
    } else {
        let mut out = String::new();
        out.push(char::from(b'0' + digits[0]));
        if precision > 1 {
            out.push('.');
            for digit in &digits[1..precision] {
                out.push(char::from(b'0' + digit));
            }
        }
        let sign = if exponent < 0 { '-' } else { '+' };
        out.push('e');
        out.push(sign);
        let _ = std::fmt::Write::write_fmt(&mut out, format_args!("{:02}", exponent.abs()));
        out
    }
}

/// Decode format arguments from a synthesized call descriptor: the
/// characters after the leading format-string parameter tag each
/// argument (`I`, `D`, `C`, `Z`, or a string reference).
pub fn args_from_descriptor(
    descriptor: &str,
    values: &[JValue],
) -> Result<Vec<FormatArg>, VmError> {
    let inner = descriptor
        .strip_prefix("(Ljava/lang/String;")
        .and_then(|rest| rest.split_once(')'))
        .map(|(args, _)| args)
        .unwrap_or_default();
    let mut tags = Vec::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            'I' | 'D' | 'C' | 'Z' | 'J' | 'F' => tags.push(c),
            'L' => {
                for inner_char in chars.by_ref() {
                    if inner_char == ';' {
                        break;
                    }
                }
                tags.push('L');
            }
            _ => {}
        }
    }
    let mut args = Vec::with_capacity(tags.len());
    for (tag, value) in tags.iter().zip(values) {
        args.push(match (tag, value) {
            ('I', JValue::Int(v)) => FormatArg::Int(*v),
            ('C', JValue::Int(v)) => FormatArg::Char(u16::try_from(*v).unwrap_or(0)),
            ('Z', JValue::Int(v)) => FormatArg::Boolean(*v != 0),
            ('D', JValue::Double(v)) => FormatArg::Double(*v),
            ('J', JValue::Long(v)) => FormatArg::Long(*v),
            ('F', JValue::Float(v)) => FormatArg::Float(*v),
            ('L', JValue::Ref(reference)) => FormatArg::Str(*reference),
            _ => {
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.VerifyError: malformed format call",
                )));
            }
        });
    }
    Ok(args)
}
