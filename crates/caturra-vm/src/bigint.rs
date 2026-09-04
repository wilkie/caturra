//! Arbitrary-precision integers, for `java.math.BigInteger`.
//!
//! Sign-and-magnitude over base-2^32 limbs, little-endian, with no trailing
//! zero limb — so zero is the empty magnitude and is never negative. Written
//! here rather than taken as a dependency for the same reason the regex engine
//! and the float formatter are: this crate compiles to WASM optimized for
//! SIZE, and every behaviour has to be exactly a JDK's anyway, which a general
//! library would not give for free (`mod` versus `remainder`, the two's
//! complement bit operations over an infinitely sign-extended value, and the
//! wording of six exceptions).

use core::cmp::Ordering;

/// A signed arbitrary-precision integer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BigInt {
    /// Zero is `false` with an empty magnitude: there is one representation of
    /// it, so `==` and the hash agree with `compareTo`.
    negative: bool,
    /// Base-2^32 limbs, least significant first, with no trailing zero.
    mag: Vec<u32>,
}

impl BigInt {
    #[must_use]
    pub fn zero() -> Self {
        Self {
            negative: false,
            mag: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_i64(value: i64) -> Self {
        let negative = value < 0;
        // `-i64::MIN` overflows; the unsigned magnitude does not.
        let magnitude = value.unsigned_abs();
        let mut mag = vec![
            u32::try_from(magnitude & 0xffff_ffff).unwrap_or(0),
            u32::try_from(magnitude >> 32).unwrap_or(0),
        ];
        trim(&mut mag);
        Self {
            negative: negative && !mag.is_empty(),
            mag,
        }
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.mag.is_empty()
    }

    #[must_use]
    pub fn signum(&self) -> i32 {
        if self.mag.is_empty() {
            0
        } else if self.negative {
            -1
        } else {
            1
        }
    }

    #[must_use]
    pub fn negated(&self) -> Self {
        Self {
            negative: !self.negative && !self.mag.is_empty(),
            mag: self.mag.clone(),
        }
    }

    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            negative: false,
            mag: self.mag.clone(),
        }
    }

