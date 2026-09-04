//! `java.text.DecimalFormat` and `java.text.NumberFormat` — a number rendered
//! for a person to read.
//!
//! The pattern is a small language (`#,##0.00`, `0.0%`, `#.##;(#.##)`), and
//! what it means is a handful of counts: how few integer digits to insist on,
//! how many fraction digits to allow, where the groups fall, what multiplies
//! the value, and what sits either side of it. Formatting is then exact
//! decimal arithmetic over [`crate::decimal`], never a second pass through a
//! `double`.
//!
//! Two things here are not guessable from the javadoc and were taken from a
//! JDK instead. In SCIENTIFIC notation the significant-digit count is
//! `maximumInteger + maximumFraction` and the fraction limit does not cap what
//! is printed. And a tie is broken by comparing the SHORTEST decimal for the
//! double against the double's exact value: `0.05` at one place is `0.1`
//! (the shortest form rounded down, so the true value is above the tie) while
//! `1.005` at two places is `1.00` (it rounded up, so the true value is below).

use crate::decimal::{BigDec, Rounding};

/// A parsed pattern: everything `format` needs, and nothing about the value.
// The four affixes each carry two flags, which is what the count is: they
// belong beside the text they describe, not in a struct of their own.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberPattern {
    /// Whether each affix has to be QUOTED when written back — true only when
    /// the pattern quoted a special character, which is the one thing the
    /// expanded text can no longer tell you.
    pub positive_prefix_quoted: bool,
    pub positive_suffix_quoted: bool,
    pub negative_prefix_quoted: bool,
    pub negative_suffix_quoted: bool,
    /// ...and whether it held the `¤¤` that expands to a currency CODE, which
    /// is the one expansion `toPattern` has to undo.
    pub positive_prefix_code: bool,
    pub positive_suffix_code: bool,
    pub negative_prefix_code: bool,
    pub negative_suffix_code: bool,
    /// The affix as it is printed, with the quoting resolved.
    pub positive_prefix: String,
    pub positive_suffix: String,
    pub negative_prefix: String,
    pub negative_suffix: String,
    pub minimum_integer_digits: u32,
    pub maximum_integer_digits: u32,
    pub minimum_fraction_digits: u32,
    pub maximum_fraction_digits: u32,
    pub grouping_used: bool,
    pub grouping_size: u32,
    /// 100 for a percent pattern, 1000 for a per-mille one.
    pub multiplier: i64,
    /// `0.###E0` — the minimum width of the exponent that follows.
    pub exponent_digits: Option<u32>,
    pub rounding: Rounding,
}

impl Default for NumberPattern {
    fn default() -> Self {
        // A bare `new DecimalFormat()` is `#,##0.###`.
        Self {
            positive_prefix_quoted: false,
            positive_suffix_quoted: false,
            negative_prefix_quoted: false,
            negative_suffix_quoted: false,
            positive_prefix_code: false,
            positive_suffix_code: false,
            negative_prefix_code: false,
            negative_suffix_code: false,
            positive_prefix: String::new(),
            positive_suffix: String::new(),
            negative_prefix: String::from("-"),
            negative_suffix: String::new(),
            minimum_integer_digits: 1,
            maximum_integer_digits: i32::MAX as u32,
            minimum_fraction_digits: 0,
            maximum_fraction_digits: 3,
            grouping_used: true,
            grouping_size: 3,
            multiplier: 1,
            exponent_digits: None,
            rounding: Rounding::HalfEven,
        }
    }
}

/// The currency sign for a locale with no country — which is caturra's, since
/// a sandbox has no way to know one. A JDK with `LANG=en` answers the same.
pub const CURRENCY_SIGN: char = '\u{00a4}';

