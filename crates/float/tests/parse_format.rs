mod reference;

use num_bigint::BigInt;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use rain_math_float::error::DecimalFloatErrorSelector as Selector;
use rain_math_float::{Float, FloatError};
use reference::{self as r, check, dec, float, run, show, Dec, Packed, RefError};

#[derive(Debug, PartialEq)]
enum ParseRef {
    Value(Dec),
    Selector(&'static str),
}

/// A well formed literal `-?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?`.
fn literal() -> BoxedStrategy<String> {
    let digits = |max: usize| proptest::collection::vec(0u8..10, 1..=max)
        .prop_map(|d| d.into_iter().map(|x| char::from(b'0' + x)).collect::<String>());
    let exponent = prop_oneof![
        3 => (-90i64..=90),
        1 => any::<i32>().prop_map(i64::from),
        1 => (i64::from(i32::MAX) - 80..=i64::from(i32::MAX) + 80),
        1 => (i64::from(i32::MIN) - 80..=i64::from(i32::MIN) + 80),
    ];
    (
        any::<bool>(),
        digits(80),
        proptest::option::of(digits(80)),
        proptest::option::of((exponent, any::<bool>(), any::<bool>())),
    )
        .prop_map(|(neg, int, frac, exp)| {
            let mut s = String::new();
            if neg {
                s.push('-');
            }
            s.push_str(&int);
            if let Some(frac) = frac {
                s.push('.');
                s.push_str(&frac);
            }
            if let Some((e, upper, plus)) = exp {
                s.push(if upper { 'E' } else { 'e' });
                if plus && e >= 0 {
                    s.push('+');
                }
                s.push_str(&e.to_string());
            }
            s
        })
        .boxed()
}

fn literal_value(s: &str) -> Dec {
    let (mantissa, exp) = match s.find(['e', 'E']) {
        Some(i) => (&s[..i], s[i + 1..].trim_start_matches('+').parse::<i64>().unwrap()),
        None => (s, 0),
    };
    let (int, frac) = match mantissa.find('.') {
        Some(i) => (&mantissa[..i], &mantissa[i + 1..]),
        None => (mantissa, ""),
    };
    let mut c: BigInt = format!("{}{}", int.trim_start_matches('-'), frac).parse().unwrap();
    if int.starts_with('-') {
        c = -c;
    }
    Dec::new(c, exp - frac.len() as i64)
}

fn parse_ref(s: &str) -> ParseRef {
    match r::pack(&literal_value(s)) {
        Packed::Value(v, true) => ParseRef::Value(v),
        Packed::Value(_, false) | Packed::Underflow => ParseRef::Selector("ParseDecimalPrecisionLoss(uint256)"),
        Packed::Overflow => ParseRef::Selector("ExponentOverflow(int256,int256)"),
    }
}

fn selector_signature(s: &Selector) -> &'static str {
    match s {
        Selector::ExponentOverflow => "ExponentOverflow(int256,int256)",
        Selector::ParseEmptyDecimalString => "ParseEmptyDecimalString(uint256)",
        Selector::MalformedDecimalPoint => "MalformedDecimalPoint(uint256)",
        Selector::MalformedExponentDigits => "MalformedExponentDigits(uint256)",
        Selector::ParseDecimalPrecisionLoss => "ParseDecimalPrecisionLoss(uint256)",
        Selector::ParseDecimalFloatExcessCharacters => "ParseDecimalFloatExcessCharacters()",
    }
}

fn check_parse(s: &str, want: ParseRef) -> Result<(), TestCaseError> {
    match (Float::parse(s.to_string()), want) {
        (Ok(got), ParseRef::Value(w)) => {
            prop_assert!(dec(got).eq_value(&w), "parse({s:?}): crate {}, reference {}", show(&dec(got)), show(&w));
        }
        (Err(FloatError::DecimalFloatSelector(Ok(got))), ParseRef::Selector(w)) => {
            prop_assert_eq!(selector_signature(&got), w, "parse({:?})", s);        }
        (got, want) => prop_assert!(false, "parse({s:?}): crate {got:?}, reference {want:?}"),
    }
    Ok(())
}

fn check_format(a: &Dec) -> Result<(), TestCaseError> {
    let f = float(a);
    let case = format!("format({})", show(a));
    prop_assert_eq!(f.format_with_scientific(true).unwrap(), r::format_scientific(a), "{} scientific", case);
    check(&case, f.format_with_scientific(false), r::format_plain(a))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn parse_matches_reference(s in literal()) {
        check_parse(&s, parse_ref(&s))?;
    }

    #[test]
    fn format_matches_reference(a in r::float()) {
        check_format(&a)?;
    }

    #[test]
    fn format_plain_exponents(c in r::coefficient(), e in -1100i64..=1100) {
        check_format(&Dec::new(c, e))?;
    }

    #[test]
    fn format_scientific_parses_back(a in r::float()) {
        let text = float(&a).format_with_scientific(true).unwrap();
        check_parse(&text, ParseRef::Value(a))?;
    }
}

#[test]
fn parse_hand_values() {
    let cases = [
        ("0", Dec::zero()),
        ("3.14", Dec::new(314, -2)),
        ("-3.14", Dec::new(-314, -2)),
        ("1e3", Dec::new(1000, 0)),
        ("1E-3", Dec::new(1, -3)),
        ("2.5e+2", Dec::new(250, 0)),
        ("-0.000", Dec::zero()),
    ];
    for (s, want) in cases {
        run(check_parse(s, ParseRef::Value(want)));
    }
}

#[test]
fn parse_error_selectors() {
    let many_digits = format!("1.{}", "1".repeat(80));
    let cases = [
        ("", "ParseEmptyDecimalString(uint256)"),
        ("abc", "ParseEmptyDecimalString(uint256)"),
        ("-", "ParseEmptyDecimalString(uint256)"),
        ("1.", "MalformedDecimalPoint(uint256)"),
        ("1e", "MalformedExponentDigits(uint256)"),
        ("1e+", "MalformedExponentDigits(uint256)"),
        (many_digits.as_str(), "ParseDecimalPrecisionLoss(uint256)"),
        ("1e3000000000", "ExponentOverflow(int256,int256)"),
        ("1x", "ParseDecimalFloatExcessCharacters()"),
        ("1.5 ", "ParseDecimalFloatExcessCharacters()"),
    ];
    for (s, want) in cases {
        run(check_parse(s, ParseRef::Selector(want)));
    }
}

#[test]
fn format_hand_values() {
    let cases = [
        (Dec::new(314, -2), "3.14", "3.14"),
        (Dec::new(-12345, -1), "-1.2345e3", "-1234.5"),
        (Dec::new(5, -6), "5e-6", "0.000005"),
        (Dec::new(1200, 0), "1.2e3", "1200"),
        (Dec::new(0, 9), "0", "0"),
    ];
    for (a, sci, plain) in cases {
        let f = float(&a);
        assert_eq!(f.format_with_scientific(true).unwrap(), sci);
        assert_eq!(f.format_with_scientific(false).unwrap(), plain);
    }
}

#[test]
fn format_plain_reverts_past_its_exponent_limit() {
    let revert = |a: Dec| run(check(&show(&a), float(&a).format_with_scientific(false), Err(RefError::UnformatableExponent)));
    revert(Dec::new(1, 1001));
    revert(Dec::new(1, -1001));
    revert(Dec::new(10, 1000));
    assert_eq!(float(&Dec::new(10, -1001)).format_with_scientific(false).unwrap(), format!("0.{}1", "0".repeat(999)));
}