    #[must_use]
    pub fn compare(&self, other: &Self) -> Ordering {
        match (self.signum(), other.signum()) {
            (a, b) if a != b => a.cmp(&b),
            (-1, _) => cmp_mag(&other.mag, &self.mag),
            _ => cmp_mag(&self.mag, &other.mag),
        }
    }

    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        if self.negative == other.negative {
            return Self::of(self.negative, add_mag(&self.mag, &other.mag));
        }
        match cmp_mag(&self.mag, &other.mag) {
            Ordering::Equal => Self::zero(),
            Ordering::Greater => Self::of(self.negative, sub_mag(&self.mag, &other.mag)),
            Ordering::Less => Self::of(other.negative, sub_mag(&other.mag, &self.mag)),
        }
    }

    #[must_use]
    pub fn subtract(&self, other: &Self) -> Self {
        self.add(&other.negated())
    }

    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        Self::of(
            self.negative != other.negative,
            mul_mag(&self.mag, &other.mag),
        )
    }

    /// Truncating division and its remainder — Java's `divide`/`remainder`,
    /// where the quotient rounds toward zero and the remainder takes the
    /// DIVIDEND's sign. `None` when the divisor is zero.
    #[must_use]
    pub fn divide_and_remainder(&self, other: &Self) -> Option<(Self, Self)> {
        if other.is_zero() {
            return None;
        }
        let (q, r) = divmod_mag(&self.mag, &other.mag);
        Some((
            Self::of(self.negative != other.negative, q),
            Self::of(self.negative, r),
        ))
    }

    /// Java's `mod`: always in `0..modulus`, so it differs from `remainder`
    /// for a negative dividend. `None` when the modulus is not positive.
    #[must_use]
    pub fn modulus(&self, other: &Self) -> Option<Self> {
        if other.signum() <= 0 {
            return None;
        }
        let (_, r) = self.divide_and_remainder(other)?;
        Some(if r.negative { r.add(other) } else { r })
    }

    #[must_use]
    pub fn pow(&self, exponent: u32) -> Self {
        let mut result = Self::from_i64(1);
        let mut base = self.clone();
        let mut left = exponent;
        while left > 0 {
            if left & 1 == 1 {
                result = result.multiply(&base);
            }
            left >>= 1;
            if left > 0 {
                base = base.multiply(&base);
            }
        }
        result
    }

    /// `base.modPow(exponent, modulus)` for a NON-NEGATIVE exponent, which is
    /// the only shape a program without modular inverses can use.
    #[must_use]
    pub fn mod_pow(&self, exponent: &Self, modulus: &Self) -> Option<Self> {
        if modulus.signum() <= 0 {
            return None;
        }
        let one = Self::from_i64(1);
        if modulus.compare(&one) == Ordering::Equal {
            return Some(Self::zero());
        }
        let mut result = one;
        let mut base = self.modulus(modulus)?;
        let mut bit = 0;
        let bits = exponent.bit_length();
        while bit < bits {
            if exponent.test_bit(bit) {
                result = result.multiply(&base).modulus(modulus)?;
            }
            base = base.multiply(&base).modulus(modulus)?;
            bit += 1;
        }
        Some(result)
    }

    /// The greatest common divisor of the two MAGNITUDES, which is what Java
    /// answers (it is never negative, and `gcd(0, 0)` is zero).
    #[must_use]
    pub fn gcd(&self, other: &Self) -> Self {
        let mut a = self.abs();
        let mut b = other.abs();
        while !b.is_zero() {
            let r = a
                .divide_and_remainder(&b)
                .map_or_else(Self::zero, |(_, r)| r);
            a = b;
            b = r.abs();
        }
        a
    }

    /// The floor of the square root. `None` for a negative value, which is
    /// Java's `ArithmeticException`.
    #[must_use]
    pub fn sqrt(&self) -> Option<Self> {
        if self.negative {
            return None;
        }
        if self.is_zero() {
            return Some(Self::zero());
        }
        // Newton's method from a power of two safely ABOVE the root, which
        // makes the sequence decrease until it reaches the floor.
        let two = Self::from_i64(2);
        let mut guess = Self::from_i64(1).shifted_left(self.bit_length() / 2 + 1);
        loop {
            let (divided, _) = self.divide_and_remainder(&guess)?;
            let (next, _) = guess.add(&divided).divide_and_remainder(&two)?;
            if next.compare(&guess) != Ordering::Less {
                break;
            }
            guess = next;
        }
        Some(guess)
    }

    /// The number of bits in the minimal two's-complement representation
    /// EXCLUDING the sign bit — Java's `bitLength`.
    #[must_use]
    pub fn bit_length(&self) -> u32 {
        if self.mag.is_empty() {
            return 0;
        }
        let top = self.magnitude_bits();
        if self.negative && self.is_power_of_two() {
            top - 1
        } else {
            top
        }
    }

    fn is_power_of_two(&self) -> bool {
        let mut seen = false;
        for limb in &self.mag {
            if *limb != 0 {
                if seen || !limb.is_power_of_two() {
                    return false;
                }
                seen = true;
            }
        }
        seen
    }

    /// Java's `bitCount`: the number of bits that DIFFER from the sign bit.
    #[must_use]
    pub fn bit_count(&self) -> u32 {
        if !self.negative {
            return self.mag.iter().map(|limb| limb.count_ones()).sum();
        }
        // A negative `-m` is `!(m - 1)`, so its bits differing from the sign
        // bit are exactly the SET bits of `m - 1`.
        let less = self.abs().subtract(&Self::from_i64(1));
        less.mag.iter().map(|limb| limb.count_ones()).sum()
    }

    /// Bit `n` of the infinite two's-complement representation.
    #[must_use]
    pub fn test_bit(&self, n: u32) -> bool {
        let limb = (n / 32) as usize;
        let shift = n % 32;
        if !self.negative {
            return self.mag.get(limb).is_some_and(|v| (v >> shift) & 1 == 1);
        }
        // `-m` is `!(m - 1)`, so the bit is the complement of `m - 1`'s.
        let less = self.abs().subtract(&Self::from_i64(1));
        less.mag.get(limb).is_none_or(|v| (v >> shift) & 1 != 1)
    }

    #[must_use]
    pub fn shifted_left(&self, by: u32) -> Self {
        if self.is_zero() || by == 0 {
            return self.clone();
        }
        let limbs = (by / 32) as usize;
        let bits = by % 32;
        let mut out = vec![0u32; limbs];
        let mut carry = 0u32;
        for limb in &self.mag {
            out.push((limb << bits) | carry);
            carry = if bits == 0 { 0 } else { limb >> (32 - bits) };
        }
        if carry != 0 {
            out.push(carry);
        }
        trim(&mut out);
        Self::of(self.negative, out)
    }

    /// An ARITHMETIC right shift: a negative value floors, as Java's does.
    #[must_use]
    pub fn shifted_right(&self, by: u32) -> Self {
        if self.is_zero() || by == 0 {
            return self.clone();
        }
        let limbs = (by / 32) as usize;
        let bits = by % 32;
        if limbs >= self.mag.len() {
            return if self.negative {
                Self::from_i64(-1)
            } else {
                Self::zero()
            };
        }
        let kept = &self.mag[limbs..];
        let mut out = Vec::with_capacity(kept.len());
        for (at, limb) in kept.iter().enumerate() {
            let high = if bits == 0 {
                0
            } else {
                kept.get(at + 1).map_or(0, |next| next << (32 - bits))
            };
            out.push((limb >> bits) | high);
        }
        trim(&mut out);
        let shifted = Self::of(self.negative, out);
        // Flooring: a negative value that lost any one bit rounds away.
        if self.negative && self.lost_bits(by) {
            shifted.subtract(&Self::from_i64(1))
        } else {
            shifted
        }
    }

    fn lost_bits(&self, by: u32) -> bool {
        (0..by).any(|bit| {
            let limb = (bit / 32) as usize;
            self.mag
                .get(limb)
                .is_some_and(|v| (v >> (bit % 32)) & 1 == 1)
        })
    }

    /// The three two's-complement bit operations, over values sign-extended to
    /// a common width.
    #[must_use]
    pub fn bit_op(&self, other: &Self, op: BitOp) -> Self {
        let width = self.mag.len().max(other.mag.len()) + 1;
        let left = self.twos_complement(width);
        let right = other.twos_complement(width);
        let mut out = Vec::with_capacity(width);
        for at in 0..width {
            out.push(match op {
                BitOp::And => left[at] & right[at],
                BitOp::Or => left[at] | right[at],
                BitOp::Xor => left[at] ^ right[at],
            });
        }
        Self::from_twos_complement(&out)
    }

    /// `~x`, which is `-x - 1`.
    #[must_use]
    pub fn not(&self) -> Self {
        self.negated().subtract(&Self::from_i64(1))
    }

    fn twos_complement(&self, width: usize) -> Vec<u32> {
        let mut out = vec![0u32; width];
        for (at, limb) in self.mag.iter().enumerate() {
            out[at] = *limb;
        }
        if !self.negative {
            return out;
        }
        let mut carry = 1u64;
        for limb in &mut out {
            let value = u64::from(!*limb) + carry;
            *limb = u32::try_from(value & 0xffff_ffff).unwrap_or(0);
            carry = value >> 32;
        }
        out
    }

    fn from_twos_complement(limbs: &[u32]) -> Self {
        let negative = limbs.last().is_some_and(|top| top >> 31 == 1);
        if !negative {
            let mut mag = limbs.to_vec();
            trim(&mut mag);
            return Self::of(false, mag);
        }
        let mut mag = Vec::with_capacity(limbs.len());
        let mut carry = 1u64;
        for limb in limbs {
            let value = u64::from(!*limb) + carry;
            mag.push(u32::try_from(value & 0xffff_ffff).unwrap_or(0));
            carry = value >> 32;
        }
        trim(&mut mag);
        Self::of(true, mag)
    }

    /// The low 32 bits of the two's complement, which is Java's `intValue`.
    #[must_use]
    pub fn to_i32(&self) -> i32 {
        // The low 32 bits of the two's complement, which is what a JDK's
        // `intValue` answers however wide the value really is.
        (self.to_i64().cast_unsigned() as u32).cast_signed()
    }

    /// The low 64 bits of the two's complement — Java's `longValue`.
    #[must_use]
    pub fn to_i64(&self) -> i64 {
        let low = u64::from(self.mag.first().copied().unwrap_or(0));
        let high = u64::from(self.mag.get(1).copied().unwrap_or(0));
        let magnitude = low | (high << 32);
        if self.negative {
            magnitude.cast_signed().wrapping_neg()
        } else {
            magnitude.cast_signed()
        }
    }

    /// `None` when the value does not fit — Java's `intValueExact`.
    #[must_use]
    pub fn to_i32_exact(&self) -> Option<i32> {
        let wide = self.to_i64_exact()?;
        i32::try_from(wide).ok()
    }

    #[must_use]
    pub fn to_i64_exact(&self) -> Option<i64> {
        if self.mag.len() > 2 {
            return None;
        }
        let low = u64::from(self.mag.first().copied().unwrap_or(0));
        let high = u64::from(self.mag.get(1).copied().unwrap_or(0));
        let magnitude = low | (high << 32);
        if self.negative {
            // `-2^63` fits and its magnitude does not, so it is asked for
            // separately.
            if magnitude == 1 << 63 {
                return Some(i64::MIN);
            }
            i64::try_from(magnitude).ok().map(i64::wrapping_neg)
        } else {
            i64::try_from(magnitude).ok()
        }
    }

    /// The modular inverse: the `x` in `0 <= x < modulus` with
    /// `self * x == 1 (mod modulus)`. `Err(false)` when the modulus is not
    /// positive, `Err(true)` when no inverse exists.
    ///
    /// # Errors
    /// See above — the two ways a JDK refuses, told apart so each gets its own
    /// message.
    pub fn mod_inverse(&self, modulus: &Self) -> Result<Self, bool> {
        if modulus.signum() <= 0 {
            return Err(false);
        }
        let one = Self::from_i64(1);
        if modulus.compare(&one) == Ordering::Equal {
            return Ok(Self::zero());
        }
        // The extended Euclidean algorithm over the reduced value.
        let Some(start) = self.modulus(modulus) else {
            return Err(false);
        };
        let (mut old_r, mut r) = (start, modulus.clone());
        let (mut old_s, mut s) = (one.clone(), Self::zero());
        while !r.is_zero() {
            let Some((quotient, _)) = old_r.divide_and_remainder(&r) else {
                return Err(true);
            };
            let next_r = old_r.subtract(&quotient.multiply(&r));
            old_r = std::mem::replace(&mut r, next_r);
            let next_s = old_s.subtract(&quotient.multiply(&s));
            old_s = std::mem::replace(&mut s, next_s);
        }
        if old_r.compare(&one) != Ordering::Equal {
            return Err(true);
        }
        old_s.modulus(modulus).ok_or(false)
    }

    /// Whether the value is prime. Deterministic below 3.3e24 (the first
    /// twelve primes are a proven witness set there); above it this is a
    /// Miller-Rabin test, which is what a JDK answers with too — hence the
    /// method's name.
    #[must_use]
    pub fn is_probable_prime(&self) -> bool {
        let value = self.abs();
        let two = Self::from_i64(2);
        if value.compare(&two) == Ordering::Equal {
            return true;
        }
        if !value.test_bit(0) || value.compare(&Self::from_i64(1)) != Ordering::Greater {
            return false;
        }
        let one = Self::from_i64(1);
        let less = value.subtract(&one);
        // `value - 1 = odd * 2^power`.
        let power = less.lowest_set_bit().unwrap_or(0);
        let odd = less.shifted_right(power);
        for base in [2i64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
            let base = Self::from_i64(base);
            if base.compare(&value) != Ordering::Less {
                continue;
            }
            let Some(mut x) = base.mod_pow(&odd, &value) else {
                return false;
            };
            if x.compare(&one) == Ordering::Equal || x.compare(&less) == Ordering::Equal {
                continue;
            }
            let mut composite = true;
            for _ in 1..power {
                x = match x.multiply(&x).modulus(&value) {
                    Some(next) => next,
                    None => return false,
                };
                if x.compare(&less) == Ordering::Equal {
                    composite = false;
                    break;
                }
            }
            if composite {
                return false;
            }
        }
        true
    }

    /// The smallest prime greater than this value.
    #[must_use]
    pub fn next_probable_prime(&self) -> Self {
        let two = Self::from_i64(2);
        if self.signum() < 0 || self.compare(&two) == Ordering::Less {
            return two;
        }
        let one = Self::from_i64(1);
        // Step to the next ODD candidate, then by twos.
        let mut candidate = self.add(&one);
        if !candidate.test_bit(0) {
            candidate = candidate.add(&one);
        }
        while !candidate.is_probable_prime() {
            candidate = candidate.add(&two);
        }
        candidate
    }

    /// The index of the lowest set bit, or `None` for zero (a JDK answers -1).
    #[must_use]
    pub fn lowest_set_bit(&self) -> Option<u32> {
        // A negative `-m` has the same lowest set bit as `m` does.
        self.mag.iter().enumerate().find_map(|(at, limb)| {
            (*limb != 0).then(|| u32::try_from(at).unwrap_or(0) * 32 + limb.trailing_zeros())
        })
    }

    /// `setBit`/`clearBit`/`flipBit`, which are the three bit operations
    /// against a single power of two.
    #[must_use]
    pub fn with_bit(&self, n: u32, how: BitOp) -> Self {
        let mask = Self::from_i64(1).shifted_left(n);
        match how {
            BitOp::And => self.bit_op(&mask.not(), BitOp::And),
            other => self.bit_op(&mask, other),
        }
    }

    /// Java's `floatValue`, rounded once from the top 25 bits for the same
    /// reason [`Self::to_f64`] rounds from the top 54.
    #[must_use]
    pub fn to_f32(&self) -> f32 {
        let bits = self.magnitude_bits();
        if bits == 0 {
            return 0.0;
        }
        let magnitude = if bits <= 24 {
            #[allow(clippy::cast_precision_loss)] // 24 bits fit a float exactly
            let exact = self.abs().to_i64() as f32;
            exact
        } else {
            let dropped = bits - 25;
            let positive = self.abs();
            let top = positive.shifted_right(dropped).to_i64().cast_unsigned();
            let sticky = positive.lost_bits(dropped);
            let mut kept = top >> 1;
            if top & 1 == 1 && (sticky || kept & 1 == 1) {
                kept += 1;
            }
            #[allow(clippy::cast_precision_loss)] // 24 bits, rounded above
            let mantissa = kept as f32;
            mantissa * 2f32.powi(i32::try_from(dropped + 1).unwrap_or(i32::MAX))
        };
        if self.negative { -magnitude } else { magnitude }
    }

    /// Java's `doubleValue`, which rounds ONCE — half to even — from the top
    /// 54 bits. Folding limb by limb instead rounds at every step, and past
    /// three limbs the accumulated error shows up as a one-ulp answer.
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        let bits = self.magnitude_bits();
        if bits == 0 {
            return 0.0;
        }
        let magnitude = if bits <= 53 {
            let mut value = 0.0f64;
            for limb in self.mag.iter().rev() {
                value = value * 4_294_967_296.0 + f64::from(*limb);
            }
            value
        } else {
            let dropped = bits - 54;
            let positive = self.abs();
            // The top 54 bits: 53 of mantissa plus the one that decides the
            // rounding, with everything below it folded into `sticky`.
            let top = positive.shifted_right(dropped).to_i64().cast_unsigned();
            let sticky = positive.lost_bits(dropped);
            let mut kept = top >> 1;
            if top & 1 == 1 && (sticky || kept & 1 == 1) {
                kept += 1;
            }
            // An exponent past a double's range makes this infinite, which is
            // the answer a JDK gives too.
            #[allow(clippy::cast_precision_loss)] // 53 bits, rounded above
            let mantissa = kept as f64;
            mantissa * 2f64.powi(i32::try_from(dropped + 1).unwrap_or(i32::MAX))
        };
        if self.negative { -magnitude } else { magnitude }
    }

    /// The bit length of the MAGNITUDE, which is not [`Self::bit_length`]: the
    /// signed one takes a bit off a negative power of two.
    fn magnitude_bits(&self) -> u32 {
        self.mag.last().map_or(0, |top| {
            u32::try_from(self.mag.len() - 1).unwrap_or(0) * 32 + (32 - top.leading_zeros())
        })
    }

    /// Java's `hashCode`: a fold over the two's-complement `int` words from
    /// the most significant down, then the sign.
    #[must_use]
    pub fn java_hash(&self) -> i32 {
        let mut hash: i32 = 0;
        for limb in self.mag.iter().rev() {
            hash = hash.wrapping_mul(31).wrapping_add(limb.cast_signed());
        }
        hash.wrapping_mul(self.signum())
    }

    /// Parse in `radix` (2..=36), with an optional leading sign. `None` for
    /// every text a JDK refuses.
    #[must_use]
    pub fn parse(text: &str, radix: u32) -> Option<Self> {
        if !(2..=36).contains(&radix) {
            return None;
        }
        let (negative, digits) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text.strip_prefix('+').unwrap_or(text)),
        };
        if digits.is_empty() {
            return None;
        }
        let mut mag: Vec<u32> = Vec::new();
        for ch in digits.chars() {
            let digit = ch.to_digit(radix)?;
            mul_small(&mut mag, radix);
            add_small(&mut mag, digit);
        }
        trim(&mut mag);
        Some(Self::of(negative, mag))
    }

    /// The text in `radix`, as `toString` gives it.
    #[must_use]
    pub fn to_text(&self, radix: u32) -> String {
        if self.is_zero() {
            return String::from("0");
        }
        let radix = if (2..=36).contains(&radix) { radix } else { 10 };
        let mut digits = Vec::new();
        let mut mag = self.mag.clone();
        while !mag.is_empty() {
            let remainder = div_small(&mut mag, radix);
            digits.push(char::from_digit(remainder, radix).unwrap_or('0'));
        }
        let mut out = String::with_capacity(digits.len() + 1);
        if self.negative {
            out.push('-');
        }
        out.extend(digits.iter().rev());
        out
    }

    fn of(negative: bool, mag: Vec<u32>) -> Self {
        let empty = mag.is_empty();
        Self {
            negative: negative && !empty,
            mag,
        }
    }
}

