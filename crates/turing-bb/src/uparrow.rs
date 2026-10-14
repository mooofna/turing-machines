//! Knuth's up-arrow notation, as far as it can be computed.
//!
//! A tower b↑↑h = b^b^…^b (h copies, evaluated from the top) is computed exactly as long as
//! its decimal expansion is short enough. Beyond that its number of digits,
//! ⌊e × log₁₀ b⌋ + 1 for b↑↑h = bᵉ, is computed from rigorous lower and upper bounds on log₁₀ b
//! obtained with integer arithmetic only, and reported only when both bounds give the same
//! integer, so a reported digit count is never an approximation.

use num_bigint::BigUint;
use num_traits::{One, ToPrimitive, Zero};

/// The largest decimal expansion [`tetration`] produces, whatever `max_digits` says.
pub const MAX_EXACT_DIGITS: usize = 1_000_000;

/// Exponents up to this many bits are handled; larger towers report no digit count.
const MAX_EXPONENT_BITS: u64 = 256;

/// The value of a tower `base ↑↑ height`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tower {
    /// The base.
    pub base: u32,
    /// The height.
    pub height: u32,
    /// The decimal digits, if there are at most the requested maximum.
    pub exact: Option<String>,
    /// The number of decimal digits, if it can be computed exactly.
    pub digits: Option<u128>,
}

/// Computes `base ↑↑ height` = base^base^...^base (`height` copies).
///
/// `exact` holds the decimal expansion if it has at most `max_digits` (and at most
/// [`MAX_EXACT_DIGITS`]) digits. `digits` is the exact number of decimal digits whenever the
/// exponent of the top level, base ↑↑ (height − 1), has at most 256 bits; for taller towers,
/// such as 2↑↑6 whose digit count itself has 19,729 digits, it is `None`. By convention
/// b↑↑0 = 1 and 0⁰ = 1.
///
/// ```
/// use turing_bb::uparrow::tetration;
/// assert_eq!(tetration(2, 4, 100).exact.as_deref(), Some("65536"));
/// assert_eq!(tetration(2, 5, 100).digits, Some(19_729));
/// assert_eq!(tetration(3, 4, 100).digits, Some(3_638_334_640_025));
/// assert_eq!(tetration(2, 6, 100).digits, None);
/// ```
pub fn tetration(base: u32, height: u32, max_digits: usize) -> Tower {
    let mut tower = Tower {
        base,
        height,
        exact: None,
        digits: None,
    };
    let max_digits = max_digits.min(MAX_EXACT_DIGITS);
    // Degenerate bases and heights: the value is 0 or 1.
    let small = match (base, height) {
        (_, 0) | (1, _) => Some(1u32),
        (0, h) => Some(u32::from(h % 2 == 0)),
        _ => None,
    };
    if let Some(value) = small {
        tower.digits = Some(1);
        tower.exact = (max_digits >= 1).then(|| value.to_string());
        return tower;
    }
    // The exponent e of the top level: base ↑↑ (height − 1).
    let mut exponent = BigUint::one();
    for _ in 1..height {
        let Some(next) = power(base, &exponent, MAX_EXPONENT_BITS) else {
            return tower;
        };
        exponent = next;
    }
    let Some(digits) = digit_count(base, &exponent) else {
        return tower;
    };
    tower.digits = digits.to_u128();
    if digits <= BigUint::from(max_digits) {
        let bits_limit = 4 * u64::try_from(max_digits).unwrap_or(u64::MAX) + 4;
        if let Some(value) = power(base, &exponent, bits_limit) {
            tower.exact = Some(value.to_string());
        }
    }
    tower
}

/// `base ^ exponent` if it has at most `max_bits` bits.
fn power(base: u32, exponent: &BigUint, max_bits: u64) -> Option<BigUint> {
    let log2_ceiling = u64::from(32 - (base - 1).leading_zeros());
    let e = exponent.to_u32()?;
    if u64::from(e).saturating_mul(log2_ceiling) > max_bits.saturating_add(log2_ceiling) {
        return None;
    }
    let value = BigUint::from(base).pow(e);
    (value.bits() <= max_bits).then_some(value)
}

/// The number of decimal digits of `base ^ exponent` (base ≥ 2): ⌊e × log₁₀ b⌋ + 1.
fn digit_count(base: u32, exponent: &BigUint) -> Option<BigUint> {
    if exponent.is_zero() {
        return Some(BigUint::one());
    }
    // Powers of ten have a rational logarithm.
    let mut k = 0u32;
    let mut rest = base;
    while rest.is_multiple_of(10) {
        rest /= 10;
        k += 1;
    }
    if rest == 1 {
        return Some(exponent * k + 1u32);
    }
    // log₁₀ b is irrational: bracket e × log₁₀ b until its integer part is certain.
    let mut precision = exponent.bits() + 64;
    for _ in 0..4 {
        let (lo, hi) = log10_bounds(base, precision);
        let floor_lo = (exponent * &lo) >> precision;
        let floor_hi = (exponent * &hi) >> precision;
        if floor_lo == floor_hi {
            return Some(floor_lo + 1u32);
        }
        precision *= 2;
    }
    None
}

