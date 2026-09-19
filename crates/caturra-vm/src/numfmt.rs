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
    /// `setDecimalSeparatorAlwaysShown(true)` — a format with no fraction
    /// digits still writes the separator, so `#` renders 5 as `5.`. It shows
    /// in the pattern too (`#.`), which is why it lives here.
    pub decimal_separator_always_shown: bool,
    /// The two questions about PARSING rather than formatting: stop at the
    /// separator, and answer a `BigDecimal` rather than a `Long`/`Double`.
    pub parse_integer_only: bool,
    pub parse_big_decimal: bool,
    /// Any of the four affixes came from a SETTER rather than the pattern.
    /// A JDK stores such an affix as a literal, and its `toPattern()` then
    /// spells both halves out.
    pub affixes_set: bool,
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
            decimal_separator_always_shown: false,
            parse_integer_only: false,
            parse_big_decimal: false,
            affixes_set: false,
            rounding: Rounding::HalfEven,
        }
    }
}

/// The currency sign for a locale with no country — which is caturra's, since
/// a sandbox has no way to know one. A JDK with `LANG=en` answers the same.
pub const CURRENCY_SIGN: char = '\u{00a4}';

/// Which of a pattern's four affixes a setter is writing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Affix {
    PositivePrefix,
    PositiveSuffix,
    NegativePrefix,
    NegativeSuffix,
}

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
        // An EMPTY negative half is simply the derived one — and so is one
        // that SAYS what the positive half already said. A negative
        // subpattern supplies nothing but the affixes, so when they are the
        // same affixes there is nothing left in it, and a JDK throws it away
        // and derives the minus sign instead: `#;#` is `#`, and `#u;#u`
        // reads `5u` as positive five rather than failing to tell the two
        // forms apart.
        let derived = halves
            .get(1)
            .filter(|half| !half.is_empty())
            .map(|negative| Self::from_subpattern(negative))
            .transpose()?
            .filter(|other| {
                other.positive_prefix != parsed.positive_prefix
                    || other.positive_suffix != parsed.positive_suffix
            });
        if let Some(other) = derived {
            parsed.negative_prefix = other.positive_prefix;
            parsed.negative_suffix = other.positive_suffix;
            parsed.negative_prefix_quoted = other.positive_prefix_quoted;
            parsed.negative_suffix_quoted = other.positive_suffix_quoted;
            parsed.negative_prefix_code = other.positive_prefix_code;
            parsed.negative_suffix_code = other.positive_suffix_code;
        } else {
            parsed.negative_prefix = format!("-{}", parsed.positive_prefix);
            parsed.negative_suffix.clone_from(&parsed.positive_suffix);
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

    /// Record an affix a SETTER supplied. A JDK stores such an affix as a
    /// literal, which is why `setNegativePrefix("-")` then `toPattern()` is
    /// `'-'` and not `-`: quoted, it no longer matches the negative half the
    /// positive one derives, so the pattern spells both out.
    pub fn set_affix(&mut self, which: Affix, text: String) {
        self.affixes_set = true;
        let needs_quoting = text.chars().any(|c| {
            matches!(
                c,
                '-' | '#' | '0' | ',' | '.' | ';' | '%' | 'E' | CURRENCY_SIGN
            )
        });
        match which {
            Affix::PositivePrefix => {
                self.positive_prefix = text;
                self.positive_prefix_quoted = needs_quoting;
                self.positive_prefix_code = false;
            }
            Affix::PositiveSuffix => {
                self.positive_suffix = text;
                self.positive_suffix_quoted = needs_quoting;
                self.positive_suffix_code = false;
            }
            Affix::NegativePrefix => {
                self.negative_prefix = text;
                self.negative_prefix_quoted = needs_quoting;
                self.negative_prefix_code = false;
            }
            Affix::NegativeSuffix => {
                self.negative_suffix = text;
                self.negative_suffix_quoted = needs_quoting;
                self.negative_suffix_code = false;
            }
        }
    }

    /// The four affixes, for the values that never reach the digits: a JDK
    /// wraps an INFINITY in them and writes NaN with none at all.
    #[must_use]
    pub fn affix(&self, which: Affix) -> &str {
        match which {
            Affix::PositivePrefix => &self.positive_prefix,
            Affix::PositiveSuffix => &self.positive_suffix,
            Affix::NegativePrefix => &self.negative_prefix,
            Affix::NegativeSuffix => &self.negative_suffix,
        }
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
        // The negative half is written only when it is not the derived one —
        // or when a SETTER supplied any affix, which a JDK spells out even if
        // the text it was handed is the derived one. That is why
        // `setNegativePrefix("-")` comes back as `'-'` in the pattern: stored
        // as a literal, it is no longer the minus sign the pattern derives.
        if self.affixes_set
            || self.negative_prefix != format!("-{}", self.positive_prefix)
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
        } else if self.decimal_separator_always_shown {
            out.push('.');
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
        // The sign is taken AFTER the multiplier, which is the only way a
        // NEGATIVE one can be seen at all: `setMultiplier(-2)` writes 10
        // as -20, not as 20. A product of ZERO has no sign — which is what
        // `setMultiplier(0)` makes of every value.
        let negative = scaled.signum() != 0 && negative != (self.multiplier < 0);
        self.format_scaled(&scaled.abs(), negative, exact)
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
        if fraction.is_empty() {
            // `setDecimalSeparatorAlwaysShown(true)` writes the separator even
            // with nothing after it, so `#` renders 5 as `5.`
            if self.decimal_separator_always_shown {
                out.push('.');
            }
        } else {
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

    /// What a percent or per-mille pattern multiplies by.
    #[must_use]
    pub fn multiplier(&self) -> i64 {
        self.multiplier
    }

    /// Read a number at `start`, the way a JDK's `DecimalFormat.subparse`
    /// does: the prefix the pattern promised, then digits, then the matching
    /// suffix.
    ///
    /// # Errors
    /// The index the read failed at — a JDK's `errorIndex`, and the only
    /// thing it reports about a failure.
    #[must_use = "the reading says where the cursor stopped"]
    pub fn read(&self, text: &[char], start: usize) -> Result<Reading, usize> {
        // A prefix is chosen by LENGTH: the default pattern's positive prefix
        // is empty, so it matches everywhere, and `-7` is negative only
        // because `-` is the longer match of the two.
        let mut positive = affix_at(text, start, &self.positive_prefix);
        let mut negative = affix_at(text, start, &self.negative_prefix);
        if let (Some(plus), Some(minus)) = (positive, negative) {
            if plus > minus {
                negative = None;
            } else if plus < minus {
                positive = None;
            }
        }
        let mut at = start + positive.or(negative).ok_or(start)?;

        // `∞` stands where the digits would, affixes and all: `-∞` is the
        // negative prefix and then the symbol.
        let infinite = affix_at(text, at, INFINITY).is_some();
        let mut digits = String::new();
        let mut fraction = 0usize;
        let mut exponent = 0i32;
        if infinite {
            at += INFINITY.chars().count();
        } else {
            let mut saw_digit = false;
            let mut saw_decimal = false;
            // A grouping separator counts only once a digit follows it, so a
            // read that ends on one gives the separator back: `12,` is 12,
            // and the comma is still unread.
            let mut backup = None;
            while let Some(&character) = text.get(at) {
                // A digit is any Unicode decimal digit, not only `0`-`9`: a
                // JDK reads one with `Character.digit`, so Arabic-Indic `٥`
                // and fullwidth `５` are both the number 5.
                if let Some(value) = crate::intrinsics::nd_digit_value(character) {
                    digits.push(char::from(b'0' + u8::try_from(value).unwrap_or(0)));
                    fraction += usize::from(saw_decimal);
                    saw_digit = true;
                    backup = None;
                } else if character == DECIMAL {
                    if self.parse_integer_only || saw_decimal {
                        break;
                    }
                    saw_decimal = true;
                } else if character == GROUPING && self.grouping_used {
                    if saw_decimal {
                        break;
                    }
                    backup = Some(at);
                } else if character == EXPONENT {
                    // The exponent is a number in its own right, read by the
                    // same rules with no affix but a minus — which is why
                    // `1E-3` is a thousandth and `1E+3` is the number 1 with
                    // `E+3` left unread.
                    if let Some((value, end)) = read_exponent(text, at + 1) {
                        exponent = value;
                        at = end;
                    }
                    break;
                } else {
                    break;
                }
                at += 1;
            }
            if let Some(back) = backup {
                at = back;
            }
            if !saw_digit {
                return Err(start);
            }
        }

        // The suffix the chosen prefix promised. Neither matching (or both,
        // for a pattern whose two forms end alike) is a failure HERE, at the
        // end of the digits — which is why `(12.50rest` fails at the `r`
        // rather than reading -12.5 and leaving the tail.
        let positive = positive.and(affix_at(text, at, &self.positive_suffix));
        let negative = negative.and(affix_at(text, at, &self.negative_suffix));
        let chosen = match (positive, negative) {
            (Some(plus), Some(minus)) if plus > minus => Some((false, plus)),
            (Some(plus), Some(minus)) if plus < minus => Some((true, minus)),
            (Some(_), Some(_)) | (None, None) => None,
            (Some(plus), None) => Some((false, plus)),
            (None, Some(minus)) => Some((true, minus)),
        };
        let (negative, taken) = chosen.ok_or(at)?;
        at += taken;
        // A read that consumed nothing at all is a failure too: a
        // `ParsePosition` already past the end of the text matches every
        // empty affix and reads no digits.
        if at == start {
            return Err(at);
        }
        let magnitude = digits_to_decimal(&digits, fraction, exponent);
        Ok(Reading {
            magnitude,
            negative,
            infinite,
            end: at,
        })
    }
}

/// What reading a number off some text found.
///
/// The magnitude is UNSIGNED and the sign is its own field: `-0` is a
/// `Double` in Java precisely because the sign outlives the zero, and a
/// magnitude cannot carry one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    pub magnitude: BigDec,
    pub negative: bool,
    /// The text said `∞` where the digits would have been.
    pub infinite: bool,
    /// The index just past what was read, in CHARS.
    pub end: usize,
}

/// The decimal separator, the grouping separator, the exponent mark and the
/// two words — caturra's locale is the one with no country (see the locale
/// rule in specs/LANGUAGE.md), whose symbols are these.
const DECIMAL: char = '.';
const GROUPING: char = ',';
const EXPONENT: char = 'E';
pub const INFINITY: &str = "\u{221e}";
pub const NAN: &str = "NaN";

/// How many chars of `text` at `at` are `affix`, or `None` when it is not
/// there. An empty affix matches anywhere, which is what the default
/// pattern's positive prefix does.
pub(crate) fn affix_at(text: &[char], at: usize, affix: &str) -> Option<usize> {
    let mut length = 0;
    for character in affix.chars() {
        if text.get(at + length) != Some(&character) {
            return None;
        }
        length += 1;
    }
    Some(length)
}

/// The exponent after the mark: an optional minus and then digits. A `+` is
/// not one — the exponent is read with the same prefix rules as a number, and
/// the only prefix it has is the minus sign.
fn read_exponent(text: &[char], start: usize) -> Option<(i32, usize)> {
    let mut at = start;
    let negative = text.get(at) == Some(&'-');
    at += usize::from(negative);
    let mut digits = String::new();
    while let Some(character) = text.get(at).filter(|character| character.is_ascii_digit()) {
        digits.push(*character);
        at += 1;
    }
    // An exponent too big to be a `long` is not an exponent: a JDK reads it
    // with the same fits-into-a-long test every parse ends in, and abandons
    // the whole thing when it fails.
    let value: i64 = digits.parse().ok()?;
    let value = i32::try_from(if negative { -value } else { value }).ok()?;
    Some((value, at))
}

/// The digits, the decimal point that fell among them, and the exponent that
/// moved it, as one decimal.
fn digits_to_decimal(digits: &str, fraction: usize, exponent: i32) -> BigDec {
    let split = digits.len() - fraction;
    let (whole, rest) = digits.split_at(split);
    let whole = if whole.is_empty() { "0" } else { whole };
    let text = if rest.is_empty() {
        whole.to_owned()
    } else {
        format!("{whole}.{rest}")
    };
    let value = BigDec::parse(&text).unwrap_or_else(|_| BigDec::zero());
    // The exponent SCALES rather than multiplies: `1.5E2` read as a
    // `BigDecimal` is `1.5E+2`, scale -1, not `150`.
    value.scale_by_power_of_ten(exponent).unwrap_or(value)
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
        let read = |pattern: &NumberPattern, text: &str| {
            let chars: Vec<char> = text.chars().collect();
            pattern.read(&chars, 0)
        };
        let money = NumberPattern::parse("#,##0.00").expect("parses");
        let number = read(&money, "1,234.56").expect("a number");
        assert_eq!(number.magnitude.to_text(), "1234.56");
        assert_eq!(number.end, 8);
        assert_eq!(read(&money, "abc").expect_err("no number"), 0);
        // The cursor stops at the first character the pattern has no use
        // for, and says where: that index is a `ParsePosition`'s whole job.
        let trailing = read(&money, "12abc").expect("a number");
        assert_eq!(trailing.magnitude.to_text(), "12");
        assert_eq!(trailing.end, 2);
        // A pattern whose negative form ends in `)` INSISTS on the `)`.
        let bracketed = NumberPattern::parse("#0.00;(#)").expect("parses");
        assert_eq!(read(&bracketed, "(12.50").expect_err("no number"), 6);
        let complete = read(&bracketed, "(12.50)").expect("a number");
        assert!(complete.negative);
        assert_eq!(complete.end, 7);
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