/// Which two's-complement operation [`BigInt::bit_op`] performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitOp {
    And,
    Or,
    Xor,
}

fn trim(mag: &mut Vec<u32>) {
    while mag.last() == Some(&0) {
        mag.pop();
    }
}

fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
    match a.len().cmp(&b.len()) {
        Ordering::Equal => {}
        other => return other,
    }
    for at in (0..a.len()).rev() {
        match a[at].cmp(&b[at]) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
    let mut carry = 0u64;
    for at in 0..a.len().max(b.len()) {
        let sum = u64::from(a.get(at).copied().unwrap_or(0))
            + u64::from(b.get(at).copied().unwrap_or(0))
            + carry;
        out.push(u32::try_from(sum & 0xffff_ffff).unwrap_or(0));
        carry = sum >> 32;
    }
    if carry != 0 {
        out.push(u32::try_from(carry).unwrap_or(0));
    }
    trim(&mut out);
    out
}

/// `a - b` where `a >= b`.
fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(a.len());
    let mut borrow = 0i64;
    for (at, limb) in a.iter().enumerate() {
        let mut value = i64::from(*limb) - i64::from(b.get(at).copied().unwrap_or(0)) - borrow;
        if value < 0 {
            value += 1 << 32;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out.push(u32::try_from(value).unwrap_or(0));
    }
    trim(&mut out);
    out
}