impl NumberPattern {
    /// Parse a `DecimalFormat` pattern.
    ///
    /// # Errors
    /// The JDK's own complaint, for the patterns it refuses.
    pub fn parse(pattern: &str) -> Result<Self, String> {
        // An EMPTY pattern leaves the locale's own in place, which is what a
        // JDK does too — its `toPattern` then asks for an array of 2^31 digits
        // and dies, and that part is not worth reproducing.
        if pattern.is_empty() {
            return Ok(Self::default());
        }
        let halves = split_subpatterns(pattern)?;
        let mut parsed = Self::from_subpattern(halves[0])?;
        // An EMPTY negative half is simply the derived one.
        if let Some(negative) = halves.get(1).filter(|half| !half.is_empty()) {
            let other = Self::from_subpattern(negative)?;
            parsed.negative_prefix = other.positive_prefix;
            parsed.negative_suffix = other.positive_suffix;
            parsed.negative_prefix_quoted = other.positive_prefix_quoted;
            parsed.negative_suffix_quoted = other.positive_suffix_quoted;
            parsed.negative_prefix_code = other.positive_prefix_code;
            parsed.negative_suffix_code = other.positive_suffix_code;
        } else {
            parsed.negative_prefix = format!("-{}", parsed.positive_prefix);
            parsed.negative_suffix = parsed.positive_suffix.clone();
            parsed.negative_prefix_quoted = parsed.positive_prefix_quoted;
            parsed.negative_suffix_quoted = parsed.positive_suffix_quoted;
            parsed.negative_prefix_code = parsed.positive_prefix_code;
            parsed.negative_suffix_code = parsed.positive_suffix_code;
        }
        Ok(parsed)
    }

    /// One half of a pattern, counted the way a JDK counts it: the `#`s BEFORE
    /// the first `0` (across the decimal point), the `0`s, the `#`s after, and
    /// where the point fell among them. Every limit comes from those four
    /// numbers, which is why `#.##` insists on one integer digit and `#` does
    /// not.
    #[allow(clippy::too_many_lines)]
    fn from_subpattern(text: &str) -> Result<Self, String> {
        let mut out = Self::default();
        let chars: Vec<char> = text.chars().collect();
        let mut at = 0;
        let (expanded, quoted, code) = take_affix(&chars, &mut at, &mut out.multiplier, text)?;
        out.positive_prefix = expanded;
        out.positive_prefix_quoted = quoted;
        out.positive_prefix_code = code;
        let mut left = 0u32;
        let mut zeros = 0u32;
        let mut right = 0u32;
        let mut point: Option<u32> = None;
        let mut since_group: Option<u32> = None;
        let mut exponent = None;
        while at < chars.len() {
            match chars[at] {
                '#' | '0' => {
                    if chars[at] == '0' {
                        zeros += 1;
                    } else if zeros > 0 {
                        right += 1;
                    } else {
                        left += 1;
                    }
                    // The grouping is an INTEGER-side count: it stops at the
                    // point rather than running on into the fraction.
                    if let Some(count) = since_group.as_mut().filter(|_| point.is_none()) {
                        *count += 1;
                    }
                    at += 1;
                }
                '.' => {
                    if point.is_some() {
                        return Err(format!("Multiple decimal separators in pattern \"{text}\""));
                    }
                    point = Some(left + zeros + right);
                    at += 1;
                }
                ',' => {
                    // A grouping separator belongs on the INTEGER side, and
                    // never at the very end.
                    if point.is_some() {
                        return Err(format!("Malformed pattern \"{text}\""));
                    }
                    since_group = Some(0);
                    at += 1;
                    if at >= chars.len() || !matches!(chars[at], '#' | '0' | ',') {
                        return Err(format!("Malformed pattern \"{text}\""));
                    }
                }
                'E' => {
                    at += 1;
                    let mut width = 0;
                    while at < chars.len() && chars[at] == '0' {
                        width += 1;
                        at += 1;
                    }
                    if width == 0 {
                        return Err(format!("Malformed exponential pattern \"{text}\""));
                    }
                    exponent = Some(width);
                }
                // Anything else is an AFFIX — and the digits may resume after
                // it, which is how `#'%'0` keeps both of its digit positions
                // and puts the literal in the suffix.
                _ => {
                    let (expanded, quoted, code) =
                        take_affix(&chars, &mut at, &mut out.multiplier, text)?;
                    out.positive_suffix.push_str(&expanded);
                    out.positive_suffix_quoted |= quoted;
                    out.positive_suffix_code |= code;
                }
            }
        }
        // A pattern with a point and no `0` behaves as if it had one: `#.##`
        // shows `0.5`, not `.5`.
        if let Some(at) = point.filter(|_| zeros == 0 && left > 0) {
            let n = at.max(1);
            right = left - n;
            left = n - 1;
            zeros = 1;
        }
        let total = left + zeros + right;
        let effective_point = point.unwrap_or(total);
        out.minimum_integer_digits = effective_point.saturating_sub(left);
        out.maximum_integer_digits = i32::MAX as u32;
        out.maximum_fraction_digits = point.map_or(0, |_| total - effective_point);
        out.minimum_fraction_digits =
            point.map_or(0, |_| (left + zeros).saturating_sub(effective_point));
        out.grouping_used = false;
        out.grouping_size = 0;
        if let Some(size) = since_group.filter(|size| *size > 0) {
            out.grouping_used = true;
            out.grouping_size = size;
        }
        if let Some(width) = exponent {
            // In scientific notation the integer digits ARE the significand's.
            out.maximum_integer_digits = left + out.minimum_integer_digits;
            out.exponent_digits = Some(width);
        }
        Ok(out)
    }

