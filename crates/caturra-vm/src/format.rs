//! Java's `String.format` / `printf` engine.
//!
//! Implements the `java.util.Formatter` subset reachable from caturra's
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
    Short(i16),
    Byte(i8),
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
    /// The class a diagnostic names this argument by. A reference is asked
    /// what it IS: naming every one of them `java.lang.String` told a program
    /// that `%d` had been given a String when it had been given a `Pet`.
    fn java_class(self, heap: &Heap) -> String {
        match self {
            FormatArg::Int(_) => String::from("java.lang.Integer"),
            FormatArg::Short(_) => String::from("java.lang.Short"),
            FormatArg::Byte(_) => String::from("java.lang.Byte"),
            FormatArg::Long(_) => String::from("java.lang.Long"),
            FormatArg::Float(_) => String::from("java.lang.Float"),
            FormatArg::Double(_) => String::from("java.lang.Double"),
            FormatArg::Char(_) => String::from("java.lang.Character"),
            FormatArg::Boolean(_) => String::from("java.lang.Boolean"),
            FormatArg::Str(None) => String::from("java.lang.String"),
            FormatArg::Str(Some(reference)) => {
                crate::interpreter::heap_binary_name(heap, reference)
            }
        }
    }
}

fn throw(class: &str, message: &str) -> VmError {
    VmError::UncaughtException(format!("{class}: {message}"))
}

/// Every suffix `java.util.Formatter.DateTime` accepts.
const DATE_TIME_SUFFIXES: &str = "HIklMSLNpzZsQBbhAaCYyjmdeRTrDFc";

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
    /// The `<` relative index: reuse the previous specifier's argument.
    relative: bool,
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
    /// The SUFFIX of a `%t`/`%T` date-time conversion (`%tY` carries `Y`).
    /// The conversion itself stays `t`/`T`, which is how a JDK stores it and
    /// why its diagnostics name the suffix as "the conversion".
    date_time: Option<char>,
}

impl Spec {
    /// The specifier as a JDK REPORTS it — which is not the text the program
    /// wrote. `FormatSpecifier.toString` rebuilds it from the parsed parts:
    /// the flags first, in their own canonical order (`-#+ 0,(<`), and only
    /// then the argument index. So `%3$,.2f` comes back as `%,3$.2f` and
    /// `%2$012.4f` as `%02$12.4f`. Echoing the source text was right for every
    /// specifier without an index and wrong for every one with one.
    fn java_text(&self) -> String {
        use std::fmt::Write as _;

        let mut text = String::from("%");
        for (present, flag) in [
            (self.left_justify, '-'),
            (self.alternate, '#'),
            (self.plus, '+'),
            (self.space, ' '),
            (self.zero_pad, '0'),
            (self.grouping, ','),
            (self.parentheses, '('),
            (self.relative, '<'),
        ] {
            if present {
                text.push(flag);
            }
        }
        if let Some(index) = self.arg_index {
            // Stored zero-based, written the way it was read.
            let _ = write!(text, "{}$", index + 1);
        }
        if let Some(width) = self.width {
            text.push_str(&width.to_string());
        }
        if let Some(precision) = self.precision {
            let _ = write!(text, ".{precision}");
        }
        if let Some(suffix) = self.date_time {
            text.push(self.conversion);
            text.push(suffix);
        } else {
            text.push(self.conversion);
        }
        text
    }

    /// The first flag written, in the order the JDK reports them for the
    /// conversions that accept none at all (`%n`, `%%`).
    fn first_flag(&self) -> Option<char> {
        [
            (self.left_justify, '-'),
            (self.alternate, '#'),
            (self.plus, '+'),
            (self.space, ' '),
            (self.zero_pad, '0'),
            (self.grouping, ','),
            (self.parentheses, '('),
        ]
        .into_iter()
        .find_map(|(present, flag)| present.then_some(flag))
    }
}

/// A format call's arguments.
///
/// `all_null` marks the JDK's odd corner: `format(fmt, (Object[]) null)` passes
/// a null ARRAY, and the Formatter's `getArg` then answers null for every
/// index — so `"%s %s"` prints "null null" rather than running out of
/// arguments, and even `%2$s` is null.
pub struct FormatArgs {
    pub values: Vec<FormatArg>,
    pub all_null: bool,
}

/// Format `template` with `args`, Java-style — discarding whatever was
/// produced before a failure. Callers that WRITE the result as they go (a
/// `printf` to a stream) want [`java_format_partial`] instead, because the
/// JDK's Formatter appends to its destination one specifier at a time and so
/// leaves the prefix visible when a later one throws.
pub fn java_format(heap: &Heap, template: &str, args: &FormatArgs) -> Result<String, VmError> {
    java_format_partial(heap, template, args).1
}

/// [`java_format`], also handing back the text produced before any failure.
pub fn java_format_partial(
    heap: &Heap,
    template: &str,
    args: &FormatArgs,
) -> (String, Result<String, VmError>) {
    let mut produced = String::new();
    let result = java_format_inner(heap, template, args, &mut produced);
    (produced, result)
}