fn mul_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = vec![0u32; a.len() + b.len()];
    for (i, x) in a.iter().enumerate() {
        let mut carry = 0u64;
        for (j, y) in b.iter().enumerate() {
            let at = i + j;
            let value = u64::from(*x) * u64::from(*y) + u64::from(out[at]) + carry;
            out[at] = u32::try_from(value & 0xffff_ffff).unwrap_or(0);
            carry = value >> 32;
        }
        let mut at = i + b.len();
        while carry != 0 {
            let value = u64::from(out[at]) + carry;
            out[at] = u32::try_from(value & 0xffff_ffff).unwrap_or(0);
            carry = value >> 32;
            at += 1;
        }
    }
    trim(&mut out);
    out
}

fn mul_small(mag: &mut Vec<u32>, by: u32) {
    let mut carry = 0u64;
    for limb in mag.iter_mut() {
        let value = u64::from(*limb) * u64::from(by) + carry;
        *limb = u32::try_from(value & 0xffff_ffff).unwrap_or(0);
        carry = value >> 32;
    }
    if carry != 0 {
        mag.push(u32::try_from(carry).unwrap_or(0));
    }
}

fn add_small(mag: &mut Vec<u32>, value: u32) {
    let mut carry = u64::from(value);
    for limb in mag.iter_mut() {
        if carry == 0 {
            return;
        }
        let sum = u64::from(*limb) + carry;
        *limb = u32::try_from(sum & 0xffff_ffff).unwrap_or(0);
        carry = sum >> 32;
    }
    if carry != 0 {
        mag.push(u32::try_from(carry).unwrap_or(0));
    }
}

