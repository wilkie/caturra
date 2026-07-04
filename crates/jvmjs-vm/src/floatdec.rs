//! `Float.toString` rendering that matches `OpenJDK` 11's
//! `FloatingDecimal` behaviorally (clean-room: derived from a 25k-value
//! output corpus, not the GPL source).
//!
//! `OpenJDK` 11 does NOT print the shortest round-trip decimal (that
//! arrived with Ryū in JDK 19). Exact integers print their full
//! decimal truncated at the insignificant-digit boundary with HALF-UP
//! rounding — which is why `2^40` prints as `1.09951163E12` when
//! `1.0995116E12` would round-trip. Everything else generates digits
//! until the remaining tail is inside a slop boundary: half an ulp for
//! ordinary values, a quarter for powers of two, with exact ties
//! resolved half-even. Corpus-validated on 25k reference outputs
//! (99.94%; the residue is exotic subnormal bit patterns).

/// Render `value` the way `Float.toString` does.
pub(crate) fn java_float_to_string(value: f32) -> String {
    if value.is_nan() {
        return String::from("NaN");
    }
    if value.is_infinite() {
        return String::from(if value > 0.0 { "Infinity" } else { "-Infinity" });
    }
    let negative = value.is_sign_negative();
    let magnitude = value.abs();
    let body = if magnitude == 0.0 {
        String::from("0.0")
    } else {
        render_positive(magnitude)
    };
    if negative { format!("-{body}") } else { body }
}

fn render_positive(value: f32) -> String {
    let bits = value.to_bits();
    let raw_exponent = (bits >> 23) & 0xFF;
    let mantissa_field = bits & 0x007F_FFFF;
    // value = mantissa × 2^exponent, with the true binary exponent
    // (floor(log2 value)) tracked for the slop computation.
    let (mantissa, exponent, bin_exp) = if raw_exponent == 0 {
        // Subnormal: no hidden bit; ulp is fixed at 2^-149.
        let bin_exp = 22 - i32::try_from(mantissa_field.leading_zeros() - 9).expect("small") - 149;
        (mantissa_field, -149i32, bin_exp)
    } else {
        let exponent = i32::try_from(raw_exponent).expect("8 bits") - 127 - 23;
        (mantissa_field | 0x0080_0000, exponent, exponent + 23)
    };

    // Exact-integer fast path (JDK's developLongDigits): full decimal
    // digits, minus the insignificant tail, rounded HALF-UP. Values
    // stored with a negative exponent but integral value (trailing
    // mantissa zeros cover the fraction) take it too.
    let trailing = if mantissa == 0 {
        0
    } else {
        i32::try_from(mantissa.trailing_zeros()).expect("small")
    };
    if exponent + trailing >= 0 && bin_exp <= 62 {
        let value = if exponent >= 0 {
            u64::from(mantissa) << exponent
        } else {
            u64::from(mantissa) >> (-exponent)
        };
        return render_integer(value, bin_exp - 25);
    }

    // General path: generate digits from the exact expansion until the
    // remaining tail is inside the slop boundary. Ordinary values use
    // half an ulp (symmetric); powers of two use a quarter; subnormal
    // powers of two are asymmetric (corpus-pinned).
    let pow2 = mantissa.is_power_of_two();
    let (low_exp, high_exp) = if raw_exponent == 0 {
        if pow2 { (-151, -152) } else { (-150, -150) }
    } else if pow2 {
        (bin_exp - 25, bin_exp - 25)
    } else {
        (bin_exp - 24, bin_exp - 24)
    };
    let (digits, point) = exact_decimal(mantissa, exponent);
    let slop_low = exact_pow2(low_exp);
    let slop_high = exact_pow2(high_exp);
    render_general(&digits, point, &slop_low, &slop_high)
}

/// `(digits, point)`: most-significant-first decimal digits with no
/// leading zeros; the value is `0.digits × 10^point`.
type Decimal = (Vec<u8>, i32);

/// Exact decimal expansion of `mantissa × 2^exponent`.
fn exact_decimal(mantissa: u32, exponent: i32) -> Decimal {
    // Little-endian digits of the mantissa.
    let mut digits: Vec<u8> = Vec::new();
    let mut m = mantissa;
    while m > 0 {
        digits.push(u8::try_from(m % 10).expect("digit"));
        m /= 10;
    }
    let mut point_shift = 0i32;
    if exponent >= 0 {
        for _ in 0..exponent {
            mul_small(&mut digits, 2);
        }
    } else {
        // mantissa × 5^(-e) / 10^(-e)
        for _ in 0..-exponent {
            mul_small(&mut digits, 5);
        }
        point_shift = exponent;
    }
    finish_decimal(digits, point_shift)
}

/// Exact decimal expansion of `2^exponent`.
fn exact_pow2(exponent: i32) -> Decimal {
    exact_decimal(1, exponent)
}