    /// The pattern a JDK's `toPattern` writes back — which is NOT always the
    /// one handed in: `0` comes back as `#0`, because the width one above the
    /// minimum is written out too. The affixes are echoed as they were WRITTEN.
    #[must_use]
    pub fn to_pattern(&self) -> String {
        let mut out = write_affix(
            &self.positive_prefix,
            self.positive_prefix_quoted,
            self.positive_prefix_code,
        );
        out.push_str(&self.digit_pattern());
        out.push_str(&write_affix(
            &self.positive_suffix,
            self.positive_suffix_quoted,
            self.positive_suffix_code,
        ));
        // The negative half is written only when it is not the derived one.
        if self.negative_prefix != format!("-{}", self.positive_prefix)
            || self.negative_suffix != self.positive_suffix
        {
            out.push(';');
            out.push_str(&write_affix(
                &self.negative_prefix,
                self.negative_prefix_quoted,
                self.negative_prefix_code,
            ));
            out.push_str(&self.digit_pattern());
            out.push_str(&write_affix(
                &self.negative_suffix,
                self.negative_suffix_quoted,
                self.negative_suffix_code,
            ));
        }
        out
    }

    fn digit_pattern(&self) -> String {
        let mut out = String::new();
        let width = if self.exponent_digits.is_some() {
            self.maximum_integer_digits
        } else {
            self.minimum_integer_digits.max(if self.grouping_used {
                self.grouping_size
            } else {
                0
            }) + 1
        };
        for at in 0..width {
            let from_right = width - at;
            if self.grouping_used
                && self.grouping_size > 0
                && at > 0
                && from_right % self.grouping_size == 0
            {
                out.push(',');
            }
            out.push(if from_right <= self.minimum_integer_digits {
                '0'
            } else {
                '#'
            });
        }
        if self.maximum_fraction_digits > 0 {
            out.push('.');
            for at in 0..self.maximum_fraction_digits {
                out.push(if at < self.minimum_fraction_digits {
                    '0'
                } else {
                    '#'
                });
            }
        }
        if let Some(width) = self.exponent_digits {
            out.push('E');
            for _ in 0..width {
                out.push('0');
            }
        }
        out
    }