/// Divide in place by a single limb, answering the remainder.
fn div_small(mag: &mut Vec<u32>, by: u32) -> u32 {
    let mut rest = 0u64;
    for limb in mag.iter_mut().rev() {
        let value = (rest << 32) | u64::from(*limb);
        *limb = u32::try_from(value / u64::from(by)).unwrap_or(0);
        rest = value % u64::from(by);
    }
    trim(mag);
    u32::try_from(rest).unwrap_or(0)
}

/// Schoolbook long division over the magnitudes, one BIT at a time. Slower
/// than Knuth's algorithm D and short enough to be obviously right, which is
/// the trade this engine wants: a `BigInteger` in a classroom program is tens
/// of digits, not thousands.
fn divmod_mag(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
    if cmp_mag(a, b) == Ordering::Less {
        return (Vec::new(), a.to_vec());
    }
    // One limb: the fast path, which is what most programs take.
    if b.len() == 1 {
        let mut q = a.to_vec();
        let r = div_small(&mut q, b[0]);
        return (q, if r == 0 { Vec::new() } else { vec![r] });
    }
    let bits = u32::try_from(a.len()).unwrap_or(0) * 32;
    let mut quotient = vec![0u32; a.len()];
    let mut rest: Vec<u32> = Vec::new();
    for bit in (0..bits).rev() {
        // rest = rest * 2 + bit of a
        shift_left_one(&mut rest);
        let limb = (bit / 32) as usize;
        if (a[limb] >> (bit % 32)) & 1 == 1 {
            add_small(&mut rest, 1);
        }
        if cmp_mag(&rest, b) != Ordering::Less {
            rest = sub_mag(&rest, b);
            quotient[limb] |= 1 << (bit % 32);
        }
    }
    trim(&mut quotient);
    trim(&mut rest);
    (quotient, rest)
}

