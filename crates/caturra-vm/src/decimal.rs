//! Decimal numbers of any size, for `java.math.BigDecimal`.
//!
//! A value is an arbitrary-precision UNSCALED integer and a `scale`: the number
//! it stands for is `unscaled x 10^-scale`. That is the JDK's model exactly, and
//! it has to be, because the scale is OBSERVABLE — `2.0` and `2.00` compare
//! equal and are not `equals`, they hash differently, and every operation has a
//! rule for the scale it answers with.

use crate::bigint::BigInt;
use core::cmp::Ordering;

/// A decimal number: `unscaled` x 10^-`scale`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BigDec {
    unscaled: BigInt,
    /// May be NEGATIVE, which is how `1E+3` differs from `1000`.
    scale: i32,
}

/// The eight `java.math.RoundingMode` constants, in the enum's own order (which
/// is also `ordinal()`, and the order `values()` answers).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    Up,
    Down,
    Ceiling,
    Floor,
    HalfUp,
    HalfDown,
    HalfEven,
    Unnecessary,
}

impl Rounding {
    /// The constant at `ordinal`, which is what `RoundingMode.valueOf(int)`
    /// takes and what the deprecated `BigDecimal.ROUND_*` ints are.
    #[must_use]
    pub fn from_ordinal(ordinal: i32) -> Option<Self> {
        Some(match ordinal {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::Ceiling,
            3 => Self::Floor,
            4 => Self::HalfUp,
            5 => Self::HalfDown,
            6 => Self::HalfEven,
            7 => Self::Unnecessary,
            _ => return None,
        })
    }

    #[must_use]
    pub fn ordinal(self) -> i32 {
        match self {
            Self::Up => 0,
            Self::Down => 1,
            Self::Ceiling => 2,
            Self::Floor => 3,
            Self::HalfUp => 4,
            Self::HalfDown => 5,
            Self::HalfEven => 6,
            Self::Unnecessary => 7,
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Up => "UP",
            Self::Down => "DOWN",
            Self::Ceiling => "CEILING",
            Self::Floor => "FLOOR",
            Self::HalfUp => "HALF_UP",
            Self::HalfDown => "HALF_DOWN",
            Self::HalfEven => "HALF_EVEN",
            Self::Unnecessary => "UNNECESSARY",
        }
    }

    #[must_use]
    pub fn by_name(name: &str) -> Option<Self> {
        (0..8)
            .filter_map(Self::from_ordinal)
            .find(|mode| mode.name() == name)
    }
}

/// Why an operation could not answer. Each maps to one JDK message, and they
/// are told apart because a student reads them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecError {
    /// `x / 0` where x is not zero.
    DivisionByZero,
    /// The same division, asked through `divide(divisor, scale, mode)` — where
    /// a JDK's complaint is the bare integer one, not the decimal wording. It
    /// says `BigInteger divide by zero` instead once the dividend is too big
    /// for the `long` path a JDK takes first, which is visible and so is here.
    ByZero,
    BigByZero,
    /// `0 / 0`.
    DivisionUndefined,
    /// An exact quotient that does not terminate in base ten.
    NonTerminating,
    /// A rounding was needed where `UNNECESSARY` was asked for, or an exact
    /// read found a fraction.
    RoundingNecessary,
    /// A value too big for the requested integer width.
    Overflow,
    /// `pow` with a negative exponent.
    InvalidOperation,
}

impl BigDec {
    #[must_use]
    pub fn new(unscaled: BigInt, scale: i32) -> Self {
        Self { unscaled, scale }
    }

    #[must_use]
    pub fn zero() -> Self {
        Self::new(BigInt::zero(), 0)
    }

    #[must_use]
    pub fn from_i64(value: i64) -> Self {
        Self::new(BigInt::from_i64(value), 0)
    }

    #[must_use]
    pub fn unscaled(&self) -> &BigInt {
        &self.unscaled
    }

    #[must_use]
    pub fn scale(&self) -> i32 {
        self.scale
    }

    #[must_use]
    pub fn signum(&self) -> i32 {
        self.unscaled.signum()
    }

    /// The number of digits in the unscaled value. A zero has precision 1,
    /// whatever its scale.
    #[must_use]
    pub fn precision(&self) -> u32 {
        if self.unscaled.is_zero() {
            return 1;
        }
        u32::try_from(self.unscaled.abs().to_text(10).len()).unwrap_or(u32::MAX)
    }