fn java_format_inner(
    heap: &Heap,
    template: &str,
    args: &FormatArgs,
    produced: &mut String,
) -> Result<String, VmError> {
    let chars: Vec<char> = template.chars().collect();
    // A JDK parses the WHOLE template before it renders any of it — every
    // `FormatSpecifier` validates its own flags as it is constructed — so a
    // flag error in a LATER specifier is reported before an earlier one has
    // even looked at its argument, and before `printf` writes anything at all.
    // Validating as we went reported the first RENDER error instead:
    // `printf("%c %-o", -1, 7)` said the code point was illegal where a JDK
    // says the `%-o` has no width.
    let mut scan = 0;
    while scan < chars.len() {
        if chars[scan] != '%' {
            scan += 1;
            continue;
        }
        let spec = parse_spec(&chars, &mut scan)?;
        validate_spec(&spec)?;
    }
    let mut out = String::new();
    let mut at = 0;
    let mut cursor = ArgCursor::default();

    while at < chars.len() {
        if chars[at] != '%' {
            out.push(chars[at]);
            produced.push(chars[at]);
            at += 1;
            continue;
        }
        let spec = parse_spec(&chars, &mut at)?;
        match spec.conversion {
            '%' => {
                let text = pad(&spec, "%");
                out.push_str(&text);
                produced.push_str(&text);
            }
            'n' => {
                out.push('\n');
                produced.push('\n');
            }
            _ => {
                let index = cursor.index(&spec)?;
                // A null argument ARRAY answers null for every index, however
                // many specifiers the template has.
                let arg = if args.all_null {
                    FormatArg::Str(None)
                } else {
                    *args.values.get(index).ok_or_else(|| {
                        throw(
                            "java.util.MissingFormatArgumentException",
                            &format!("Format specifier '{}'", spec.java_text()),
                        )
                    })?
                };
                // AFTER the argument is fetched: a missing one is reported
                // before a flag that needs an argument to judge.
                validate_with_argument(&spec)?;
                let text = render(heap, &spec, arg)?;
                out.push_str(&text);
                produced.push_str(&text);
            }
        }
    }
    Ok(out)
}

/// Which argument a conversion consumes: the next one, the one an explicit
/// `2$` names, or — for `%<` — the one the previous conversion took.
///
/// Written out inside the render loop, it could not be asked ahead of the
/// render; the interpreter needs exactly this to know which arguments a
/// template will want the TEXT of (see [`argument_needs`]), and asking it a
/// second way is how the two would come to disagree about `%<d`.
#[derive(Default)]
struct ArgCursor {
    next: usize,
    /// The index the previous conversion consumed, for the `<` relative index.
    last: usize,
    /// Whether any conversion has consumed an argument yet — a leading `%<`
    /// has nothing to reuse.
    consumed: bool,
}

impl ArgCursor {
    fn index(&mut self, spec: &Spec) -> Result<usize, VmError> {
        // `%<s` with nothing before it: there is no previous argument to
        // reuse. The JDK reports the specifier as a missing argument, so a
        // leading relative index is an error.
        if spec.relative && !self.consumed {
            return Err(throw(
                "java.util.MissingFormatArgumentException",
                &format!("Format specifier '{}'", spec.java_text()),
            ));
        }
        let index = if spec.relative {
            self.last
        } else if let Some(explicit) = spec.arg_index {
            explicit
        } else {
            let index = self.next;
            self.next += 1;
            index
        };
        self.last = index;
        self.consumed = true;
        Ok(index)
    }
}

/// What a template will ASK of each argument — the question the interpreter
/// has to answer before the formatter runs, because the formatter sees only
/// the heap and cannot call a user `toString()` or `hashCode()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgNeed {
    /// Nothing: the conversion either rejects a reference outright (`%d` of an
    /// object is an error naming the object's CLASS — the JDK never asks it
    /// for text) or answers without asking (`%b` is "true" for any non-null).
    Nothing,
    /// Its `toString()` — `%s` and `%S`.
    Text,
    /// Its `hashCode()` — `%h` and `%H`.
    Hash,
    /// Both, when one argument is consumed by a `%s` and a `%h` alike.
    Both,
}

/// What the arguments will be asked for by `template`, in order.
///
/// Sized by what the TEMPLATE asks for, not by how many values were passed: a
/// forwarded varargs array arrives as ONE value whose elements are all the
/// arguments, so a length taken from the call would have covered only the
/// first of them.
///
/// Rendering every heap object to text up front was invisible until the
/// conversion was one that never wanted text: `String.format("%d", pet)` ran
/// `Pet.toString()` (observable when it throws, or has a side effect) and then
/// reported the mismatch against `java.lang.String` rather than against `Pet`,
/// and `%h` hashed the TEXT instead of the object.
#[must_use]
pub fn argument_needs(template: &str) -> Vec<ArgNeed> {
    let chars: Vec<char> = template.chars().collect();
    let mut needs: Vec<ArgNeed> = Vec::new();
    let mut at = 0;
    let mut cursor = ArgCursor::default();
    while at < chars.len() {
        if chars[at] != '%' {
            at += 1;
            continue;
        }
        let Ok(spec) = parse_spec(&chars, &mut at) else {
            break;
        };
        if matches!(spec.conversion, '%' | 'n') {
            continue;
        }
        let Ok(index) = cursor.index(&spec) else {
            break;
        };
        let need = match spec.conversion {
            's' | 'S' => ArgNeed::Text,
            'h' | 'H' => ArgNeed::Hash,
            _ => ArgNeed::Nothing,
        };
        if needs.len() <= index {
            needs.resize(index + 1, ArgNeed::Nothing);
        }
        needs[index] = match (needs[index], need) {
            (ArgNeed::Nothing, other) | (other, ArgNeed::Nothing) => other,
            (a, b) if a == b => a,
            // A `%s` and a `%h` over the same argument want different things
            // of it, and both have to be taken before the formatter runs.
            _ => ArgNeed::Both,
        };
    }
    needs
}