fn shift_left_one(mag: &mut Vec<u32>) {
    let mut carry = 0u32;
    for limb in mag.iter_mut() {
        let next = *limb >> 31;
        *limb = (*limb << 1) | carry;
        carry = next;
    }
    if carry != 0 {
        mag.push(carry);
    }
}

#[cfg(test)]
mod tests {
    use super::{BigInt, BitOp};
    use core::cmp::Ordering;

    fn n(text: &str) -> BigInt {
        BigInt::parse(text, 10).expect("parses")
    }

    fn text(value: &BigInt) -> String {
        value.to_text(10)
    }

    #[test]
    fn the_four_operations_carry_their_signs() {
        let a = n("12345678901234567890");
        let b = n("-98765432109876543210");
        assert_eq!(text(&a.add(&b)), "-86419753208641975320");
        assert_eq!(text(&a.subtract(&b)), "111111111011111111100");
        assert_eq!(
            text(&a.multiply(&b)),
            "-1219326311370217952237463801111263526900"
        );
        let (quotient, rest) = b.divide_and_remainder(&a).expect("nonzero");
        // The quotient rounds toward ZERO and the remainder takes the
        // DIVIDEND's sign, which is what `divide`/`remainder` mean.
        assert_eq!(text(&quotient), "-8");
        assert_eq!(text(&rest), "-900000000090");
    }