/// Bounds `lo ≤ 2^p × log₁₀ n ≤ hi` for n ≥ 2.
fn log10_bounds(n: u32, p: u64) -> (BigUint, BigUint) {
    let (n_lo, n_hi) = log2_bounds(n, p);
    let (t_lo, t_hi) = log2_bounds(10, p);
    // log₁₀ n = log₂ n / log₂ 10, rounded outwards.
    let lo = (n_lo << p) / t_hi;
    let num = n_hi << p;
    let hi = (&num + &t_lo - 1u32) / t_lo;
    (lo, hi)
}

/// Bounds `lo ≤ 2^p × log₂ n ≤ hi` for n ≥ 1, with integer arithmetic only.
///
/// With n = 2ᵏ × x and x in [1, 2), the bits of log₂ x come from repeated squaring: if x² ≥ 2
/// the next bit is 1 and x becomes x²/2, otherwise it is 0 and x becomes x². Carrying out the
/// squarings in fixed point once rounded down and once rounded up (capping the upper value at
/// 2) yields a lower and an upper bound, because log₂ is increasing and every rounded value
/// stays on its side of the exact one. The remainder after p bits lies in [0, 1].
fn log2_bounds(n: u32, p: u64) -> (BigUint, BigUint) {
    let k = u64::from(n.ilog2());
    let w = p + 32;
    let one = BigUint::one() << w;
    let two = BigUint::one() << (w + 1);
    let start = BigUint::from(n) << (w - k);
    let mut lower = start.clone();
    let mut upper = start;
    let mut bits_lo = BigUint::zero();
    let mut bits_hi = BigUint::zero();
    let ulp = BigUint::one();
    for _ in 0..p {
        lower = (&lower * &lower) >> w;
        bits_lo <<= 1;
        if lower >= two {
            lower >>= 1;
            bits_lo += 1u32;
        }
        let square = &upper * &upper;
        let mut next = &square >> w;
        if (&next << w) != square {
            next += &ulp;
        }
        upper = next;
        bits_hi <<= 1;
        if upper >= two {
            let odd = upper.bit(0);
            upper >>= 1;
            if odd {
                upper += &ulp;
            }
            bits_hi += 1u32;
        }
        if upper > two {
            upper.clone_from(&two);
        }
        debug_assert!(lower >= one);
    }
    let integer = BigUint::from(k) << p;
    let lo = &integer + bits_lo;
    let hi = integer + bits_hi + 1u32;
    (lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_towers_are_exact() {
        assert_eq!(tetration(2, 0, 10).exact.as_deref(), Some("1"));
        assert_eq!(tetration(2, 1, 10).exact.as_deref(), Some("2"));
        assert_eq!(tetration(2, 2, 10).exact.as_deref(), Some("4"));
        assert_eq!(tetration(2, 3, 10).exact.as_deref(), Some("16"));
        assert_eq!(tetration(3, 3, 100).exact.as_deref(), Some("7625597484987"));
        assert_eq!(tetration(10, 2, 100).exact.as_deref(), Some("10000000000"));
        assert_eq!(tetration(1, 100, 10).exact.as_deref(), Some("1"));
        assert_eq!(tetration(0, 3, 10).exact.as_deref(), Some("0"));
        assert_eq!(tetration(0, 4, 10).exact.as_deref(), Some("1"));
    }

    #[test]
    fn two_up_up_five_is_exact_with_enough_room() {
        let t = tetration(2, 5, 20_000);
        let digits = t.exact.unwrap();
        assert_eq!(digits.len(), 19_729);
        assert!(
            digits.starts_with("2003529930406846464979072351560255750447825475569751419265016973")
        );
        assert!(digits.ends_with("6736"));
        assert_eq!(t.digits, Some(19_729));
        assert_eq!(tetration(2, 5, 19_728).exact, None);
    }

    #[test]
    fn digit_counts_of_large_towers() {
        // 10↑↑3 = 10^(10^10).
        assert_eq!(tetration(10, 3, 10).digits, Some(10_000_000_001));
        // 4↑↑3 = 4^256 = 2^512 has 155 digits.
        assert_eq!(tetration(4, 3, 10).digits, Some(155));
        assert_eq!(tetration(4, 3, 10).exact, None);
        // 3↑↑4 = 3^7625597484987: ⌊7625597484987 × log₁₀ 3⌋ + 1.
        assert_eq!(tetration(3, 4, 10).digits, Some(3_638_334_640_025));
        assert_eq!(tetration(10, 4, 10).digits, None);
    }

    #[test]
    fn digit_counts_agree_with_exact_values() {
        for base in 2..=40u32 {
            for exponent in [1u32, 2, 7, 64, 100, 1000, 4321] {
                let value = BigUint::from(base).pow(exponent);
                let expected = value.to_string().len();
                let found = digit_count(base, &BigUint::from(exponent)).unwrap();
                assert_eq!(found, BigUint::from(expected), "{base}^{exponent}");
            }
        }
    }

    #[test]
    fn log2_bounds_bracket_known_values() {
        // log₂ 3 = 1.584962500721156...
        let (lo, hi) = log2_bounds(3, 64);
        let scale = 2f64.powi(64);
        let lo = lo.to_f64().unwrap() / scale;
        let hi = hi.to_f64().unwrap() / scale;
        assert!(lo <= 1.584_962_500_721_156_3 && 1.584_962_500_721_156 <= hi);
        assert!(hi - lo < 1e-15);
        let (lo, hi) = log2_bounds(8, 64);
        assert!(lo <= BigUint::from(3u32) << 64u32 && BigUint::from(3u32) << 64u32 <= hi);
    }
}