    #[must_use]
    pub fn negated(&self) -> Self {
        Self::new(self.unscaled.negated(), self.scale)
    }

    #[must_use]
    pub fn abs(&self) -> Self {
        Self::new(self.unscaled.abs(), self.scale)
    }

    /// The two values with a common scale — the larger of theirs, which is what
    /// every JDK comparison and sum uses.
    fn aligned(&self, other: &Self) -> (BigInt, BigInt, i32) {
        let scale = self.scale.max(other.scale);
        (
            shift_unscaled(&self.unscaled, scale - self.scale),
            shift_unscaled(&other.unscaled, scale - other.scale),
            scale,
        )
    }

    #[must_use]
    pub fn compare(&self, other: &Self) -> Ordering {
        let (left, right, _) = self.aligned(other);
        left.compare(&right)
    }

    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        let (left, right, scale) = self.aligned(other);
        Self::new(left.add(&right), scale)
    }

    #[must_use]
    pub fn subtract(&self, other: &Self) -> Self {
        let (left, right, scale) = self.aligned(other);
        Self::new(left.subtract(&right), scale)
    }

    /// The scales ADD, so `2.50 x 4.000` is `10.00000` — five decimal places
    /// that say how precisely the answer is known.
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        Self::new(
            self.unscaled.multiply(&other.unscaled),
            self.scale.saturating_add(other.scale),
        )
    }

    /// `divide(divisor)` — the EXACT quotient, or a refusal. Its scale is the
    /// "preferred" one (`this.scale - divisor.scale`) when the exact answer
    /// fits there, and the smallest that holds it otherwise.
    ///
    /// # Errors
    /// A zero divisor, and a quotient that does not terminate in base ten.
    pub fn divide(&self, other: &Self) -> Result<Self, DecError> {
        let preferred = i64::from(self.scale) - i64::from(other.scale);
        if other.unscaled.is_zero() {
            return Err(if self.unscaled.is_zero() {
                DecError::DivisionUndefined
            } else {
                DecError::DivisionByZero
            });
        }
        if self.unscaled.is_zero() {
            return Ok(Self::new(BigInt::zero(), clamp_scale(preferred)));
        }
        // Reduce, then the quotient terminates exactly when what is left of the
        // divisor is a product of 2s and 5s.
        let divisor_sign = other.unscaled.signum();
        let common = self.unscaled.gcd(&other.unscaled);
        let (numerator, _) = self
            .unscaled
            .divide_and_remainder(&common)
            .ok_or(DecError::DivisionByZero)?;
        let (denominator, _) = other
            .unscaled
            .divide_and_remainder(&common)
            .ok_or(DecError::DivisionByZero)?;
        let mut left = denominator.abs();
        let mut twos = 0u32;
        let mut fives = 0u32;
        for (factor, count) in [(2i64, &mut twos), (5i64, &mut fives)] {
            let factor = BigInt::from_i64(factor);
            while let Some((quotient, rest)) = left.divide_and_remainder(&factor) {
                if !rest.is_zero() {
                    break;
                }
                left = quotient;
                *count += 1;
            }
        }
        if left.compare(&BigInt::from_i64(1)) != Ordering::Equal {
            return Err(DecError::NonTerminating);
        }
        // n / (2^a 5^b) = n x 2^(k-a) x 5^(k-b) / 10^k for k = max(a, b).
        let power = twos.max(fives);
        let mut value = numerator;
        if divisor_sign < 0 {
            value = value.negated();
        }
        value = value.multiply(&BigInt::from_i64(2).pow(power - twos));
        value = value.multiply(&BigInt::from_i64(5).pow(power - fives));
        let scale = preferred + i64::from(power);
        let minimal = Self::new(value, clamp_scale(scale)).stripped();
        Ok(minimal.toward_scale(preferred))
    }

    /// Divide to exactly `scale` decimal places, rounding as asked.
    ///
    /// # Errors
    /// A zero divisor, and a rounding that `UNNECESSARY` forbids.
    pub fn divide_to_scale(
        &self,
        other: &Self,
        scale: i32,
        mode: Rounding,
    ) -> Result<Self, DecError> {
        if other.unscaled.is_zero() {
            return Err(if self.unscaled.to_i64_exact().is_some() {
                DecError::ByZero
            } else {
                DecError::BigByZero
            });
        }
        // Compute `this / other` at the requested scale: the numerator is
        // shifted so the integer division lands exactly there.
        let lift = i64::from(scale) - i64::from(self.scale) + i64::from(other.scale);
        let (numerator, denominator) = if lift >= 0 {
            (
                shift_unscaled(&self.unscaled, clamp_scale(lift)),
                other.unscaled.clone(),
            )
        } else {
            (
                self.unscaled.clone(),
                shift_unscaled(&other.unscaled, clamp_scale(-lift)),
            )
        };
        let rounded = round_quotient(&numerator, &denominator, mode)?;
        Ok(Self::new(rounded, scale))
    }

    /// The integer part of the quotient, kept as a decimal — `divideToIntegralValue`.
    ///
    /// # Errors
    /// A zero divisor.
    pub fn divide_to_integral(&self, other: &Self) -> Result<Self, DecError> {
        if other.unscaled.is_zero() {
            return Err(if self.unscaled.is_zero() {
                DecError::DivisionUndefined
            } else {
                DecError::DivisionByZero
            });
        }
        let (left, right, _) = self.aligned(other);
        let (quotient, _) = left
            .divide_and_remainder(&right)
            .ok_or(DecError::DivisionByZero)?;
        let preferred = i64::from(self.scale) - i64::from(other.scale);
        // A zero takes the preferred scale outright; anything else can only be
        // scaled up to it, since a lower scale would drop digits.
        if quotient.is_zero() {
            return Ok(Self::new(quotient, clamp_scale(preferred)));
        }
        Ok(Self::new(quotient, 0).stripped().toward_scale(preferred))
    }

    /// `this - this.divideToIntegralValue(other) * other`.
    ///
    /// # Errors
    /// A zero divisor.
    pub fn remainder(&self, other: &Self) -> Result<Self, DecError> {
        let integral = self.divide_to_integral(other)?;
        Ok(self.subtract(&integral.multiply(other)))
    }

    /// `setScale`, which is where every rounding mode is actually met.
    ///
    /// # Errors
    /// A rounding that `UNNECESSARY` forbids.
    pub fn with_scale(&self, scale: i32, mode: Rounding) -> Result<Self, DecError> {
        let shift = i64::from(scale) - i64::from(self.scale);
        if shift >= 0 {
            return Ok(Self::new(
                shift_unscaled(&self.unscaled, clamp_scale(shift)),
                scale,
            ));
        }
        let divisor = BigInt::from_i64(10).pow(clamp_scale(-shift).unsigned_abs());
        let rounded = round_quotient(&self.unscaled, &divisor, mode)?;
        Ok(Self::new(rounded, scale))
    }

    /// Round to `digits` significant digits — a `MathContext`, which is how a
    /// program asks for a non-terminating division.
    ///
    /// # Errors
    /// A rounding that `UNNECESSARY` forbids.
    pub fn with_precision(&self, digits: u32, mode: Rounding) -> Result<Self, DecError> {
        if digits == 0 || self.precision() <= digits {
            return Ok(self.clone());
        }
        let drop = self.precision() - digits;
        let scale = i64::from(self.scale) - i64::from(drop);
        let mut rounded = self.with_scale(clamp_scale(scale), mode)?;
        // Rounding 999 to two digits gives 1000, one digit too many.
        if rounded.precision() > digits {
            rounded = rounded.with_scale(clamp_scale(scale - 1), mode)?;
        }
        Ok(rounded)
    }

    /// `divide(divisor, mc)` — the quotient to `digits` significant digits,
    /// which is how a program asks for one that does not terminate.
    ///
    /// # Errors
    /// A zero divisor.
    pub fn divide_with_precision(
        &self,
        other: &Self,
        digits: u32,
        mode: Rounding,
    ) -> Result<Self, DecError> {
        if digits == 0 {
            return self.divide(other);
        }
        // An EXACT quotient that fits the precision is the answer, at the scale
        // the operands preferred — the context only bites when it does not fit.
        match self.divide(other) {
            Ok(exact) if exact.precision() <= digits => return Ok(exact),
            Err(error @ (DecError::DivisionByZero | DecError::DivisionUndefined)) => {
                return Err(error);
            }
            _ => {}
        }
        let scale = i64::from(digits) - 1 - self.quotient_exponent(other);
        let answer = self.divide_to_scale(other, clamp_scale(scale), mode)?;
        // Rounding 999 up to three digits gives 1000, one digit too many.
        if answer.precision() > digits {
            return self.divide_to_scale(other, clamp_scale(scale - 1), mode);
        }
        Ok(answer)
    }

    /// Where the quotient's leading digit falls — its adjusted exponent —
    /// worked out from the two operands rather than by dividing. It is the
    /// difference of theirs, one lower when this significand is the smaller.
    fn quotient_exponent(&self, other: &Self) -> i64 {
        let mine = i64::from(self.precision()) - 1 - i64::from(self.scale);
        let theirs = i64::from(other.precision()) - 1 - i64::from(other.scale);
        let mut ours = self.unscaled.abs().to_text(10);
        let mut theirs_digits = other.unscaled.abs().to_text(10);
        let width = ours.len().max(theirs_digits.len());
        while ours.len() < width {
            ours.push('0');
        }
        while theirs_digits.len() < width {
            theirs_digits.push('0');
        }
        mine - theirs - i64::from(ours < theirs_digits)
    }

    /// `pow(n)` for a non-negative `n`: the scales MULTIPLY.
    ///
    /// # Errors
    /// A negative exponent, which a JDK calls an invalid operation.
    pub fn pow(&self, exponent: i32) -> Result<Self, DecError> {
        if exponent < 0 {
            return Err(DecError::InvalidOperation);
        }
        Ok(Self::new(
            self.unscaled.pow(exponent.unsigned_abs()),
            clamp_scale(i64::from(self.scale) * i64::from(exponent)),
        ))
    }

    /// Trailing zeros removed from the unscaled value, the scale falling with
    /// them. A zero strips all the way to `0`, whatever scale it had.
    #[must_use]
    pub fn stripped(&self) -> Self {
        if self.unscaled.is_zero() {
            return Self::zero();
        }
        let ten = BigInt::from_i64(10);
        let mut value = self.unscaled.clone();
        let mut scale = self.scale;
        while let Some((quotient, rest)) = value.divide_and_remainder(&ten) {
            if !rest.is_zero() || scale == i32::MIN {
                break;
            }
            value = quotient;
            scale -= 1;
        }
        Self::new(value, scale)
    }

    /// Scale UP toward `preferred` while the value stays exact — the last step
    /// of an exact division, and what makes `2.50 / 1` answer `2.50` rather
    /// than `2.5`.
    fn toward_scale(&self, preferred: i64) -> Self {
        let want = clamp_scale(preferred);
        if want <= self.scale {
            return self.clone();
        }
        Self::new(shift_unscaled(&self.unscaled, want - self.scale), want)
    }

    /// `movePointLeft`/`movePointRight`, which never leave a NEGATIVE scale
    /// behind — unlike `scaleByPowerOfTen`, which does.
    #[must_use]
    pub fn move_point(&self, by: i32) -> Self {
        let scale = clamp_scale(i64::from(self.scale) + i64::from(by));
        let moved = Self::new(self.unscaled.clone(), scale);
        if scale < 0 {
            Self::new(shift_unscaled(&moved.unscaled, -scale), 0)
        } else {
            moved
        }
    }

    #[must_use]
    pub fn scale_by_power_of_ten(&self, by: i32) -> Self {
        Self::new(
            self.unscaled.clone(),
            clamp_scale(i64::from(self.scale) - i64::from(by)),
        )
    }

    /// The unit in the last place: 1 at this value's scale.
    #[must_use]
    pub fn ulp(&self) -> Self {
        Self::new(BigInt::from_i64(1), self.scale)
    }

    /// Truncated toward zero — `toBigInteger`.
    #[must_use]
    pub fn to_big_integer(&self) -> BigInt {
        self.with_scale(0, Rounding::Down)
            .map_or_else(|_| BigInt::zero(), |value| value.unscaled)
    }

    /// `toBigIntegerExact`.
    ///
    /// # Errors
    /// A value with a fractional part.
    pub fn to_big_integer_exact(&self) -> Result<BigInt, DecError> {
        Ok(self.with_scale(0, Rounding::Unnecessary)?.unscaled)
    }

    /// `intValueExact` and its siblings, once the width is known.
    ///
    /// # Errors
    /// A fractional part, or a value the width cannot hold.
    pub fn to_i64_exact(&self) -> Result<i64, DecError> {
        self.to_big_integer_exact()?
            .to_i64_exact()
            .ok_or(DecError::Overflow)
    }

    /// Java's `doubleValue`: correctly rounded, which the scientific text is
    /// the honest way to reach — Rust's own parser rounds once, as a JDK does.
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        self.scientific_text()
            .parse::<f64>()
            .unwrap_or(if self.signum() < 0 {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            })
    }

    #[must_use]
    pub fn to_f32(&self) -> f32 {
        self.scientific_text()
            .parse::<f32>()
            .unwrap_or(if self.signum() < 0 {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            })
    }

    fn scientific_text(&self) -> String {
        format!("{}E{}", self.unscaled.to_text(10), -i64::from(self.scale))
    }

    /// Java's `hashCode`: `31 * unscaled.hashCode() + scale`, which is why
    /// `2.0` and `2.00` land in different buckets.
    #[must_use]
    pub fn java_hash(&self) -> i32 {
        self.unscaled
            .java_hash()
            .wrapping_mul(31)
            .wrapping_add(self.scale)
    }

    /// `toString` — plain notation while the value stays readable, scientific
    /// once it does not. The rule is the javadoc's: a negative scale, or an
    /// adjusted exponent below -6, goes to scientific.
    #[must_use]
    pub fn to_text(&self) -> String {
        let digits = self.unscaled.abs().to_text(10);
        let sign = if self.unscaled.signum() < 0 { "-" } else { "" };
        let adjusted = i64::from(self.precision()) - 1 - i64::from(self.scale);
        if self.scale >= 0 && adjusted >= -6 {
            return format!("{sign}{}", plain_digits(&digits, self.scale));
        }
        // One digit, then the rest, then the adjusted exponent — with an
        // explicit sign, which `Integer.toString` would not write.
        let mut out = String::from(sign);
        out.push_str(&digits[..1]);
        if digits.len() > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        out.push('E');
        if adjusted >= 0 {
            out.push('+');
        }
        out.push_str(&adjusted.to_string());
        out
    }

    /// `toPlainString` — never an exponent, however many zeros that takes.
    #[must_use]
    pub fn to_plain_text(&self) -> String {
        let digits = self.unscaled.abs().to_text(10);
        let sign = if self.unscaled.signum() < 0 { "-" } else { "" };
        if self.scale >= 0 {
            return format!("{sign}{}", plain_digits(&digits, self.scale));
        }
        let zeros = "0".repeat(self.scale.unsigned_abs() as usize);
        if self.unscaled.is_zero() {
            return String::from("0");
        }
        format!("{sign}{digits}{zeros}")
    }

    /// `toEngineeringString` — scientific, but with the exponent a multiple of
    /// three, so it reads as thousands, millions, nanos. Two things separate it
    /// from `toString`: one to three digits sit before the point, and an
    /// exponent that works out to zero is not written at all.
    #[must_use]
    pub fn to_engineering_text(&self) -> String {
        let digits = self.unscaled.abs().to_text(10);
        let sign = if self.unscaled.signum() < 0 { "-" } else { "" };
        let adjusted = i64::from(self.precision()) - 1 - i64::from(self.scale);
        if self.scale >= 0 && adjusted >= -6 {
            return format!("{sign}{}", plain_digits(&digits, self.scale));
        }
        // How far above the multiple of three below it the exponent sits, which
        // is how many EXTRA digits move left of the point.
        let extra = adjusted.rem_euclid(3);
        let mut exponent = adjusted - extra;
        let mut out = String::from(sign);
        if self.unscaled.is_zero() {
            // A zero has no digits to move, so it grows a fraction instead and
            // the exponent climbs back to the next multiple of three.
            match extra {
                0 => out.push('0'),
                1 => {
                    out.push_str("0.00");
                    exponent += 3;
                }
                _ => {
                    out.push_str("0.0");
                    exponent += 3;
                }
            }
        } else {
            let lead = usize::try_from(extra + 1).unwrap_or(1);
            if lead >= digits.len() {
                out.push_str(&digits);
                for _ in 0..lead - digits.len() {
                    out.push('0');
                }
            } else {
                out.push_str(&digits[..lead]);
                out.push('.');
                out.push_str(&digits[lead..]);
            }
        }
        if exponent != 0 {
            out.push('E');
            if exponent > 0 {
                out.push('+');
            }
            out.push_str(&exponent.to_string());
        }
        out
    }

    /// `new BigDecimal(d)` — the EXACT value the double holds, which is the
    /// lesson: `new BigDecimal(0.1)` is not one tenth. (`valueOf(double)` goes
    /// through `Double.toString` instead, and is.)
    #[must_use]
    pub fn from_f64_exactly(value: f64) -> Self {
        let bits = value.to_bits();
        let raw_exponent = ((bits >> 52) & 0x7ff) as i32;
        let raw_mantissa = bits & 0x000f_ffff_ffff_ffff;
        let (mut mantissa, mut exponent) = if raw_exponent == 0 {
            (raw_mantissa, -1074)
        } else {
            (raw_mantissa | (1 << 52), raw_exponent - 1075)
        };
        if mantissa == 0 {
            return Self::zero();
        }
        // The mantissa is made ODD first, or the scale carries trailing zeros
        // the JDK's answer does not have.
        while mantissa & 1 == 0 {
            mantissa >>= 1;
            exponent += 1;
        }
        let magnitude = BigInt::from_i64(mantissa.cast_signed());
        let value = if exponent >= 0 {
            // m x 2^e is an integer.
            Self::new(magnitude.shifted_left(exponent.unsigned_abs()), 0)
        } else {
            // m / 2^k is m x 5^k / 10^k, exactly.
            let power = exponent.unsigned_abs();
            Self::new(
                magnitude.multiply(&BigInt::from_i64(5).pow(power)),
                clamp_scale(i64::from(power)),
            )
        };
        if value.signum() != 0 && bits >> 63 == 1 {
            value.negated()
        } else {
            value
        }
    }

    /// Parse `new BigDecimal(text)`. `Err` carries the JDK's own complaint,
    /// which names the offending character.
    ///
    /// # Errors
    /// Every text a JDK refuses, with the message it gives.
    pub fn parse(text: &str) -> Result<Self, String> {
        let bytes: Vec<char> = text.chars().collect();
        if bytes.is_empty() {
            // A JDK's message here is genuinely null.
            return Err(String::new());
        }
        let mut at = 0;
        let mut negative = false;
        if matches!(bytes[at], '+' | '-') {
            negative = bytes[at] == '-';
            at += 1;
        }
        let mut digits = String::new();
        let mut scale: i64 = 0;
        let mut seen_point = false;
        let mut any_digit = false;
        while at < bytes.len() {
            let ch = bytes[at];
            if ch.is_ascii_digit() {
                digits.push(ch);
                any_digit = true;
                if seen_point {
                    scale += 1;
                }
                at += 1;
            } else if ch == '.' {
                if seen_point {
                    return Err(String::from(
                        "Character array contains more than one decimal point.",
                    ));
                }
                seen_point = true;
                at += 1;
            } else if ch == 'e' || ch == 'E' {
                break;
            } else {
                return Err(bad_character(ch));
            }
        }
        if !any_digit {
            return Err(String::new());
        }
        if at < bytes.len() {
            // The exponent, which SHIFTS the scale rather than the digits.
            at += 1;
            let mut exponent_negative = false;
            if at < bytes.len() && matches!(bytes[at], '+' | '-') {
                exponent_negative = bytes[at] == '-';
                at += 1;
            }
            let mut exponent: i64 = 0;
            let mut any = false;
            while at < bytes.len() {
                let ch = bytes[at];
                if !ch.is_ascii_digit() {
                    return Err(bad_character(ch));
                }
                exponent = exponent
                    .saturating_mul(10)
                    .saturating_add(i64::from(ch.to_digit(10).unwrap_or(0)));
                any = true;
                at += 1;
            }
            if !any {
                return Err(String::new());
            }
            if exponent_negative {
                exponent = -exponent;
            }
            scale -= exponent;
        }
        let unscaled = BigInt::parse(&digits, 10).unwrap_or_else(BigInt::zero);
        let unscaled = if negative {
            unscaled.negated()
        } else {
            unscaled
        };
        Ok(Self::new(unscaled, clamp_scale(scale)))
    }
}

