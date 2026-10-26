mod reference;

use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use reference::{self as r, check_float, float, run, show, Dec, RefError};

fn check_binary(a: &Dec, b: &Dec) -> Result<(), TestCaseError> {
    let (fa, fb) = (float(a), float(b));
    let case = |op: &str| format!("{op}({}, {})", show(a), show(b));
    check_float(&case("add"), fa + fb, r::add(a, b))?;
    check_float(&case("sub"), fa - fb, r::sub(a, b))?;
    check_float(&case("mul"), fa * fb, r::mul(a, b))?;
    check_float(&case("div"), fa / fb, r::div(a, b))
}

fn check_unary(a: &Dec) -> Result<(), TestCaseError> {
    let f = float(a);
    let case = |op: &str| format!("{op}({})", show(a));
    check_float(&case("neg"), -f, r::minus(a))?;
    check_float(&case("abs"), f.abs(), r::abs(a))?;
    check_float(&case("inv"), f.inv(), r::inv(a))?;
    check_float(&case("integer"), f.integer(), r::integer(a))?;
    check_float(&case("frac"), f.frac(), r::frac(a))?;
    check_float(&case("floor"), f.floor(), r::floor(a))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn binary_ops_match_reference((a, b) in r::pair()) {
        check_binary(&a, &b)?;
    }

    #[test]
    fn unary_ops_match_reference(a in r::float()) {
        check_unary(&a)?;
    }
}

#[test]
fn binary_ops_distinguish_operations() {
    let cases = [
        ((7, 0), (2, 0)),
        ((-15, -1), (4, 0)),
        ((1, 0), (3, 0)),
        ((123456789, -4), (-987, 2)),
        ((5, -3), (0, 0)),
        ((0, 0), (-6, 1)),
    ];
    for ((ac, ae), (bc, be)) in cases {
        run(check_binary(&Dec::new(ac, ae), &Dec::new(bc, be)));
    }
}

#[test]
fn binary_ops_hand_values() {
    let p = |s: &str| rain_math_float::Float::parse(s.to_string()).unwrap();
    let fmt = |f: Result<rain_math_float::Float, _>| f.unwrap().format_with_scientific(false).unwrap();
    assert_eq!(fmt(p("3.14") + p("-1.04")), "2.1");
    assert_eq!(fmt(p("3.14") - p("-1.04")), "4.18");
    assert_eq!(fmt(p("3.14") * p("-2")), "-6.28");
    assert_eq!(fmt(p("1") / p("8")), "0.125");
    assert_eq!(fmt(p("-7.5").floor()), "-8");
    assert_eq!(fmt(p("-7.5").integer()), "-7");
    assert_eq!(fmt(p("-7.25").frac()), "-0.25");
    assert_eq!(fmt(p("-0.04").abs()), "0.04");
    assert_eq!(fmt(p("0.04").inv()), "25");
    assert_eq!(fmt(-p("12.5")), "-12.5");
}

#[test]
fn unary_ops_distinguish_operations() {
    for (c, e) in [(-75, -1), (75, -1), (-3, 0), (12345, -2), (-1, -5), (0, 0), (9, 3)] {
        run(check_unary(&Dec::new(c, e)));
    }
}

#[test]
fn arithmetic_reverts() {
    let max = r::max_positive();
    let min_pos = r::min_positive();
    let zero = Dec::zero();
    let one = Dec::new(1, 0);
    let ten = Dec::new(10, 0);
    let tenth = Dec::new(1, -1);
    let (fmax, fmin_pos, fzero, fone, ften, ftenth) =
        (float(&max), float(&min_pos), float(&zero), float(&one), float(&ten), float(&tenth));
    let revert = |got, want: RefError| run(check_float("revert", got, Err(want)));
    revert(fmax + fmax, RefError::ExponentOverflow);
    revert(fmax * ften, RefError::ExponentOverflow);
    revert(fmin_pos * ftenth, RefError::ExponentUnderflow);
    revert(fmin_pos / ften, RefError::ExponentUnderflow);
    revert(fone / fzero, RefError::DivisionByZero);
    revert(fzero.inv(), RefError::DivisionByZero);
    revert(fmin_pos.inv(), RefError::ExponentOverflow);
}
