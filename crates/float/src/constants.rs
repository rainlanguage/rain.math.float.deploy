//! Cross-references the constants `LibDecimalFloat` packs against values
//! derived here in integer arithmetic. The constants are read back through the
//! compiled concrete's getters, so what is checked is the library's Solidity,
//! not a copy of its digits.

#[cfg(test)]
mod tests {
    use crate::Float;
    use alloy::primitives::{aliases::I224, U512};

    /// Decimal places the library packs the constants at.
    const PLACES: u32 = 66;

    /// Extra digits carried through the series so rounding at `PLACES` is
    /// exact.
    const GUARD: u32 = 12;

    fn ten_pow(n: u32) -> U512 {
        U512::from(10u64).pow(U512::from(n))
    }

    /// `e * 10^scale` by the series sum of 1/k!. Every intermediate is an
    /// integer; the series stops once a term underflows the scale.
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
    fn e_is_e_rounded_to_nearest() {
        let scaled = e_scaled(PLACES + GUARD);
        let expected = Float::pack_lossless(coefficient(scaled), -(PLACES as i32)).unwrap();
        let e = Float::e().unwrap();
        assert_eq!(e.as_hex(), expected.as_hex());
    }

    #[test]
    fn e_parses_from_its_digits() {
        let s = decimal_string(e_scaled(PLACES + GUARD));
        assert!(s.starts_with("2.71828182845904523536028747135266249"));
        let parsed = Float::parse(s).unwrap();
        let e = Float::e().unwrap();
        assert!(e.eq(parsed).unwrap());
    }
}