fn bad_character(ch: char) -> String {
    format!(
        "Character {ch} is neither a decimal digit number, decimal point, nor \"e\" notation exponential mark."
    )
}

/// A scale is an `int` in Java, and every overflow of one is its own error;
/// saturating is close enough for values no program will reach.
fn clamp_scale(scale: i64) -> i32 {
    i32::try_from(scale).unwrap_or(if scale < 0 { i32::MIN } else { i32::MAX })
}

fn shift_unscaled(value: &BigInt, by: i32) -> BigInt {
    if by <= 0 {
        return value.clone();
    }
    value.multiply(&BigInt::from_i64(10).pow(by.unsigned_abs()))
}

/// `digits` with a decimal point `scale` places from the right, padded with
/// leading zeros when there are not enough digits.
fn plain_digits(digits: &str, scale: i32) -> String {
    if scale <= 0 {
        return digits.to_owned();
    }
    let scale = scale.unsigned_abs() as usize;
    if digits.len() > scale {
        let split = digits.len() - scale;
        return format!("{}.{}", &digits[..split], &digits[split..]);
    }
    format!("0.{}{digits}", "0".repeat(scale - digits.len()))
}

/// `numerator / denominator`, rounded as `mode` says. This is where all eight
/// rounding modes actually live.
fn round_quotient(
    numerator: &BigInt,
    denominator: &BigInt,
    mode: Rounding,
) -> Result<BigInt, DecError> {
    let (quotient, rest) = numerator
        .divide_and_remainder(denominator)
        .ok_or(DecError::DivisionByZero)?;
    if rest.is_zero() {
        return Ok(quotient);
    }
    if mode == Rounding::Unnecessary {
        return Err(DecError::RoundingNecessary);
    }
    // The sign of the discarded part is the sign of the exact quotient, since
    // the remainder carries the dividend's.
    let negative = numerator.signum() * denominator.signum() < 0;
    // Twice the remainder against the divisor tells a half from either side.
    let doubled = rest.abs().multiply(&BigInt::from_i64(2));
    let half = doubled.compare(&denominator.abs());
    let away = match mode {
        Rounding::Up => true,
        Rounding::Down => false,
        Rounding::Ceiling => !negative,
        Rounding::Floor => negative,
        Rounding::HalfUp => half != Ordering::Less,
        Rounding::HalfDown => half == Ordering::Greater,
        Rounding::HalfEven => match half {
            Ordering::Greater => true,
            Ordering::Less => false,
            // A tie goes to the EVEN neighbour, which is the whole point of
            // this mode: it does not drift upward over many roundings.
            Ordering::Equal => quotient.test_bit(0),
        },
        Rounding::Unnecessary => unreachable!("handled above"),
    };
    if !away {
        return Ok(quotient);
    }
    let one = BigInt::from_i64(1);
    Ok(if negative {
        quotient.subtract(&one)
    } else {
        quotient.add(&one)
    })
}

