//! Integration tests for the `float!` proc macro.
//!
//! These run as a separate binary that consumes the proc macro crate,
//! verifying compile-time evaluation matches runtime `Float::parse`.

#![allow(clippy::unwrap_used)]

use rain_math_float::Float;
use rain_math_float_macro::{float, float_result};

#[test]
fn integer_literals() {
    assert!(float!(0).is_zero().unwrap());
    assert!(
        float!(1)
            .eq(Float::parse("1".to_string()).unwrap())
            .unwrap()
    );
    assert!(
        float!(42)
            .eq(Float::parse("42".to_string()).unwrap())
            .unwrap()
    );
    assert!(
        float!(100000)
            .eq(Float::parse("100000".to_string()).unwrap())
            .unwrap()
    );
}

#[test]
fn decimal_literals() {
    assert!(
        float!(1.5)
            .eq(Float::parse("1.5".to_string()).unwrap())
            .unwrap()
    );
    assert!(
        float!(0.001)
            .eq(Float::parse("0.001".to_string()).unwrap())
            .unwrap()
    );
    assert!(
        float!(155.00)
            .eq(Float::parse("155.00".to_string()).unwrap())
            .unwrap()
    );
    assert!(
        float!(123.456789)
            .eq(Float::parse("123.456789".to_string()).unwrap())
            .unwrap()
    );
}

#[test]
fn negative_literals() {
    assert!(
        float!(-1)
            .eq(Float::parse("-1".to_string()).unwrap())
            .unwrap()
    );
    assert!(
        float!(-42.7)
            .eq(Float::parse("-42.7".to_string()).unwrap())
            .unwrap()
    );
    assert!(
        float!(-0.5)
            .eq(Float::parse("-0.5".to_string()).unwrap())
            .unwrap()
    );
}

#[test]
fn arithmetic() {
    let sum = (float!(1.5) + float!(2.5)).unwrap();
    assert!(sum.eq(float!(4)).unwrap());

    let diff = (float!(10) - float!(3)).unwrap();
    assert!(diff.eq(float!(7)).unwrap());

    let product = (float!(6) * float!(7)).unwrap();
    assert!(product.eq(float!(42)).unwrap());

    let quotient = (float!(10) / float!(4)).unwrap();
    assert!(quotient.eq(float!(2.5)).unwrap());
}

#[test]
fn scientific_literals_keep_full_precision() {
    let cases = [
        (float!(1.234567890123456789e2), "123.4567890123456789"),
        (float!(1.234567890123456789E2), "123.4567890123456789"),
        (float!(1.234567890123456789e+2), "123.4567890123456789"),
        (float!(1.234567890123456789e-2), "0.01234567890123456789"),
        (float!(-1.234567890123456789e2), "-123.4567890123456789"),
        (float!(1e5), "100000"),
    ];
    for (from_macro, expected) in cases {
        assert!(
            from_macro
                .eq(Float::parse(expected.to_string()).unwrap())
                .unwrap(),
            "{expected}"
        );
    }

    let from_result_macro = float_result!(1.234567890123456789e2).unwrap();
    assert!(
        from_result_macro
            .eq(Float::parse("123.4567890123456789".to_string()).unwrap())
            .unwrap()
    );
}

#[test]
fn literal_is_const() {
    const ONE_AND_A_HALF: Float = float!(1.5);
    assert!(
        ONE_AND_A_HALF
            .eq(Float::parse("1.5".to_string()).unwrap())
            .unwrap()
    );
}

#[test]
fn runtime_fallback() {
    let value = "99.99".to_string();
    let from_macro = float!(&value);
    let from_parse = Float::parse("99.99".to_string()).unwrap();
    assert!(from_macro.eq(from_parse).unwrap());
}

#[test]
#[should_panic(expected = "failed")]
fn runtime_invalid_string_panics() {
    let invalid = "not_a_number".to_string();
    let _ = float!(&invalid);
}

#[test]
#[should_panic(expected = "failed")]
fn runtime_empty_string_panics() {
    let empty = String::new();
    let _ = float!(&empty);
}

#[test]
#[should_panic(expected = "failed")]
fn runtime_special_chars_panic() {
    let special = "1.2.3".to_string();
    let _ = float!(&special);
}

#[test]
fn compile_fail_tests() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/compile_fail/*.rs");
}
