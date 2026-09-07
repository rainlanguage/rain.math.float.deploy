//! Cross-references the library's packed mathematical constants against values
//! derived here in integer arithmetic, so the Solidity literals are checked by
//! something that did not copy them.

#[cfg(test)]
mod tests {
    use crate::Float;
    use alloy::primitives::{aliases::I224, U512};

    /// The library's `FLOAT_PI`: pi rounded to nearest at 66 decimal places,
    /// as the 67-digit coefficient at exponent -66.
    const FLOAT_PI_HEX: &str = "0xffffffbe1dd4c9e873614f593bba9c6007d9a7ac8d03a4b6c700a65cb537a1b4";

    /// The library's `FLOAT_E`, same packing.
    const FLOAT_E_HEX: &str = "0xffffffbe19cfc6ef4f44cf88f14500d013df534fcaad48fca1d5ca47bea26fcc";

    /// Decimal places the library packs the constants at.
    const PLACES: u32 = 66;

    /// Extra digits carried through the series so rounding at `PLACES` is
    /// exact.
    const GUARD: u32 = 12;

    fn ten_pow(n: u32) -> U512 {
        U512::from(10u64).pow(U512::from(n))
    }

    /// `atan(1/x) * 10^scale` by the alternating series, truncated once a term
    /// underflows the scale. Every intermediate is an integer.
    fn atan_inv(x: u64, scale: u32) -> U512 {
        let x = U512::from(x);
        let x2 = x * x;
        let mut power = ten_pow(scale) / x;
        let mut sum = U512::ZERO;
        let mut k = 0u64;
        loop {
            let term = power / U512::from(2 * k + 1);
            if term.is_zero() {
                break;
            }
            if k.is_multiple_of(2) {
                sum += term;
            } else {
                sum -= term;
            }
            power /= x2;
            k += 1;
        }
        sum
    }

    /// `pi * 10^scale` by Machin's formula.
    fn pi_scaled(scale: u32) -> U512 {
        U512::from(16u64) * atan_inv(5, scale) - U512::from(4u64) * atan_inv(239, scale)
    }

    /// `e * 10^scale` by the series sum of 1/k!.
    fn e_scaled(scale: u32) -> U512 {
        let mut term = ten_pow(scale);
        let mut sum = U512::ZERO;
        let mut k = 1u64;
        while !term.is_zero() {
            sum += term;
            term /= U512::from(k);
            k += 1;
        }
        sum
    }

    /// Drops `GUARD` digits, rounding to nearest.
    fn round_to_places(scaled: U512) -> U512 {
        let divisor = ten_pow(GUARD);
        let (q, r) = (scaled / divisor, scaled % divisor);
        if r * U512::from(2u64) >= divisor {
            q + U512::from(1u64)
        } else {
            q
        }
    }

    fn coefficient(scaled: U512) -> I224 {
        I224::from_dec_str(&round_to_places(scaled).to_string()).unwrap()
    }

    fn decimal_string(scaled: U512) -> String {
        let digits = round_to_places(scaled).to_string();
        let (int, frac) = digits.split_at(digits.len() - PLACES as usize);
        format!("{int}.{frac}")
    }

    #[test]
    fn pi_literal_is_pi_rounded_to_nearest() {
        let scaled = pi_scaled(PLACES + GUARD);
        let expected = Float::pack_lossless(coefficient(scaled), -(PLACES as i32)).unwrap();
        let literal = Float::from_hex(FLOAT_PI_HEX).unwrap();
        assert_eq!(literal.as_hex(), expected.as_hex());
        assert!(literal.eq(expected).unwrap());
    }

    #[test]
    fn pi_literal_parses_from_its_digits() {
        let s = decimal_string(pi_scaled(PLACES + GUARD));
        assert!(s.starts_with("3.14159265358979323846264338327950288"));
        let parsed = Float::parse(s).unwrap();
        let literal = Float::from_hex(FLOAT_PI_HEX).unwrap();
        assert!(literal.eq(parsed).unwrap());
    }

    #[test]
    fn e_literal_is_e_rounded_to_nearest() {
        let scaled = e_scaled(PLACES + GUARD);
        let expected = Float::pack_lossless(coefficient(scaled), -(PLACES as i32)).unwrap();
        let literal = Float::from_hex(FLOAT_E_HEX).unwrap();
        assert_eq!(literal.as_hex(), expected.as_hex());
    }

    #[test]
    fn e_literal_parses_from_its_digits() {
        let s = decimal_string(e_scaled(PLACES + GUARD));
        assert!(s.starts_with("2.71828182845904523536028747135266249"));
        let parsed = Float::parse(s).unwrap();
        let literal = Float::from_hex(FLOAT_E_HEX).unwrap();
        assert!(literal.eq(parsed).unwrap());
    }
}