#[cfg(test)]
mod tests {
    use super::{BigDec, DecError, Rounding};
    use core::cmp::Ordering;

    fn d(text: &str) -> BigDec {
        BigDec::parse(text).expect("parses")
    }

    #[test]
    fn the_scale_is_part_of_every_answer() {
        // The scales ADD on a multiply and take the larger on a sum.
        assert_eq!(d("2.50").multiply(&d("4.000")).to_text(), "10.00000");
        assert_eq!(d("2.5").add(&d("0.001")).to_text(), "2.501");
        assert_eq!(d("1E+3").add(&d("1")).to_text(), "1001");
        assert_eq!(d("2.50").negated().to_text(), "-2.50");
        // ...and it is what separates `equals` from `compareTo`.
        assert_ne!(d("2.0"), d("2.00"));
        assert_eq!(d("2.0").compare(&d("2.00")), Ordering::Equal);
        assert_eq!(d("2.0").java_hash(), 621);
        assert_eq!(d("2.00").java_hash(), 6202);
    }

    #[test]
    fn an_exact_division_prefers_the_operands_scale() {
        assert_eq!(d("2.50").divide(&d("1")).expect("exact").to_text(), "2.50");
        assert_eq!(d("10").divide(&d("4")).expect("exact").to_text(), "2.5");
        assert_eq!(d("100").divide(&d("5")).expect("exact").to_text(), "20");
        // ...and one that does not terminate is refused rather than rounded.
        assert_eq!(d("1").divide(&d("3")), Err(DecError::NonTerminating));
        assert_eq!(d("1").divide(&d("0")), Err(DecError::DivisionByZero));
        assert_eq!(d("0").divide(&d("0")), Err(DecError::DivisionUndefined));
        assert_eq!(
            d("1")
                .divide_to_scale(&d("3"), 4, Rounding::HalfUp)
                .expect("rounds")
                .to_text(),
            "0.3333"
        );
    }