    #[test]
    fn a_modulus_is_never_negative() {
        assert_eq!(text(&n("-7").modulus(&n("5")).expect("positive")), "3");
        assert_eq!(text(&n("7").modulus(&n("5")).expect("positive")), "2");
        // ...and a modulus that is not positive has no answer at all.
        assert!(n("7").modulus(&n("-5")).is_none());
        assert!(n("7").modulus(&n("0")).is_none());
        assert!(n("7").divide_and_remainder(&n("0")).is_none());
    }

    #[test]
    fn the_bits_are_an_infinite_twos_complement() {
        // -7 is ...11111001: three bits long, two of them differing from the
        // sign, and bit 0 set.
        let negative = n("-7");
        assert_eq!(negative.bit_length(), 3);
        assert_eq!(negative.bit_count(), 2);
        assert!(negative.test_bit(0));
        assert!(!negative.test_bit(1));
        assert!(negative.test_bit(5));
        // -1 is all ones, so nothing differs from the sign bit.
        assert_eq!(n("-1").bit_length(), 0);
        assert_eq!(n("-1").bit_count(), 0);
        // A right shift FLOORS: -7 >> 1 is -4, not -3.
        assert_eq!(text(&negative.shifted_right(1)), "-4");
        assert_eq!(text(&n("-1").shifted_right(5)), "-1");
        assert_eq!(text(&n("12").bit_op(&n("10"), BitOp::And)), "8");
        assert_eq!(text(&n("12").bit_op(&n("10"), BitOp::Or)), "14");
        assert_eq!(text(&n("12").bit_op(&n("10"), BitOp::Xor)), "6");
        assert_eq!(text(&n("12").not()), "-13");
    }