    /// Render `value` (already a decimal, already positive) with `negative`
    /// saying which affixes to wear. `exact` is the value the source really
    /// held, when it came from a `double` — it is what breaks a tie.
    #[must_use]
    pub fn format(&self, value: &BigDec, negative: bool, exact: Option<&BigDec>) -> String {
        let scaled = if self.multiplier == 1 {
            value.clone()
        } else {
            value.multiply(&BigDec::from_i64(self.multiplier))
        };
        self.format_scaled(&scaled, negative, exact)
    }

    /// The same, for a value the caller has ALREADY multiplied — which a
    /// `double` must be, in `double` arithmetic, because that is where a JDK
    /// applies the percent factor and the product's own rounding is visible
    /// (`0%` of 2.675 is `268%`, not `267%`).
    #[must_use]
    pub fn format_scaled(&self, scaled: &BigDec, negative: bool, exact: Option<&BigDec>) -> String {
        let body = if self.exponent_digits.is_some() {
            self.scientific_body(scaled, exact)
        } else {
            self.plain_body(scaled, exact)
        };
        let (prefix, suffix) = if negative {
            (&self.negative_prefix, &self.negative_suffix)
        } else {
            (&self.positive_prefix, &self.positive_suffix)
        };
        format!("{prefix}{body}{suffix}")
    }

    fn plain_body(&self, value: &BigDec, exact: Option<&BigDec>) -> String {
        let places = i32::try_from(self.maximum_fraction_digits).unwrap_or(0);
        let rounded = round_like_a_jdk(value, exact, places, self.rounding);
        let text = rounded.to_plain_text();
        let text = text.strip_prefix('-').unwrap_or(&text).to_owned();
        let (integral, fraction) = text
            .split_once('.')
            .map_or((text.as_str(), ""), |(i, f)| (i, f));
        // How many integer digits the VALUE has is not how many the pattern
        // shows: the count is the larger of the two, capped by the maximum.
        let digits = integral.trim_start_matches('0');
        let have = u32::try_from(digits.len()).unwrap_or(0);
        let count = have
            .max(self.minimum_integer_digits)
            .min(self.maximum_integer_digits);
        let mut integral = String::new();
        for _ in have..count {
            integral.push('0');
        }
        let kept = usize::try_from(count.min(have)).unwrap_or(0);
        integral.push_str(&digits[digits.len() - kept..]);
        let mut fraction = fraction.trim_end_matches('0').to_owned();
        while (fraction.len() as u32) < self.minimum_fraction_digits {
            fraction.push('0');
        }
        // Nothing at all on either side prints a single zero — which is how
        // `#` renders 0.5 as `0` rather than as an empty string.
        if integral.is_empty() && fraction.is_empty() {
            integral.push('0');
        }
        let mut out = if self.grouping_used && self.grouping_size > 0 {
            group(&integral, self.grouping_size as usize)
        } else {
            integral
        };
        if !fraction.is_empty() {
            out.push('.');
            out.push_str(&fraction);
        }
        out
    }