    #[test]
    fn every_rounding_mode_answers_its_own_way() {
        let half = d("2.5");
        for (mode, want) in [
            (Rounding::Up, "3"),
            (Rounding::Down, "2"),
            (Rounding::Ceiling, "3"),
            (Rounding::Floor, "2"),
            (Rounding::HalfUp, "3"),
            (Rounding::HalfDown, "2"),
            // A tie goes to the EVEN neighbour, which is why this mode does
            // not drift upward over many roundings.
            (Rounding::HalfEven, "2"),
        ] {
            assert_eq!(half.with_scale(0, mode).expect("rounds").to_text(), want);
        }
        assert_eq!(
            d("3.5")
                .with_scale(0, Rounding::HalfEven)
                .map(|v| v.to_text()),
            Ok(String::from("4"))
        );
        assert_eq!(
            d("-2.1")
                .with_scale(0, Rounding::Floor)
                .map(|v| v.to_text()),
            Ok(String::from("-3"))
        );
        assert_eq!(
            d("-2.1")
                .with_scale(0, Rounding::Ceiling)
                .map(|v| v.to_text()),
            Ok(String::from("-2"))
        );
        assert_eq!(
            d("2.55").with_scale(1, Rounding::Unnecessary),
            Err(DecError::RoundingNecessary)
        );
    }