    #[test]
    fn text_round_trips_through_every_radix() {
        let value = n("18446744073709551616");
        assert_eq!(value.to_text(16), "10000000000000000");
        assert_eq!(value.to_text(36), "3w5e11264sgsg");
        assert_eq!(n("-7").to_text(2), "-111");
        for radix in 2..=36 {
            let written = value.to_text(radix);
            assert_eq!(BigInt::parse(&written, radix).expect("parses"), value);
        }
        // A radix out of range prints in ten, which is what `toString(int)`
        // documents (the CONSTRUCTOR refuses instead).
        assert_eq!(n("7").to_text(37), "7");
    }

    #[test]
    fn the_numeric_reads_round_once() {
        // Folding limb by limb would round at every step and answer one ulp
        // low here.
        assert!(
            n("12345678901234567890").to_f64().to_bits() == 1.234_567_890_123_456_7e19f64.to_bits()
        );
        assert!(
            n("18446744073709551616").to_f64().to_bits() == 1.844_674_407_370_955_2e19f64.to_bits()
        );
        assert!(n(&format!("1{}", "0".repeat(400))).to_f64().is_infinite());
        assert_eq!(n("-98765432109876543210").to_i32(), 450_461_974);
        assert_eq!(
            n("-98765432109876543210").to_i64(),
            -6_531_711_741_328_785_130
        );
        assert_eq!(n("99999999999999999999").to_i32_exact(), None);
        assert_eq!(n("-2147483648").to_i32_exact(), Some(i32::MIN));
        assert_eq!(n("12345678901234567890").java_hash(), -1_436_577_082);
        assert_eq!(n("-7").java_hash(), -7);
    }

    #[test]
    fn the_number_theory_agrees_with_a_jdk() {
        assert_eq!(text(&n("-12").gcd(&n("-18"))), "6");
        assert_eq!(
            text(
                &n("1000000000000000000000000000000")
                    .sqrt()
                    .expect("positive")
            ),
            "1000000000000000"
        );
        assert!(n("-1").sqrt().is_none());
        assert_eq!(
            text(&n("7").mod_pow(&n("128"), &n("13")).expect("positive")),
            "3"
        );
        assert_eq!(
            text(&n("3").mod_inverse(&n("11")).expect("invertible")),
            "4"
        );
        assert_eq!(n("4").mod_inverse(&n("8")), Err(true));
        assert_eq!(n("3").mod_inverse(&n("-11")), Err(false));
        assert!(n("7").is_probable_prime());
        assert!(!n("9").is_probable_prime());
        assert!(!n("1").is_probable_prime());
        assert!(n("-7").is_probable_prime());
        assert_eq!(
            text(&n("1000000000000").next_probable_prime()),
            "1000000000039"
        );
        let mut factorial = BigInt::from_i64(1);
        for step in 1..=30 {
            factorial = factorial.multiply(&BigInt::from_i64(step));
        }
        assert_eq!(text(&factorial), "265252859812191058636308480000000");
    }

    #[test]
    fn zero_has_exactly_one_representation() {
        let zero = BigInt::zero();
        assert_eq!(zero.compare(&n("0")), Ordering::Equal);
        assert_eq!(zero, n("-0"));
        assert_eq!(zero.signum(), 0);
        assert_eq!(zero.negated(), zero);
        assert_eq!(text(&n("5").multiply(&BigInt::zero())), "0");
        assert_eq!(n("5").subtract(&n("5")), zero);
        assert_eq!(zero.lowest_set_bit(), None);
    }
}