    /// The scientific form. Three rules here are a JDK's and not the
    /// javadoc's: the SIGNIFICANT digits are `maximumInteger +
    /// maximumFraction`; the total printed is at least `minimumInteger +
    /// minimumFraction` and the fraction minimum does not pad on its own; and
    /// a zero has no exponent at all.
    fn scientific_body(&self, value: &BigDec, exact: Option<&BigDec>) -> String {
        let width = self.exponent_digits.unwrap_or(1);
        let significant = self.maximum_integer_digits + self.maximum_fraction_digits;
        let repeat = self.maximum_integer_digits;
        let least = self.minimum_integer_digits + self.minimum_fraction_digits;
        let mut minimum_integer = self.minimum_integer_digits;
        let repeating = repeat > 1 && repeat > self.minimum_integer_digits;
        if repeating {
            minimum_integer = 1;
        }
        let (digits, integer_digits, exponent) = if value.signum() == 0 {
            (String::new(), minimum_integer, 0i64)
        } else {
            let rounded = round_significant(value, exact, significant.max(1), self.rounding);
            let decimal_at = i64::from(rounded.precision()) - i64::from(rounded.scale());
            let exponent = if repeating {
                let repeat = i64::from(repeat);
                if decimal_at >= 1 {
                    (decimal_at - 1) / repeat * repeat
                } else {
                    // Java's integer division truncates toward zero, which is
                    // what puts a small value's exponent on the step below it.
                    (decimal_at - repeat) / repeat * repeat
                }
            } else {
                decimal_at - i64::from(minimum_integer)
            };
            let digits = rounded.unscaled().abs().to_text(10);
            let digits = digits.trim_end_matches('0').to_owned();
            let integer_digits = u32::try_from(decimal_at - exponent).unwrap_or(0);
            (digits, integer_digits, exponent)
        };
        // Enough digits for the minimum the pattern asks, for what the value
        // has, and for the integer places the exponent left in front.
        let total = least
            .max(u32::try_from(digits.len()).unwrap_or(0))
            .max(integer_digits);
        let mut all = digits;
        while (all.len() as u32) < total {
            all.push('0');
        }
        let split = usize::try_from(integer_digits).unwrap_or(0);
        // An empty integer part stays empty here — `#E0` of 4723 is `.5E4`.
        let out: String = all.chars().take(split).collect();
        let mut out = out;
        let fraction: String = all.chars().skip(split).collect();
        if !fraction.is_empty() {
            out.push('.');
            out.push_str(&fraction);
        }
        out.push('E');
        if exponent < 0 {
            out.push('-');
        }
        let text = exponent.abs().to_string();
        for _ in text.len()..width as usize {
            out.push('0');
        }
        out.push_str(&text);
        out
    }

    /// `parse(text)` — the leading number, as a JDK reads it. `None` when
    /// nothing there is a number at all.
    /// What a percent or per-mille pattern multiplies by.
    #[must_use]
    pub fn multiplier(&self) -> i64 {
        self.multiplier
    }

    #[must_use]
    pub fn parse_number(&self, text: &str) -> Option<BigDec> {
        let mut rest = text;
        let mut negative = false;
        if !self.negative_prefix.is_empty() && rest.starts_with(&self.negative_prefix) {
            negative = true;
            rest = &rest[self.negative_prefix.len()..];
        } else if !self.positive_prefix.is_empty() && rest.starts_with(&self.positive_prefix) {
            rest = &rest[self.positive_prefix.len()..];
        }
        let mut digits = String::new();
        let mut seen_point = false;
        for ch in rest.chars() {
            if ch.is_ascii_digit() {
                digits.push(ch);
            } else if ch == '.' && !seen_point {
                seen_point = true;
                digits.push('.');
            } else if ch == ',' {
                // A grouping separator inside the number is simply skipped.
            } else {
                break;
            }
        }
        let digits = digits.trim_end_matches('.');
        if digits.is_empty() {
            return None;
        }
        let value = BigDec::parse(digits).ok()?;
        let value = if self.multiplier == 1 {
            value
        } else {
            value
                .divide_to_scale(
                    &BigDec::from_i64(self.multiplier),
                    value.scale() + 3,
                    Rounding::HalfEven,
                )
                .unwrap_or(value)
        };
        Some(if negative { value.negated() } else { value })
    }
}

/// Round to `places` decimals. When the value came from a `double`, a TIE is
/// broken by which side of the shortest decimal the true value fell — which is
/// how `0.05` at one place is `0.1` while `1.005` at two places is `1.00`.
fn round_like_a_jdk(value: &BigDec, exact: Option<&BigDec>, places: i32, mode: Rounding) -> BigDec {
    let settled = tie_mode(value, exact, places, mode);
    value
        .with_scale(places, settled)
        .unwrap_or_else(|_| value.clone())
}