#[allow(clippy::too_many_lines)] // one part of the specifier grammar per block
fn parse_spec(chars: &[char], at: &mut usize) -> Result<Spec, VmError> {
    let start = *at;
    *at += 1; // '%'
    let mut spec = Spec {
        text: String::new(),
        arg_index: None,
        relative: false,
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
        date_time: None,
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

    // Flags. A repeated flag is a `DuplicateFormatFlagsException` — each is a
    // single bit here, so the second occurrence would otherwise vanish
    // silently. `<` is a positional flag and may not repeat either.
    while let Some(c @ ('-' | '+' | ' ' | '0' | ',' | '(' | '#' | '<')) = chars.get(*at).copied() {
        let flag = c;
        let already = match flag {
            '-' => spec.left_justify,
            '+' => spec.plus,
            ' ' => spec.space,
            '0' => spec.zero_pad,
            ',' => spec.grouping,
            '(' => spec.parentheses,
            '#' => spec.alternate,
            '<' => spec.relative,
            _ => false,
        };
        if already {
            return Err(throw(
                "java.util.DuplicateFormatFlagsException",
                &format!("Flags = '{flag}'"),
            ));
        }
        match flag {
            '-' => spec.left_justify = true,
            '+' => spec.plus = true,
            ' ' => spec.space = true,
            '0' => spec.zero_pad = true,
            ',' => spec.grouping = true,
            '(' => spec.parentheses = true,
            '#' => spec.alternate = true,
            '<' => spec.relative = true,
            _ => {}
        }
        *at += 1;
    }

    // Width.
    let width_start = *at;
    while chars.get(*at).is_some_and(char::is_ascii_digit) {
        *at += 1;
    }
    if *at > width_start {
        // A width that does not fit an `int` is IGNORED, not clamped: the JDK
        // parses it with `Integer.parseInt` and catches the failure, leaving no
        // width at all. Clamping built a 2-billion-character string instead.
        let digits: String = chars[width_start..*at].iter().collect();
        spec.width = digits
            .parse::<i32>()
            .ok()
            .and_then(|w| usize::try_from(w).ok());
    }

    // Precision. A '.' with no DIGITS after it is not a precision at all: the
    // JDK's specifier pattern stops before it, so the '.' becomes the
    // conversion character and `%.f` is an unknown conversion — where caturra
    // read an empty precision and silently formatted.
    if chars.get(*at) == Some(&'.') && chars.get(*at + 1).is_some_and(char::is_ascii_digit) {
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
    // A specifier the JDK's pattern does not match at all — `%5.d`, where the
    // '.' begins a precision with no digits — is reported against the
    // character right after the '%', not against wherever the parse gave up.
    if !conversion.is_ascii_alphabetic() && conversion != '%' {
        let after_percent = chars.get(start + 1).copied().unwrap_or('%');
        return Err(unknown_conversion(after_percent));
    }
    *at += 1;
    spec.conversion = conversion;
    // `%tY` / `%TY` — a date-time conversion is TWO characters, and the second
    // is what a JDK's diagnostics call the conversion. An unknown suffix names
    // both (`Conversion = 'tw'`); a `%t` at the end of the template names just
    // the `t`.
    if matches!(conversion, 't' | 'T') {
        let suffix = *chars
            .get(*at)
            .ok_or_else(|| unknown_conversion(conversion))?;
        *at += 1;
        if !DATE_TIME_SUFFIXES.contains(suffix) {
            return Err(throw(
                "java.util.UnknownFormatConversionException",
                &format!("Conversion = '{conversion}{suffix}'"),
            ));
        }
        spec.date_time = Some(suffix);
    }
    spec.text = chars[start..*at].iter().collect();
    Ok(spec)
}

/// Reject flag / precision combinations Java's Formatter rejects at run time
/// (before the argument is even looked at), with its exact exception types and
/// messages. caturra used to render these silently — the accept-invalid
/// direction. Only the widely-hit rules are enforced; an omission renders as
/// before, never a spurious throw.
#[allow(clippy::too_many_lines)] // one flag rule per conversion family
fn validate_spec(spec: &Spec) -> Result<(), VmError> {
    let c = spec.conversion;
    // `%n` and `%%` take no width or precision.
    // A flag is illegal for BOTH of them; a width and a precision are illegal
    // for `%n`, and only a precision for `%%` (which may be padded).
    if c == 'n' || c == '%' {
        if let Some(flag) = spec.first_flag() {
            return Err(illegal_format_flags(&flag.to_string()));
        }
        if let Some(precision) = spec.precision {
            return Err(throw(
                "java.util.IllegalFormatPrecisionException",
                &precision.to_string(),
            ));
        }
        if c == 'n'
            && let Some(width) = spec.width
        {
            return Err(throw(
                "java.util.IllegalFormatWidthException",
                &width.to_string(),
            ));
        }
        return Ok(());
    }
    let lower = c.to_ascii_lowercase();
    // An unknown conversion is reported BEFORE anything else about the
    // specifier — including before the argument is fetched, which is why
    // `printf("%q")` is an UnknownFormatConversionException and not a
    // missing-argument one.
    if !matches!(
        lower,
        's' | 'b' | 'h' | 'c' | 'd' | 'o' | 'x' | 'e' | 'f' | 'g' | 'a' | 't'
    ) {
        return Err(throw(
            "java.util.UnknownFormatConversionException",
            &format!("Conversion = '{c}'"),
        ));
    }

    // The JDK reports the LOWERCASE conversion here: `%,E` says `Conversion =
    // e`, because an uppercase conversion is the lowercase one plus a flag.
    let mismatch = |flag: char| -> VmError {
        throw(
            "java.util.FormatFlagsConversionMismatchException",
            &format!("Conversion = {lower}, Flags = {flag}"),
        )
    };
    let missing_width =
        || -> VmError { throw("java.util.MissingFormatWidthException", &spec.java_text()) };
    let bad_precision = || -> VmError {
        throw(
            "java.util.IllegalFormatPrecisionException",
            &spec.precision.unwrap_or(0).to_string(),
        )
    };
    // The ORDER of these checks is observable: a specifier can be wrong in
    // two ways at once, and which exception a program sees is the one the JDK
    // reaches first. Each family below follows `java.util.Formatter`'s own
    // sequence, which is not the same sequence twice.
    let flags_in_order = |flags: &[char]| -> Result<(), VmError> {
        for flag in flags {
            let present = match flag {
                '+' => spec.plus,
                ' ' => spec.space,
                '0' => spec.zero_pad,
                ',' => spec.grouping,
                '(' => spec.parentheses,
                _ => spec.alternate,
            };
            if present {
                return Err(mismatch(*flag));
            }
        }
        Ok(())
    };
    // The two flag pairs that contradict each other. `checkNumeric` looks at
    // them for the numeric families; the general ones never reach it.
    let contradictions = || -> Result<(), VmError> {
        if spec.left_justify && spec.zero_pad {
            return Err(illegal_format_flags("-0"));
        }
        if spec.plus && spec.space {
            return Err(illegal_format_flags("+ "));
        }
        Ok(())
    };

    match lower {
        // General: `#` is wrong for a boolean or a hash BEFORE the width rule,
        // and `#` with `s` is not decided here at all — it depends on the
        // ARGUMENT (a `Formattable` accepts it), so it waits for one.
        's' | 'b' | 'h' => {
            if matches!(lower, 'b' | 'h') && spec.alternate {
                return Err(mismatch('#'));
            }
            if spec.width.is_none() && spec.left_justify {
                return Err(missing_width());
            }
            flags_in_order(&['+', ' ', '0', ',', '('])?;
        }
        // Character: the precision is wrong before any flag is, and the
        // missing width is last.
        'c' => {
            if spec.precision.is_some() {
                return Err(bad_precision());
            }
            flags_in_order(&['+', ' ', '0', ',', '(', '#'])?;
            if spec.width.is_none() && spec.left_justify {
                return Err(missing_width());
            }
        }
        // Integers: the width rule, then the contradictions, then the
        // precision — and only then the one flag each radix rejects. The
        // signed flags on `%o`/`%x` are not rejected here: they wait for the
        // argument, as a JDK's `print(long)` does.
        'd' | 'o' | 'x' => {
            if spec.width.is_none() && (spec.left_justify || spec.zero_pad) {
                return Err(missing_width());
            }
            contradictions()?;
            if spec.precision.is_some() {
                return Err(bad_precision());
            }
            if lower == 'd' {
                flags_in_order(&['#'])?;
            } else {
                flags_in_order(&[','])?;
            }
        }
        // Floats: the same numeric preamble, then the one flag each rejects.
        'e' | 'f' | 'g' | 'a' => {
            if spec.width.is_none() && (spec.left_justify || spec.zero_pad) {
                return Err(missing_width());
            }
            contradictions()?;
            match lower {
                'e' => flags_in_order(&[','])?,
                'g' => flags_in_order(&['#'])?,
                'a' => flags_in_order(&[',', '('])?,
                _ => {}
            }
        }
        // A date-time conversion takes only `-`, and its diagnostics name the
        // SUFFIX as the conversion, since that is the character that chose the
        // format.
        _ => {
            if spec.width.is_none() && spec.left_justify {
                return Err(missing_width());
            }
            let suffix = spec.date_time.unwrap_or(spec.conversion);
            for (present, flag) in [
                (spec.grouping, ','),
                (spec.plus, '+'),
                (spec.space, ' '),
                (spec.parentheses, '('),
                (spec.zero_pad, '0'),
                (spec.alternate, '#'),
            ] {
                if present {
                    return Err(throw(
                        "java.util.FormatFlagsConversionMismatchException",
                        &format!("Conversion = {suffix}, Flags = {flag}"),
                    ));
                }
            }
            if spec.precision.is_some() {
                return Err(bad_precision());
            }
        }
    }
    Ok(())
}

/// The checks a JDK makes when the ARGUMENT arrives rather than when the
/// specifier is read: `%#s` is legal for a `Formattable` (caturra has none),
/// and the signed flags on `%o`/`%x` are rejected by the integer printer. They
/// are separate because the ORDER is observable — `"%#.0s %,(3.0b"` blames the
/// `b`, since every specifier is read before any argument is looked at.
fn validate_with_argument(spec: &Spec) -> Result<(), VmError> {
    let lower = spec.conversion.to_ascii_lowercase();
    let mismatch = |flag: char| -> VmError {
        throw(
            "java.util.FormatFlagsConversionMismatchException",
            &format!("Conversion = {lower}, Flags = {flag}"),
        )
    };
    if lower == 's' && spec.alternate {
        return Err(mismatch('#'));
    }
    Ok(())
}

fn illegal_format_flags(flags: &str) -> VmError {
    throw(
        "java.util.IllegalFormatFlagsException",
        &format!("Flags = '{flags}'"),
    )
}

/// Apply width padding (spaces; the numeric zero-pad happens earlier).
///
/// The width counts UTF-16 code UNITS, because a JDK's `Formatter` measures
/// with `CharSequence.length()`. A supplementary code point is two of them, so
/// counting code points padded `%5s` of an emoji one space too far.
fn pad(spec: &Spec, body: &str) -> String {
    let width = spec.width.unwrap_or(0);
    let len = body.encode_utf16().count();
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

/// Resolve a reference argument that points at a BOXED wrapper into the
/// corresponding primitive argument, so every conversion arm sees the value.
/// A plain string (or anything else) passes through.
fn unwrap_boxed(heap: &Heap, arg: FormatArg) -> FormatArg {
    let FormatArg::Str(Some(reference)) = arg else {
        return arg;
    };
    let Some(HeapObject::Boxed { class_name, value }) = heap.get(reference) else {
        return arg;
    };
    #[allow(clippy::cast_possible_truncation)]
    match (class_name.as_ref(), value) {
        ("java/lang/Integer", JValue::Int(v)) => FormatArg::Int(*v),
        ("java/lang/Short", JValue::Int(v)) => FormatArg::Short(*v as i16),
        ("java/lang/Byte", JValue::Int(v)) => FormatArg::Byte(*v as i8),
        ("java/lang/Character", JValue::Int(v)) => FormatArg::Char(u16::try_from(*v).unwrap_or(0)),
        ("java/lang/Boolean", JValue::Int(v)) => FormatArg::Boolean(*v != 0),
        ("java/lang/Long", JValue::Long(v)) => FormatArg::Long(*v),
        ("java/lang/Float", JValue::Float(v)) => FormatArg::Float(*v),
        ("java/lang/Double", JValue::Double(v)) => FormatArg::Double(*v),
        _ => arg,
    }
}

/// The JDK's `IllegalFormatConversionException` names the conversion in
/// LOWER case — `%X` is `%x` with the upper flag, and the exception carries the
/// conversion, not the spelling — and names the ARGUMENT's own class, which for
/// a user object is that class and not the `String` the formatter would have
/// made of it.
fn conversion_mismatch(heap: &Heap, conversion: char, arg: FormatArg) -> VmError {
    let conversion = conversion.to_ascii_lowercase();
    throw(
        "java.util.IllegalFormatConversionException",
        &format!("{conversion} != {}", arg.java_class(heap)),
    )
}

#[allow(clippy::too_many_lines)] // one arm per conversion
fn render(heap: &Heap, spec: &Spec, arg: FormatArg) -> Result<String, VmError> {
    let conversion = spec.conversion;
    // A BOXED wrapper argument formats as its value: the compiler passes the
    // reference through (so a null Boolean can reach `%b` as null, not as an
    // NPE at the call site), and the unwrap happens here.
    let arg = unwrap_boxed(heap, arg);
    // The JDK's null rule: for every conversion except %b (false) and %h
    // ("null" — its arm handles it), a null argument renders as the STRING
    // "null", width- and precision-treated like %s, uppercased by an
    // uppercase conversion. This is what lets `%d` and `%x` of null print
    // `null` instead of throwing.
    if matches!(arg, FormatArg::Str(None)) && !matches!(conversion, 'b' | 'B' | 'h' | 'H') {
        let mut text = String::from("null");
        if let Some(precision) = spec.precision {
            text = text.chars().take(precision).collect();
        }
        if conversion.is_ascii_uppercase() {
            text = text.to_uppercase();
        }
        return Ok(pad(spec, &text));
    }
    match conversion.to_ascii_lowercase() {
        's' => {
            let mut text = match arg {
                // A reference that is not a String was rendered ahead of the
                // formatter, since only the interpreter can run a user
                // `toString` — but it is still the OBJECT here, so `%d` in the
                // same template can name its class.
                FormatArg::Str(Some(reference)) => heap
                    .string_text(reference)
                    .or_else(|| heap.rendered_format_text(reference).map(str::to_owned))
                    .unwrap_or_default(),
                FormatArg::Str(None) => String::from("null"),
                FormatArg::Int(v) => v.to_string(),
                FormatArg::Short(v) => v.to_string(),
                FormatArg::Byte(v) => v.to_string(),
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
            if let Some(precision) = spec.precision {
                text = text.chars().take(precision).collect();
            }
            if conversion == 'B' {
                text = text.to_uppercase();
            }
            Ok(pad(spec, &text))
        }
        'h' => {
            let hash = match arg {
                FormatArg::Str(None) => {
                    let text = if conversion == 'H' { "NULL" } else { "null" };
                    return Ok(pad(spec, text));
                }
                FormatArg::Str(Some(reference)) => match heap.get(reference) {
                    Some(HeapObject::JavaString(units)) => {
                        let mut hash: i32 = 0;
                        for unit in units {
                            hash = hash.wrapping_mul(31).wrapping_add(i32::from(*unit));
                        }
                        hash
                    }
                    // A user `hashCode` is user code, so the interpreter took
                    // it ahead of the formatter and recorded it here.
                    _ => heap
                        .rendered_format_hash(reference)
                        .unwrap_or_else(|| reference.cast_signed()),
                },
                FormatArg::Int(v) => v,
                FormatArg::Short(v) => i32::from(v),
                FormatArg::Byte(v) => i32::from(v),
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
            // A hash is TEXT once it is written, so a precision truncates it
            // exactly as it truncates a `%s` — `%.3h` is three hex digits.
            if let Some(precision) = spec.precision {
                text.truncate(text.len().min(precision));
            }
            Ok(pad(spec, &text))
        }
        'c' => {
            let bad_code_point = |v: i32| {
                throw(
                    "java.util.IllegalFormatCodePointException",
                    &format!("Code point = {:#x}", v.cast_unsigned()),
                )
            };
            let unit = match arg {
                FormatArg::Char(u) => u32::from(u),
                FormatArg::Byte(v) => u32::from(v.cast_unsigned()),
                FormatArg::Short(v) => {
                    u32::try_from(v).map_err(|_| bad_code_point(i32::from(v)))?
                }
                FormatArg::Int(v) => u32::try_from(v).map_err(|_| bad_code_point(v))?,
                other => return Err(conversion_mismatch(heap, conversion, other)),
            };
            // JLS: a value past U+10FFFF is not a code point at all — the
            // JDK throws rather than substituting (a lone surrogate is a
            // VALID code point and renders as the replacement char here).
            if unit > 0x10_FFFF {
                return Err(bad_code_point(unit.cast_signed()));
            }
            let mut text = char::from_u32(unit).unwrap_or('\u{FFFD}').to_string();
            if conversion == 'C' {
                text = text.to_uppercase();
            }
            Ok(pad(spec, &text))
        }
        'd' => {
            let value = match arg {
                FormatArg::Int(v) => i64::from(v),
                FormatArg::Short(v) => i64::from(v),
                FormatArg::Byte(v) => i64::from(v),
                FormatArg::Long(v) => v,
                other => return Err(conversion_mismatch(heap, conversion, other)),
            };
            let negative = value < 0;
            let mut magnitude = value.unsigned_abs().to_string();
            if spec.grouping {
                magnitude = group_digits(&magnitude);
            }
            if negative && spec.parentheses {
                // Zero-padding fills INSIDE the parens (`%(08d` of -42 is
                // `(000042)`), which the plain width `pad` would not do.
                let width = spec.width.unwrap_or(0);
                let body_len = 2 + magnitude.chars().count();
                if spec.zero_pad && !spec.left_justify && body_len < width {
                    let zeros = "0".repeat(width - body_len);
                    return Ok(format!("({zeros}{magnitude})"));
                }
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
                FormatArg::Short(v) => u64::from(v.cast_unsigned()),
                FormatArg::Byte(v) => u64::from(v.cast_unsigned()),
                FormatArg::Long(v) => v.cast_unsigned(),
                other => return Err(conversion_mismatch(heap, conversion, other)),
            };
            // The signed flags are refused by the INTEGER printer, which is
            // reached only once the argument turns out to be an integer — so
            // `%(20o` handed a Double is a conversion mismatch, not a flag
            // one.
            let lower = conversion.to_ascii_lowercase();
            for (present, flag) in [(spec.parentheses, '('), (spec.space, ' '), (spec.plus, '+')] {
                if present {
                    return Err(throw(
                        "java.util.FormatFlagsConversionMismatchException",
                        &format!("Conversion = {lower}, Flags = {flag}"),
                    ));
                }
            }
            let mut text = match conversion.to_ascii_lowercase() {
                'o' => format!("{value:o}"),
                _ => format!("{value:x}"),
            };
            if conversion == 'X' {
                text = text.to_uppercase();
            }
            // The `#` radix prefix (`0x`/`0X`/`0`) sits before any zero-pad,
            // like a sign — `%#010x` of 255 is `0x000000ff`, not `0000000xff`.
            let prefix = if spec.alternate {
                match conversion {
                    'o' | 'O' => "0",
                    'X' => "0X",
                    _ => "0x",
                }
            } else {
                ""
            };
            Ok(pad_numeric(spec, prefix, &text))
        }
        'f' | 'e' | 'g' => {
            let value = match arg {
                FormatArg::Double(v) => v,
                // Java's Formatter widens Float via doubleValue().
                FormatArg::Float(v) => f64::from(v),
                other => return Err(conversion_mismatch(heap, conversion, other)),
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
        // `%t`/`%T` — a date-time conversion. The only argument types a JDK
        // accepts are `long`/`Long`, `Date`, `Calendar` and `TemporalAccessor`,
        // and caturra models none of the three classes — so every WRONG type
        // is reported exactly as a JDK reports it, and the one right type is
        // an honest refusal rather than a wrong date. (A calendar reading also
        // depends on the default time zone, which is the browser's.)
        't' | 'T' => {
            let suffix = spec.date_time.unwrap_or(conversion);
            match arg {
                FormatArg::Long(_) => Err(VmError::UnknownIntrinsic(format!(
                    "a date-time conversion (%{conversion}{suffix}) over a long —                      caturra has no Date, Calendar or java.time"
                ))),
                other => Err(throw(
                    "java.util.IllegalFormatConversionException",
                    &format!("{suffix} != {}", other.java_class(heap)),
                )),
            }
        }
        // `%a`/`%A` — hexadecimal floating-point. The MAGNITUDE is rendered
        // (rounded to the precision, if any), and the sign is put back here,
        // which is what makes `%+a` and `% a` work and what puts a zero-pad
        // between the `0x` and the digits rather than in front of them.
        'a' => {
            let value = match arg {
                FormatArg::Double(v) => v,
                FormatArg::Float(v) => f64::from(v),
                other => return Err(conversion_mismatch(heap, conversion, other)),
            };
            // `-0.0` is negative here, as `Double.compare(v, 0.0) == -1` says.
            let negative = value.is_sign_negative() && !value.is_nan();
            let magnitude = if negative { -value } else { value };
            if !magnitude.is_finite() {
                let body = java_non_finite(magnitude);
                let body = if conversion == 'A' {
                    body.to_uppercase()
                } else {
                    body.to_owned()
                };
                // NaN is neither positive nor negative, so `%+a` leaves it
                // alone where it signs an Infinity — and a negative infinity
                // keeps its own sign, which the magnitude no longer carries.
                let sign = if magnitude.is_nan() {
                    ""
                } else if negative {
                    "-"
                } else if spec.plus {
                    "+"
                } else if spec.space {
                    " "
                } else {
                    ""
                };
                return Ok(pad(spec, &format!("{sign}{body}")));
            }
            let (digits, padded) = hex_significand(magnitude, spec.precision);
            let upper = conversion == 'A';
            let prefix = if upper { "0X" } else { "0x" };
            let (digits, padded) = if upper {
                (digits.to_uppercase(), padded.to_uppercase())
            } else {
                (digits, padded)
            };
            Ok(pad_hex_float(spec, negative, prefix, &digits, &padded))
        }
        _ => Err(unknown_conversion(conversion)),
    }
}

/// `NaN` / `Infinity`, the text a JDK's `Formatter` writes for one.
fn java_non_finite(value: f64) -> &'static str {
    if value.is_nan() { "NaN" } else { "Infinity" }
}

/// The digits of `%a` for a non-negative finite `value`: everything after the
/// `0x`, rounded to `precision` hexadecimal fraction digits.
///
/// This is `Formatter.hexDouble`, and it rounds in BINARY rather than on the
/// text — `%.2a` of `1e23` is `0x1.53p76`, which no truncation of
/// `0x1.52d02c7e14af6p76` gives. Without a precision (and with one of 13 or
/// more) the shortest form `Double.toHexString` answers is what a JDK prints,
/// padded out with trailing zeros; with one of 1..=12 the significand is
/// rounded half-even to `1 + 4 * precision` bits first, and a subnormal is
/// scaled up by 2^54 so it can be, then written with the exponent it had.
fn hex_significand(value: f64, precision: Option<usize>) -> (String, String) {
    // A JDK maps "no precision" and an explicit `.0` to the same two cases:
    // the shortest form, and one digit.
    let precision = match precision {
        None => 0,
        Some(0) => 1,
        Some(other) => other,
    };
    let text = if value == 0.0 || precision == 0 || precision >= 13 {
        strip_hex_prefix(&crate::intrinsics::java_double_to_hex(value))
    } else {
        rounded_hex(value, precision)
    };
    // Both forms: a JDK measures the width against the UNPADDED text and then
    // pads the fraction, so `%012.3a` of `0.0` answers FOURTEEN characters —
    // the five zeros a five-character `0.0p0` needed, plus the two the
    // precision then added. Measuring the padded text would have been tidier
    // and is not what a JDK prints.
    let padded = pad_hex_fraction(&text, precision);
    (text, padded)
}

/// Round `value`'s significand to `1 + 4 * precision` bits, half-even, and
/// render what comes out. `precision` is 1..=12 here.
fn rounded_hex(value: f64, precision: usize) -> String {
    // Scale a subnormal into the normal range so it HAS that many significand
    // bits to round; its exponent comes back at the end.
    let subnormal = value < f64::MIN_POSITIVE;
    let scaled = if subnormal {
        value * 2f64.powi(54)
    } else {
        value
    };
    let shift = 53 - (1 + 4 * precision);
    let bits = scaled.to_bits();
    let mut significand = (bits & 0x7fff_ffff_ffff_ffff) >> shift;
    let dropped = bits & !(!0u64 << shift);
    let least_zero = significand & 1 == 0;
    let round = (1u64 << (shift - 1)) & dropped != 0;
    let sticky = shift > 1 && !(1u64 << (shift - 1)) & dropped != 0;
    // Half-even: round up on a set round bit unless the result would be
    // exactly halfway with an even last digit. (Written as the JDK writes it;
    // clippy would rather see `round && (!least_zero || sticky)`.)
    if round && (!least_zero || sticky) {
        significand += 1;
    }
    let rounded = f64::from_bits(significand << shift);
    if rounded.is_infinite() {
        // Rounding `Double.MAX_VALUE` up leaves the range: a JDK writes the
        // exponent that would follow rather than `Infinity`.
        return String::from("1.0p1024");
    }
    let text = strip_hex_prefix(&crate::intrinsics::java_double_to_hex(rounded));
    if !subnormal {
        return text;
    }
    // The scaling has to come back out of the exponent, not the digits.
    match text.split_once('p') {
        Some((digits, exponent)) => {
            let exponent: i32 = exponent.parse().unwrap_or(0);
            format!("{digits}p{}", exponent - 54)
        }
        None => text,
    }
}

fn strip_hex_prefix(text: &str) -> String {
    text.strip_prefix("0x").unwrap_or(text).to_owned()
}

/// Trailing zeros out to `precision` fraction digits — the shortest form a
/// `Double.toHexString` gives is often shorter than what was asked for.
fn pad_hex_fraction(text: &str, precision: usize) -> String {
    if precision == 0 {
        return text.to_owned();
    }
    let Some((significand, exponent)) = text.split_once('p') else {
        return text.to_owned();
    };
    let fraction = significand.split_once('.').map_or(0, |(_, f)| f.len());
    if fraction >= precision {
        return text.to_owned();
    }
    let zeros = "0".repeat(precision - fraction);
    format!("{significand}{zeros}p{exponent}")
}

/// Width for `%a`: the sign (or `+`/` `) leads, then `0x`, and a zero-pad goes
/// BETWEEN that prefix and the digits — `%012a` of `0.0` is `0x000000.0p0`,
/// where padding the whole body would have written `000000x0.0p0`.
fn pad_hex_float(spec: &Spec, negative: bool, prefix: &str, digits: &str, padded: &str) -> String {
    let sign = if negative {
        "-"
    } else if spec.plus {
        "+"
    } else if spec.space {
        " "
    } else {
        ""
    };
    if spec.zero_pad && !spec.left_justify {
        // Neither the SIGN nor the fraction's own trailing zeros count toward
        // the width: a JDK subtracts the `0x` and the UNPADDED digits and
        // nothing else, so `%012.3a` of a negative answers thirteen
        // characters rather than twelve.
        let width = spec.width.unwrap_or(0);
        let measured = digits.chars().count() + prefix.len();
        if measured < width {
            let zeros = "0".repeat(width - measured);
            return format!("{sign}{prefix}{zeros}{padded}");
        }
    }
    pad(spec, &format!("{sign}{prefix}{padded}"))
}

/// Width handling for floats: the body already contains its sign, so
/// zero-padding must go after it.
fn pad_sign_aware(spec: &Spec, _negative: bool, body: &str) -> String {
    // The JDK ignores zero-padding for non-finite values: `%010f` of
    // Infinity is `  Infinity`, not `00Infinity`.
    let non_finite = {
        let core = body.to_ascii_uppercase();
        core.ends_with("NAN") || core.contains("INFINITY")
    };
    if spec.zero_pad && !spec.left_justify && !non_finite {
        let width = spec.width.unwrap_or(0);
        let len = body.chars().count();
        if len < width {
            let zeros = "0".repeat(width - len);
            // A parenthesized negative (`%(08.2f` of -3.5) fills INSIDE the
            // parens: `(003.50)`, not `00(3.50)`.
            if let Some(inner) = body.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                return format!("({zeros}{inner})");
            }
            let (sign, magnitude) = if let Some(rest) = body.strip_prefix('-') {
                ("-", rest)
            } else if let Some(rest) = body.strip_prefix('+') {
                ("+", rest)
            } else {
                ("", body)
            };
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
        // The `(` flag parenthesizes a NEGATIVE infinity — `%(f` of
        // -Infinity is `(Infinity)`, not `-Infinity`.
        if value < 0.0 && spec.parentheses {
            return String::from("(Infinity)");
        }
        let sign = if value < 0.0 {
            "-"
        } else if spec.plus {
            "+"
        } else if spec.space {
            " "
        } else {
            ""
        };
        return format!("{sign}Infinity");
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
            // The `#` flag forces a decimal point even at precision 0:
            // `%#.0f` of 3.0 is `3.`, where `%.0f` is `3`.
            if spec.alternate && precision == 0 {
                text.push('.');
            }
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
            let mut text = general_digits(value.abs(), precision);
            // `%,g` groups the integer part when the result is in fixed
            // notation (a scientific result has nothing to group).
            if spec.grouping && !text.contains(['e', 'E']) {
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
    // The digits `Double.toString` would print, NOT the shortest round-trip
    // ones. OpenJDK 11 derives `%f`/`%e`/`%g` from the same `FloatingDecimal`
    // digits as `toString`, so `%f` of 1e23 is `99999999999999990000000.000000`
    // — reading Rust's shortest form here printed a different NUMBER, padded
    // with the wrong zeros.
    let text = crate::floatdec::java_double_to_string(value.abs());
    let (mantissa, exponent) = match text.split_once('E') {
        Some((mantissa, exponent)) => {
            (mantissa, exponent.parse::<i32>().expect("numeric exponent"))
        }
        None => (text.as_str(), 0),
    };
    let integral = mantissa.split('.').next().unwrap_or("");
    let mut digits: Vec<u8> = mantissa
        .bytes()
        .filter(u8::is_ascii_digit)
        .map(|b| b - b'0')
        .collect();
    // `0.digits × 10^point`, so the point starts after the integral part.
    let mut point = i32::try_from(integral.len()).expect("short") + exponent;
    while digits.first() == Some(&0) {
        digits.remove(0);
        point -= 1;
    }
    while digits.last() == Some(&0) && digits.len() > 1 {
        digits.pop();
    }
    (digits, point)
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
        // Zero in %g is fixed notation with `precision - 1` fraction digits:
        // `%.1g` of 0.0 is `0`, `%.6g` is `0.00000`. It was emitting `0.0`
        // regardless of precision.
        let fraction = precision.saturating_sub(1);
        if fraction == 0 {
            return String::from("0");
        }
        return format!("0.{}", "0".repeat(fraction));
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
    heap: &Heap,
    descriptor: &str,
    values: &[JValue],
) -> Result<FormatArgs, VmError> {
    let inner = descriptor
        .strip_prefix("(Ljava/lang/String;")
        .and_then(|rest| rest.split_once(')'))
        .map(|(args, _)| args)
        .unwrap_or_default();
    let mut tags = Vec::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            'I' | 'D' | 'C' | 'Z' | 'J' | 'F' | 'S' | 'B' => tags.push(c),
            'L' => {
                for inner_char in chars.by_ref() {
                    if inner_char == ';' {
                        break;
                    }
                }
                tags.push('L');
            }
            // A FORWARDED varargs array — `printf(fmt, parts)` inside a
            // `void log(String fmt, Object... parts)`. Its ELEMENTS are the
            // format arguments, so this one tag stands for all of them.
            '[' => {
                for inner_char in chars.by_ref() {
                    if inner_char == ';' {
                        break;
                    }
                }
                tags.push('[');
            }
            _ => {}
        }
    }
    let mut args = Vec::with_capacity(tags.len());
    let mut all_null = false;
    for (tag, value) in tags.iter().zip(values) {
        if *tag == '[' {
            let elements = if let JValue::Ref(Some(array)) = value {
                match heap.get(*array) {
                    Some(HeapObject::RefArray(_, elements)) => elements.clone(),
                    _ => {
                        return Err(VmError::UncaughtException(String::from(
                            "java.lang.VerifyError: malformed format call",
                        )));
                    }
                }
            } else {
                // `format(fmt, (Object[]) null)`: EVERY specifier reads null,
                // whatever its index.
                all_null = true;
                Vec::new()
            };
            for element in elements {
                args.push(match element {
                    JValue::Ref(reference) => FormatArg::Str(reference),
                    JValue::Int(v) => FormatArg::Int(v),
                    JValue::Long(v) => FormatArg::Long(v),
                    JValue::Double(v) => FormatArg::Double(v),
                    JValue::Float(v) => FormatArg::Float(v),
                });
            }
            continue;
        }
        args.push(match (tag, value) {
            ('I', JValue::Int(v)) => FormatArg::Int(*v),
            ('C', JValue::Int(v)) => FormatArg::Char(u16::try_from(*v).unwrap_or(0)),
            ('Z', JValue::Int(v)) => FormatArg::Boolean(*v != 0),
            ('D', JValue::Double(v)) => FormatArg::Double(*v),
            ('J', JValue::Long(v)) => FormatArg::Long(*v),
            ('F', JValue::Float(v)) => FormatArg::Float(*v),
            ('S', JValue::Int(v)) =>
            {
                #[allow(clippy::cast_possible_truncation)]
                FormatArg::Short(*v as i16)
            }
            ('B', JValue::Int(v)) =>
            {
                #[allow(clippy::cast_possible_truncation)]
                FormatArg::Byte(*v as i8)
            }
            ('L', JValue::Ref(reference)) => FormatArg::Str(*reference),
            _ => {
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.VerifyError: malformed format call",
                )));
            }
        });
    }
    Ok(FormatArgs {
        values: args,
        all_null,
    })
}
