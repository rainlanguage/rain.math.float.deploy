mod reference;

use alloy::primitives::{b256, B256};
use proptest::prelude::*;
use rain_math_float::{Float, FloatError};
use reference::{self as r, dec, float, Dec};

#[test]
fn constants_are_the_documented_extremes() {
    let cases = [
        (Float::max_positive_value(), r::max_positive()),
        (Float::min_positive_value(), r::min_positive()),
        (Float::max_negative_value(), r::max_negative()),
        (Float::min_negative_value(), r::min_negative()),
        (Float::zero(), Dec::zero()),
    ];
    for (got, want) in cases {
        assert_eq!(got.unwrap().get_inner(), want.to_bytes());
    }
    assert_eq!(
        Float::max_positive_value().unwrap().get_inner(),
        b256!("7fffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffff")
    );
    assert_eq!(
        Float::min_negative_value().unwrap().get_inner(),
        b256!("7fffffff80000000000000000000000000000000000000000000000000000000")
    );
    assert_eq!(
        Float::min_positive_value().unwrap().get_inner(),
        b256!("8000000000000000000000000000000000000000000000000000000000000001")
    );
    assert_eq!(
        Float::max_negative_value().unwrap().get_inner(),
        b256!("80000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
    );
    assert_eq!(Float::zero().unwrap().get_inner(), B256::ZERO);
}

#[test]
fn raw_accessors_and_conversions() {
    let raw = b256!("fffffffe0000000000000000000000000000000000000000000000000000013a");
    let mut f = Float::from_raw(raw);
    assert_eq!(f.get_inner(), raw);
    assert!(dec(f).eq_value(&Dec::new(314, -2)));
    assert_eq!(B256::from(f), raw);
    assert_eq!(Float::from(raw).get_inner(), raw);
    f.set_inner(B256::with_last_byte(7));
    assert_eq!(f.get_inner(), B256::with_last_byte(7));
    assert_eq!(Float::default().get_inner(), B256::ZERO);
}

#[test]
fn hex_hand_values() {
    let f = float(&Dec::new(-1, -1));
    assert_eq!(f.as_hex(), "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    let f = float(&Dec::new(314, -2));
    assert_eq!(f.as_hex(), "0xfffffffe0000000000000000000000000000000000000000000000000000013a");
    let parsed = Float::from_hex("0xfffffffe0000000000000000000000000000000000000000000000000000013a").unwrap();
    assert_eq!(parsed.get_inner(), f.get_inner());
    let unprefixed = Float::from_hex("fffffffe0000000000000000000000000000000000000000000000000000013a").unwrap();
    assert_eq!(unprefixed.get_inner(), f.get_inner());
    for bad in ["0x13a", "0xzz", ""] {
        match Float::from_hex(bad) {
            Err(FloatError::InvalidHex(s)) => assert_eq!(s, bad),
            other => panic!("{bad:?}: {other:?}"),
        }
    }
}

#[test]
fn serde_is_the_hex_string() {
    let f = float(&Dec::new(314, -2));
    let json = serde_json::to_string(&f).unwrap();
    assert_eq!(json, "\"0xfffffffe0000000000000000000000000000000000000000000000000000013a\"");
    let back: Float = serde_json::from_str(&json).unwrap();
    assert_eq!(back.get_inner(), f.get_inner());
}

proptest! {
    #[test]
    fn hex_round_trips(a in r::float()) {
        let f = float(&a);
        let hex = f.as_hex();
        prop_assert_eq!(hex.clone(), format!("0x{}", alloy::hex::encode(a.to_bytes())));
        prop_assert_eq!(Float::from_hex(&hex).unwrap().get_inner(), a.to_bytes());
        let json = serde_json::to_string(&f).unwrap();
        prop_assert_eq!(json.clone(), format!("\"{hex}\""));
        prop_assert_eq!(serde_json::from_str::<Float>(&json).unwrap().get_inner(), a.to_bytes());
    }
}