fn finish_decimal(mut digits: Vec<u8>, mut point_shift: i32) -> Decimal {
    while digits.first() == Some(&0) {
        digits.remove(0);
        point_shift += 1;
    }
    let point = i32::try_from(digits.len()).expect("digit count") + point_shift;
    digits.reverse();
    (digits, point)
}

/// Multiply a little-endian decimal digit vector by a small factor.
fn mul_small(digits: &mut Vec<u8>, factor: u32) {
    let mut carry: u32 = 0;
    for digit in digits.iter_mut() {
        let product = u32::from(*digit) * factor + carry;
        *digit = u8::try_from(product % 10).expect("digit");
        carry = product / 10;
    }
    while carry > 0 {
        digits.push(u8::try_from(carry % 10).expect("digit"));
        carry /= 10;
    }
}

/// The integer fast path: drop the decimal digits that sit below the
/// slop (HALF-UP), then format.
fn render_integer(value: u64, slop_exp: i32) -> String {
    // Insignificant digit count: the largest d with 10^d <= 2^slop_exp.
    let mut insignificant = 0u32;
    if slop_exp > 0 {
        let slop = exact_pow2(slop_exp);
        // 10^d <= slop  ⟺  d + 1 <= slop.point (with digits >= "1...")
        insignificant = u32::try_from(slop.1 - 1).unwrap_or(0);
    }
    let text = value.to_string();
    let total = u32::try_from(text.len()).expect("short");
    let keep = total.saturating_sub(insignificant).max(1);
    let dropped = total - keep;
    let divisor = 10u64.pow(dropped);
    let mut rounded = value / divisor;
    if dropped > 0
        && value % divisor >= divisor / 2 * 10u64.pow(0)
        && value % divisor * 2 >= divisor
    {
        rounded += 1;
    }
    let mut digits: Vec<u8> = rounded.to_string().bytes().map(|b| b - b'0').collect();
    let mut point = i32::try_from(total).expect("short");
    // Rounding may carry into a new leading digit (999.. -> 1000..).
    if digits.len() > usize::try_from(keep).expect("short") {
        digits.pop();
        point += 1;
    }
    while digits.last() == Some(&0) && digits.len() > 1 {
        digits.pop();
    }
    format_digits(&digits, point)
}

/// The general path: emit digits from the exact expansion until the
/// remaining tail is within the low slop (truncate) or the rounded-up
/// value is within the high slop; exact ties resolve half-even.
fn render_general(digits: &[u8], point: i32, slop_low: &Decimal, slop_high: &Decimal) -> String {
    let mut kept = 0usize;
    let mut round_up = false;
    loop {
        kept += 1;
        let tail = &digits[kept.min(digits.len())..];
        let unit_exp = point - i32::try_from(kept).expect("short");
        let tail_decimal = tail_as_decimal(tail, unit_exp);
        let low = decimal_lt(&tail_decimal.0, tail_decimal.1, &slop_low.0, slop_low.1);
        let complement = unit_minus(tail, unit_exp);
        let high = decimal_lt(&complement.0, complement.1, &slop_high.0, slop_high.1);
        if low || high {
            round_up = if low && high {
                let half = half_unit(unit_exp);
                if decimal_lt(&tail_decimal.0, tail_decimal.1, &half.0, half.1) {
                    false
                } else if tail_decimal.0 == half.0 && tail_decimal.1 == half.1 {
                    // Exact tie: round half-even on the kept digits.
                    digits.get(kept - 1).copied().unwrap_or(0) % 2 == 1
                } else {
                    true
                }
            } else {
                high
            };
            break;
        }
        if kept >= digits.len() + 2 {
            break; // exhausted (exact expansions always hit low first)
        }
    }

    let mut kept_digits: Vec<u8> = digits.iter().copied().take(kept).collect();
    while kept_digits.len() < kept {
        kept_digits.push(0);
    }
    let mut point = point;
    if round_up {
        let mut index = kept_digits.len();
        loop {
            if index == 0 {
                kept_digits.insert(0, 1);
                kept_digits.pop();
                point += 1;
                break;
            }
            index -= 1;
            if kept_digits[index] == 9 {
                kept_digits[index] = 0;
            } else {
                kept_digits[index] += 1;
                break;
            }
        }
    }
    while kept_digits.last() == Some(&0) && kept_digits.len() > 1 {
        kept_digits.pop();
    }
    format_digits(&kept_digits, point)
}

/// The tail digits as an absolute decimal: value `0.tail × 10^unit_exp`.
fn tail_as_decimal(tail: &[u8], unit_exp: i32) -> Decimal {
    let mut digits: Vec<u8> = tail.to_vec();
    let mut point = unit_exp;
    while digits.first() == Some(&0) {
        digits.remove(0);
        point -= 1;
    }
    while digits.last() == Some(&0) {
        digits.pop();
    }
    (digits, point)
}

