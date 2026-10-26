mod reference;

use core::cmp::Ordering;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use reference::{self as r, float, run, show, Dec};

fn check_compare(a: &Dec, b: &Dec) -> Result<(), TestCaseError> {
    let (fa, fb) = (float(a), float(b));
    let case = format!("({}, {})", show(a), show(b));
    let want = a.cmp_value(b);
    prop_assert_eq!(fa.lt(fb).unwrap(), want == Ordering::Less, "lt{}", case);
    prop_assert_eq!(fa.gt(fb).unwrap(), want == Ordering::Greater, "gt{}", case);
    prop_assert_eq!(fa.eq(fb).unwrap(), want == Ordering::Equal, "eq{}", case);
    prop_assert_eq!(fa.lte(fb).unwrap(), want != Ordering::Greater, "lte{}", case);
    prop_assert_eq!(fa.gte(fb).unwrap(), want != Ordering::Less, "gte{}", case);
    let min = if want == Ordering::Less { fa } else { fb };
    let max = if want == Ordering::Greater { fa } else { fb };
    prop_assert_eq!(fa.min(fb).unwrap().get_inner(), min.get_inner(), "min{}", case);
    prop_assert_eq!(fa.max(fb).unwrap().get_inner(), max.get_inner(), "max{}", case);
    prop_assert_eq!(fa.is_zero().unwrap(), a.is_zero(), "is_zero({})", show(a));
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn comparisons_match_reference((a, b) in r::pair()) {
        check_compare(&a, &b)?;
    }
}

#[test]
fn comparisons_on_each_ordering() {
    let cases = [
        (Dec::new(1, 0), Dec::new(2, 0)),
        (Dec::new(2, 0), Dec::new(1, 0)),
        (Dec::new(1, 0), Dec::new(10, -1)),
        (Dec::new(-5, 0), Dec::new(5, -1)),
        (Dec::new(0, 0), Dec::new(0, 7)),
        (Dec::new(0, 0), Dec::new(-1, -9)),
        (r::max_positive(), r::min_negative()),
        (r::min_positive(), r::max_negative()),
    ];
    for (a, b) in cases {
        run(check_compare(&a, &b));
        run(check_compare(&b, &a));
    }
}

#[test]
fn min_max_keep_the_operand_representation() {
    let ten = float(&Dec::new(10, 0));
    let also_ten = float(&Dec::new(1, 1));
    assert_ne!(ten.get_inner(), also_ten.get_inner());
    assert_eq!(ten.min(also_ten).unwrap().get_inner(), also_ten.get_inner());
    assert_eq!(ten.max(also_ten).unwrap().get_inner(), also_ten.get_inner());
}