/// The same, to a count of SIGNIFICANT digits rather than decimal places.
fn round_significant(
    value: &BigDec,
    exact: Option<&BigDec>,
    digits: u32,
    mode: Rounding,
) -> BigDec {
    if value.precision() <= digits {
        return value.clone();
    }
    let places = i32::try_from(
        i64::from(value.scale()) - (i64::from(value.precision()) - i64::from(digits)),
    )
    .unwrap_or(0);
    round_like_a_jdk(value, exact, places, mode)
}

/// Which rounding actually applies at `places`. Away from a tie every HALF_*
/// mode agrees, so this only ever changes the tie.
fn tie_mode(value: &BigDec, exact: Option<&BigDec>, places: i32, mode: Rounding) -> Rounding {
    if !matches!(
        mode,
        Rounding::HalfUp | Rounding::HalfDown | Rounding::HalfEven
    ) {
        return mode;
    }
    let Some(exact) = exact else {
        return mode;
    };
    let Ok(truncated) = value.with_scale(places, Rounding::Down) else {
        return mode;
    };
    // A tie is a remainder of exactly half a unit in the last kept place.
    let half = BigDec::new(crate::bigint::BigInt::from_i64(5), places + 1);
    if value.subtract(&truncated).compare(&half) != core::cmp::Ordering::Equal {
        return mode;
    }
    // An INTEGRAL double never takes the half-even branch in a JDK — its
    // shortest form carries the `.0` a `Double.toString` writes, and that
    // trailing zero puts the rounding position one short of the last digit.
    if value.scale() <= 0 || value.stripped().scale() <= 0 {
        return Rounding::Up;
    }
    // A tie that UNDERFLOWS to zero rounds down when the shortest form is
    // scientific — which `Double.toString` makes it below 1e-3. `0.005` at two
    // places is `0.01`; `0.0005` at three is `0.000`, and the only thing that
    // separates them is which shape the text took.
    let adjusted = i64::from(value.precision()) - 1 - i64::from(value.scale());
    if truncated.signum() == 0 && !(-3..7).contains(&adjusted) {
        return Rounding::Down;
    }
    match value.compare(exact) {
        // The shortest decimal rounded UP off the true value, so the true
        // value is below the tie and this rounds down.
        core::cmp::Ordering::Greater => Rounding::Down,
        core::cmp::Ordering::Less => Rounding::Up,
        core::cmp::Ordering::Equal => mode,
    }
}

/// Split at an UNQUOTED `;`. There may be one, and the half before it may not
/// be empty — a JDK calls anything else an unquoted special character.
///
/// # Errors
/// The JDK's own complaint.
fn split_subpatterns(pattern: &str) -> Result<Vec<&str>, String> {
    let mut quoted = false;
    let mut cut = None;
    for (at, ch) in pattern.char_indices() {
        match ch {
            '\'' => quoted = !quoted,
            ';' if !quoted => {
                if cut.is_some() || at == 0 {
                    return Err(format!(
                        "Unquoted special character ';' in pattern \"{pattern}\""
                    ));
                }
                cut = Some(at);
            }
            _ => {}
        }
    }
    Ok(match cut {
        Some(at) => vec![&pattern[..at], &pattern[at + 1..]],
        None => vec![pattern],
    })
}