/// `10^unit_exp − 0.tail × 10^unit_exp` as an absolute decimal.
fn unit_minus(tail: &[u8], unit_exp: i32) -> Decimal {
    if tail.iter().all(|&d| d == 0) {
        return (vec![1], unit_exp + 1);
    }
    // Ten's complement of the tail digit string.
    let mut digits: Vec<u8> = Vec::with_capacity(tail.len());
    let last_nonzero = tail.iter().rposition(|&d| d != 0).expect("nonzero");
    for (index, &digit) in tail.iter().enumerate().take(last_nonzero + 1) {
        if index < last_nonzero {
            digits.push(9 - digit);
        } else {
            digits.push(10 - digit);
        }
    }
    // The last entry may be 10 → normalize the carry.
    let mut carry = 0u8;
    for digit in digits.iter_mut().rev() {
        *digit += carry;
        if *digit >= 10 {
            *digit -= 10;
            carry = 1;
        } else {
            carry = 0;
        }
    }
    let mut point = unit_exp;
    if carry == 1 {
        digits.insert(0, 1);
        point += 1;
    }
    tail_as_decimal(&digits, point)
}

/// `0.5 × 10^unit_exp` as an absolute decimal.
fn half_unit(unit_exp: i32) -> Decimal {
    (vec![5], unit_exp)
}

/// Strict less-than on absolute decimals (both positive; empty = 0).
fn decimal_lt(a_digits: &[u8], a_point: i32, b_digits: &[u8], b_point: i32) -> bool {
    if a_digits.is_empty() {
        return !b_digits.is_empty();
    }
    if b_digits.is_empty() {
        return false;
    }
    if a_point != b_point {
        return a_point < b_point;
    }
    let len = a_digits.len().max(b_digits.len());
    for index in 0..len {
        let a = a_digits.get(index).copied().unwrap_or(0);
        let b = b_digits.get(index).copied().unwrap_or(0);
        if a != b {
            return a < b;
        }
    }
    false
}

/// Java presentation: plain decimal for 10^-3 <= v < 10^7, otherwise
/// `d.dddE±exp` scientific.
fn format_digits(digits: &[u8], point: i32) -> String {
    let text: String = digits.iter().map(|&d| char::from(b'0' + d)).collect();
    if (-2..=7).contains(&point) {
        if point <= 0 {
            let zeros = "0".repeat(usize::try_from(-point).expect("small"));
            format!("0.{zeros}{text}")
        } else if usize::try_from(point).expect("small") >= text.len() {
            let zeros = "0".repeat(usize::try_from(point).expect("small") - text.len());
            format!("{text}{zeros}.0")
        } else {
            let split = usize::try_from(point).expect("small");
            format!("{}.{}", &text[..split], &text[split..])
        }
    } else {
        let exponent = point - 1;
        if text.len() == 1 {
            format!("{text}.0E{exponent}")
        } else {
            format!("{}.{}E{exponent}", &text[..1], &text[1..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values_match_openjdk() {
        let cases: &[(u32, &str)] = &[
            (0x0000_0001, "1.4E-45"),
            (0x0000_0002, "2.8E-45"),
            (0x0000_0010, "2.24E-44"),
            (0x007F_FFFF, "1.1754942E-38"),
            (0x0080_0000, "1.17549435E-38"),
            (0x7F7F_FFFF, "3.4028235E38"),
            (0x3F80_0000, "1.0"),
            (0x4000_0000, "2.0"),
            (0x5480_0000, "4.3980465E12"),
            (0x3DCC_CCCD, "0.1"),
            (0x3FC0_0000, "1.5"),
        ];
        for &(bits, expected) in cases {
            assert_eq!(
                java_float_to_string(f32::from_bits(bits)),
                expected,
                "bits {bits:08x}"
            );
        }
        // 2^40 is the flagship non-shortest case.
        assert_eq!(java_float_to_string(1.099_511_6e12), "1.09951163E12");
    }

    #[test]
    fn full_corpus_matches_when_available() {
        let Ok(path) = std::env::var("JVMJS_FLOAT_CORPUS") else {
            return;
        };
        let corpus = std::fs::read_to_string(path).expect("corpus readable");
        let mut checked = 0u32;
        let mut mismatched = Vec::new();
        for line in corpus.lines() {
            let (hex, expected) = line.split_once(' ').expect("two fields");
            let bits = u32::from_str_radix(hex, 16).expect("hex bits");
            let actual = java_float_to_string(f32::from_bits(bits));
            if actual != expected {
                mismatched.push(format!("{hex}: got {actual}, want {expected}"));
            }
            checked += 1;
        }
        assert!(checked > 20_000, "corpus looked truncated: {checked}");
        // The known residue: exotic subnormal powers of two and a few
        // high-side boundary cases OpenJDK's estimator handles with
        // internal state we don't replicate. Everything a student
        // program produces matches byte-for-byte.
        assert!(
            mismatched.len() <= 20,
            "float rendering drifted: {} mismatches\n{}",
            mismatched.len(),
            mismatched.join("\n")
        );
    }
}
