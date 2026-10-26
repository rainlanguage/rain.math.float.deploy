mod reference;

use alloy::primitives::U256;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use rain_math_float::Float;
use reference::{self as r, check, check_float, dec, float, run, show, Dec, RefError};

fn value() -> BoxedStrategy<U256> {
    prop_oneof![
        any::<[u8; 32]>().prop_map(U256::from_be_bytes),
        any::<u64>().prop_map(U256::from),
        (0u64..=77).prop_map(|k| U256::from(10u8).pow(U256::from(k))),
        Just(U256::MAX),
        Just(U256::ZERO),
    ]
    .boxed()
}

fn check_from(value: U256, decimals: u8) -> Result<(), TestCaseError> {
    let case = format!("from_fixed_decimal({value}, {decimals})");
    check_float(
        &case,
        Float::from_fixed_decimal(value, decimals),
        r::from_fixed_decimal_lossless(value, decimals),
    )?;
    let (got, got_lossless) = Float::from_fixed_decimal_lossy(value, decimals).unwrap();
    let (want, want_lossless) = r::from_fixed_decimal_lossy(value, decimals);
    prop_assert!(
        dec(got).eq_value(&want),
        "{case} lossy: crate {}, reference {}",
        show(&dec(got)),
        show(&want)
    );
    prop_assert_eq!(got_lossless, want_lossless, "{} lossless", case);
    Ok(())
}

fn check_to(a: &Dec, decimals: u8) -> Result<(), TestCaseError> {
    let f = float(a);
    let case = format!("to_fixed_decimal({}, {decimals})", show(a));
    check(
        &case,
        f.to_fixed_decimal(decimals),
        r::to_fixed_decimal_lossless(a, decimals),
    )?;
    check(
        &format!("{case} lossy"),
        f.to_fixed_decimal_lossy(decimals),
        r::to_fixed_decimal_lossy(a, decimals),
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn from_fixed_decimal_matches_reference(v in value(), decimals in any::<u8>()) {
        check_from(v, decimals)?;
    }

    #[test]
    fn to_fixed_decimal_matches_reference(a in r::arb_float(), decimals in any::<u8>()) {
        check_to(&a, decimals)?;
    }

    #[test]
    fn to_fixed_decimal_small_exponents(c in r::coefficient(), e in -100i64..=40, decimals in 0u8..=40) {
        check_to(&Dec::new(c, e), decimals)?;
    }
}

#[test]
fn fixed_decimal_hand_values() {
    let v = |n: u64| U256::from(n);
    let f = Float::from_fixed_decimal(v(12345), 2).unwrap();
    assert!(dec(f).eq_value(&Dec::new(12345, -2)));
    assert_eq!(f.to_fixed_decimal(2).unwrap(), v(12345));
    assert_eq!(f.to_fixed_decimal(4).unwrap(), v(1234500));
    assert_eq!(f.to_fixed_decimal_lossy(1).unwrap(), (v(1234), false));
    assert_eq!(f.to_fixed_decimal_lossy(3).unwrap(), (v(123450), true));
    let (lossy, lossless) = Float::from_fixed_decimal_lossy(U256::MAX, 0).unwrap();
    assert!(!lossless);
    assert!(dec(lossy).eq_value(&Dec::new(
        "11579208923731619542357098500868790785326998466564056403945758400791"
            .parse::<num_bigint::BigInt>()
            .unwrap(),
        10
    )));
    let (exact, lossless) = Float::from_fixed_decimal_lossy(v(5), 1).unwrap();
    assert!(lossless);
    assert!(dec(exact).eq_value(&Dec::new(5, -1)));
}

#[test]
fn fixed_decimal_reverts() {
    let revert =
        |got: Result<U256, _>, want: RefError| run(check(&format!("{want:?}"), got, Err(want)));
    revert(
        float(&Dec::new(-1, 0)).to_fixed_decimal(0),
        RefError::NegativeFixedDecimalConversion,
    );
    revert(
        float(&Dec::new(1, -1)).to_fixed_decimal(0),
        RefError::LossyConversionFromFloat,
    );
    revert(
        float(&Dec::new(1, 78)).to_fixed_decimal(0),
        RefError::FixedDecimalOverflow,
    );
    revert(
        float(&Dec::new(1, 0)).to_fixed_decimal(78),
        RefError::FixedDecimalOverflow,
    );
    assert!(r::is_revert(
        &float(&Dec::new(-1, 0))
            .to_fixed_decimal_lossy(0)
            .unwrap_err(),
        RefError::NegativeFixedDecimalConversion
    ));
    run(check_float(
        "lossy from",
        Float::from_fixed_decimal(U256::MAX, 0),
        Err(RefError::LossyConversionToFloat),
    ));
}