/// A prefix or suffix: everything until a digit character. Answers the SOURCE
/// text (quotes and all, for `toPattern`) and the expanded one.
fn take_affix(
    chars: &[char],
    at: &mut usize,
    multiplier: &mut i64,
    whole: &str,
) -> Result<(String, bool, bool), String> {
    let mut out = String::new();
    let mut quoted = false;
    let mut code = false;
    while *at < chars.len() {
        match chars[*at] {
            '0' | '#' | '.' | ',' => break,
            '\'' => {
                *at += 1;
                // `''` is one literal quote; anything else runs to the closing
                // one, and a run that never closes is a malformed pattern.
                if *at < chars.len() && chars[*at] == '\'' {
                    out.push('\'');
                    *at += 1;
                    continue;
                }
                let mut closed = false;
                while *at < chars.len() {
                    if chars[*at] == '\'' {
                        closed = true;
                        *at += 1;
                        break;
                    }
                    if SPECIAL.contains(chars[*at]) {
                        quoted = true;
                    }
                    out.push(chars[*at]);
                    *at += 1;
                }
                if !closed {
                    return Err(format!("Malformed pattern \"{whole}\""));
                }
            }
            '%' => {
                *multiplier = 100;
                out.push('%');
                *at += 1;
            }
            '\u{2030}' => {
                *multiplier = 1000;
                out.push('\u{2030}');
                *at += 1;
            }
            // `¤` is the currency sign and `¤¤` its international code, which
            // a locale with no country has none of.
            '\u{00a4}' => {
                *at += 1;
                if *at < chars.len() && chars[*at] == '\u{00a4}' {
                    *at += 1;
                    out.push_str("XXX");
                    code = true;
                } else {
                    out.push('\u{00a4}');
                }
            }
            other => {
                out.push(other);
                *at += 1;
            }
        }
    }
    Ok((out, quoted, code))
}

/// The characters a pattern gives a meaning to, so a LITERAL one has to be
/// quoted when the pattern is written back.
const SPECIAL: &str = "0#.,;%\u{2030}\u{00a4}'";

/// One affix as `toPattern` writes it: quoted as a WHOLE when it holds a
/// literal special, and never otherwise.
fn write_affix(affix: &str, quoted: bool, code: bool) -> String {
    let text = if code {
        affix.replacen("XXX", "\u{00a4}\u{00a4}", 1)
    } else {
        affix.to_owned()
    };
    let doubled: String = text
        .chars()
        .flat_map(|ch| {
            if ch == '\'' {
                vec!['\'', '\'']
            } else {
                vec![ch]
            }
        })
        .collect();
    if quoted {
        format!("'{doubled}'")
    } else {
        doubled
    }
}