    #[test]
    fn the_three_texts_say_different_things() {
        let thousand = d("1E+3");
        assert_eq!(thousand.to_text(), "1E+3");
        assert_eq!(thousand.to_plain_text(), "1000");
        assert_eq!(thousand.to_engineering_text(), "1E+3");
        let tiny = d("0.0000001");
        assert_eq!(tiny.to_text(), "1E-7");
        assert_eq!(tiny.to_plain_text(), "0.0000001");
        assert_eq!(tiny.to_engineering_text(), "100E-9");
        // An engineering exponent that lands on zero is not written at all.
        assert_eq!(d("3E+1").to_engineering_text(), "30");
        assert_eq!(d("0E+1").to_engineering_text(), "0.00E+3");
        // There is no negative zero here, as there is none in a JDK.
        assert_eq!(d("-0.0").to_text(), "0.0");
        assert_eq!(d("600.0").stripped().to_text(), "6E+2");
        assert_eq!(d("0.000").stripped().to_text(), "0");
    }

    #[test]
    fn a_double_arrives_two_different_ways() {
        // The constructor takes the EXACT binary value...
        assert_eq!(
            BigDec::from_f64_exactly(0.1).to_text(),
            "0.1000000000000000055511151231257827021181583404541015625"
        );
        // ...and reading one back is correctly rounded.
        assert!(d("0.1").to_f64().to_bits() == 0.1f64.to_bits());
        assert!(d("1E+400").to_f64().is_infinite());
        assert!(d("1E-400").to_f64().to_bits() == 0.0f64.to_bits());
    }

    #[test]
    fn the_refusals_are_told_apart() {
        assert_eq!(d("2.5").pow(-1), Err(DecError::InvalidOperation));
        assert_eq!(d("2.01").to_i64_exact(), Err(DecError::RoundingNecessary));
        assert_eq!(d("1E+30").to_i64_exact(), Err(DecError::Overflow));
        // A zero divisor says one thing through `divide` and another through
        // the scale-and-mode form, because a JDK's implementations differ.
        assert_eq!(
            d("1").divide_to_scale(&d("0"), 2, Rounding::HalfUp),
            Err(DecError::ByZero)
        );
        assert!(BigDec::parse("1.2.3").is_err());
        assert!(BigDec::parse(" 1").is_err());
        assert!(BigDec::parse("").is_err());
    }

    #[test]
    fn a_context_keeps_significant_digits() {
        assert_eq!(
            d("1")
                .divide_with_precision(&d("3"), 5, Rounding::HalfUp)
                .expect("rounds")
                .to_text(),
            "0.33333"
        );
        assert_eq!(
            d("123.456")
                .with_precision(4, Rounding::HalfUp)
                .expect("rounds")
                .to_text(),
            "123.5"
        );
        // An EXACT quotient that fits the precision keeps the preferred scale.
        assert_eq!(
            d("10")
                .divide_with_precision(&d("1"), 2, Rounding::HalfUp)
                .expect("exact")
                .to_text(),
            "10"
        );
        assert_eq!(
            d("10")
                .divide_with_precision(&d("1"), 1, Rounding::HalfUp)
                .expect("rounds")
                .to_text(),
            "1E+1"
        );
    }
}