fn group(digits: &str, size: usize) -> String {
    let mut out = String::new();
    for (at, ch) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(size) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{CURRENCY_SIGN, NumberPattern};
    use crate::decimal::{BigDec, Rounding};

    fn shown(pattern: &str, value: f64) -> String {
        let parsed = NumberPattern::parse(pattern).expect("parses");
        #[allow(clippy::cast_precision_loss)] // 1, 100 or 1000
        let scaled = value * parsed.multiplier() as f64;
        let text = crate::floatdec::java_double_to_string(scaled.abs());
        let decimal = BigDec::parse(&text).expect("a double's text parses");
        let exact = BigDec::from_f64_exactly(scaled.abs());
        parsed.format_scaled(&decimal, scaled.is_sign_negative(), Some(&exact))
    }

    #[test]
    fn a_pattern_is_a_handful_of_counts() {
        assert_eq!(shown("0", 1234.5678), "1235");
        assert_eq!(shown("0.00", 1234.5678), "1234.57");
        assert_eq!(shown("#,##0.00", 1_000_000.0), "1,000,000.00");
        assert_eq!(shown("#,###", 1234.5678), "1,235");
        assert_eq!(shown("000", 7.0), "007");
        assert_eq!(shown("$#,##0.00", -1234.5), "-$1,234.50");
        assert_eq!(shown("#,##0.00 units", 2.5), "2.50 units");
        assert_eq!(shown("#.##;(#.##)", -0.5), "(0.5)");
        assert_eq!(shown("0%", 0.756), "76%");
        assert_eq!(shown("'#'0", 7.0), "#7");
    }

    #[test]
    fn an_empty_part_still_shows_a_zero() {
        // `#` allows no integer digit and no fraction, so 0.5 has nothing to
        // print — and a JDK prints a single `0` rather than nothing.
        assert_eq!(shown("#", 0.5), "0");
        assert_eq!(shown("#.##", 0.5), "0.5");
        assert_eq!(shown("#,###", 0.5), "0");
        // ...and a negative that rounds to zero keeps its sign.
        assert_eq!(shown("0", -0.5), "-0");
    }

    #[test]
    fn scientific_counts_significant_digits() {
        // The count is `maximumInteger + maximumFraction`, and the fraction
        // limit does not cap what is printed.
        assert_eq!(shown("0.###E0", 12345.0), "1.235E4");
        assert_eq!(shown("##.00#E0", -707_326_118.0), "-7.0733E8");
        assert_eq!(shown("000.##E00", 8337.0), "833.7E01");
        assert_eq!(shown("#E00", -4723.8609), "-.5E04");
        // A zero has no exponent to speak of, and grows a fraction instead.
        assert_eq!(shown("###.#E00", 0.0), "0E00");
        assert_eq!(shown("###.00#E0", 0.0), "0.0E0");
    }

    #[test]
    fn a_tie_is_broken_by_where_the_double_really_sat() {
        // `0.05` is above the tie, so it rounds up; `1.005` is below it.
        assert_eq!(shown("0.0", 0.05), "0.1");
        assert_eq!(shown("0.00", 1.005), "1.00");
        // An exact half rounds to EVEN...
        assert_eq!(shown("0", 2.5), "2");
        assert_eq!(shown("0", 1.5), "2");
        // ...unless the value is whole, where the `.0` a `Double.toString`
        // writes puts the rounding position one short of the last digit.
        assert_eq!(shown("0.###E0", 12335.0), "1.234E4");
        // A tie that underflows to zero rounds down once the shortest form is
        // scientific, which it is below 1e-3.
        assert_eq!(shown("0.00", 0.005), "0.01");
        assert_eq!(shown("0.000", 0.0005), "0.000");
    }

    #[test]
    fn to_pattern_writes_one_position_above_the_minimum() {
        for (given, want) in [
            ("0", "#0"),
            ("0.00", "#0.00"),
            ("#.##", "#0.##"),
            ("#,##0.00", "#,##0.00"),
            ("#,###", "#,###"),
            ("#", "#"),
            ("000", "#000"),
            ("0.###E0", "0.###E0"),
            ("'#'0", "'#'#0"),
            ("+0;-0", "+#0;-#0"),
            ("'x'0", "x#0"),
        ] {
            assert_eq!(
                NumberPattern::parse(given).expect("parses").to_pattern(),
                want,
                "toPattern of {given}"
            );
        }
    }

    #[test]
    fn the_refusals_are_a_jdks_own() {
        assert!(NumberPattern::parse("0.0.0").is_err());
        assert!(NumberPattern::parse(";0").is_err());
        assert!(NumberPattern::parse("0;0;0").is_err());
        assert!(NumberPattern::parse("'").is_err());
        assert!(NumberPattern::parse("0E").is_err());
        assert!(NumberPattern::parse("0,").is_err());
        // ...and an empty half is simply the derived one.
        assert!(NumberPattern::parse("0;").is_ok());
    }

    #[test]
    fn reading_a_number_back() {
        let money = NumberPattern::parse("#,##0.00").expect("parses");
        assert_eq!(
            money.parse_number("1,234.56").expect("a number").to_text(),
            "1234.56"
        );
        assert_eq!(money.parse_number("abc"), None);
        assert_eq!(
            money.parse_number("12abc").expect("a number").to_text(),
            "12"
        );
        // A locale with no country has the generic currency sign, which is
        // what a JDK with `LANG=en` answers with too.
        let currency = NumberPattern::parse(&format!("{CURRENCY_SIGN}#,##0.00")).expect("parses");
        assert_eq!(
            currency.format(&BigDec::from_i64(5), true, None),
            format!("-{CURRENCY_SIGN}5.00")
        );
        assert_eq!(currency.rounding, Rounding::HalfEven);
    }
}
