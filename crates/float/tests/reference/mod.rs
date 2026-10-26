//! Exact big-integer reference for the concrete's documented semantics, as in
//! rain.math.float's `crates/tests`. A value is `c × 10^e` with an unbounded
//! `c`; every result rounds once, towards zero, to the nearest Float.

#![allow(dead_code)]

use alloy::primitives::{B256, U256};
use core::cmp::Ordering;
use num_bigint::{BigInt, Sign};
use num_traits::{Signed, Zero};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use rain_math_float::{Float, FloatError};
use alloy::sol_types::SolInterface;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefError {
    ExponentOverflow,
    ExponentUnderflow,
    DivisionByZero,
    LossyConversionToFloat,
    LossyConversionFromFloat,
    NegativeFixedDecimalConversion,
    FixedDecimalOverflow,
    UnformatableExponent,
}

pub const I32_MIN: i64 = i32::MIN as i64;
pub const I32_MAX: i64 = i32::MAX as i64;

pub fn pow10(k: u64) -> BigInt {
    num_traits::pow(BigInt::from(10), k as usize)
}

pub fn int224_min() -> BigInt {
    -(BigInt::from(1) << 223usize)
}

pub fn int224_max() -> BigInt {
    (BigInt::from(1) << 223usize) - 1
}

pub fn fits_int224(c: &BigInt) -> bool {
    *c >= int224_min() && *c <= int224_max()
}

pub fn digits(c: &BigInt) -> u64 {
    if c.is_zero() {
        0
    } else {
        c.abs().to_string().len() as u64
    }
}

fn shed(c: &BigInt, k: u64) -> BigInt {
    if k > digits(c) {
        BigInt::zero()
    } else {
        c / pow10(k)
    }
}

#[derive(Debug, Clone)]
pub struct Dec {
    pub c: BigInt,
    pub e: i64,
}

impl Dec {
    pub fn new(c: impl Into<BigInt>, e: i64) -> Self {
        Self { c: c.into(), e }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn is_zero(&self) -> bool {
        self.c.is_zero()
    }

    pub fn is_negative(&self) -> bool {
        self.c.is_negative()
    }

    /// The packed layout: int32 exponent in the high 4 bytes, int224
    /// coefficient in the low 28.
    pub fn from_bytes(f: B256) -> Self {
        let b = f.0;
        let e = i32::from_be_bytes([b[0], b[1], b[2], b[3]]) as i64;
        Self::new(BigInt::from_signed_bytes_be(&b[4..]), e)
    }

    pub fn to_bytes(&self) -> B256 {
        assert!(fits_int224(&self.c), "coefficient {} is not int224", self.c);
        let e = i32::try_from(self.e).expect("exponent is not int32");
        let mut bytes = [0u8; 32];
        bytes[..4].copy_from_slice(&e.to_be_bytes());
        let c = self.c.to_signed_bytes_be();
        let fill = if self.c.is_negative() { 0xff } else { 0 };
        bytes[4..32 - c.len()].fill(fill);
        bytes[32 - c.len()..].copy_from_slice(&c);
        B256::from(bytes)
    }

    pub fn normalized(&self) -> Self {
        if self.c.is_zero() {
            return Self::zero();
        }
        let (mut c, mut e) = (self.c.clone(), self.e);
        while (&c % 10u32).is_zero() {
            c /= 10u32;
            e += 1;
        }
        Self::new(c, e)
    }

    pub fn neg(&self) -> Self {
        Self::new(-&self.c, self.e)
    }

    pub fn abs(&self) -> Self {
        Self::new(self.c.abs(), self.e)
    }

    fn at_exponent(&self, e: i64) -> BigInt {
        assert!(e <= self.e);
        &self.c * pow10((self.e - e) as u64)
    }

    pub fn cmp_value(&self, other: &Self) -> Ordering {
        let (sa, sb) = (self.c.sign(), other.c.sign());
        if sa != sb {
            return rank(sa).cmp(&rank(sb));
        }
        if sa == Sign::NoSign {
            return Ordering::Equal;
        }
        let oa = self.e + digits(&self.c) as i64;
        let ob = other.e + digits(&other.c) as i64;
        let magnitude = if oa != ob {
            oa.cmp(&ob)
        } else {
            let e = self.e.min(other.e);
            self.at_exponent(e)
                .abs()
                .cmp(&other.at_exponent(e).abs())
        };
        if sa == Sign::Minus {
            magnitude.reverse()
        } else {
            magnitude
        }
    }

    pub fn eq_value(&self, other: &Self) -> bool {
        self.cmp_value(other) == Ordering::Equal
    }

    fn add_exact(&self, other: &Self) -> Option<Self> {
        if self.is_zero() {
            return Some(other.clone());
        }
        if other.is_zero() {
            return Some(self.clone());
        }
        if (self.e - other.e).abs() > 400 {
            return None;
        }
        let e = self.e.min(other.e);
        Some(Self::new(self.at_exponent(e) + other.at_exponent(e), e))
    }

    pub fn trunc(&self) -> Self {
        if self.e >= 0 {
            self.clone()
        } else {
            Self::new(shed(&self.c, (-self.e) as u64), 0)
        }
    }
}

fn rank(s: Sign) -> i8 {
    match s {
        Sign::Minus => -1,
        Sign::NoSign => 0,
        Sign::Plus => 1,
    }
}

#[derive(Debug, Clone)]
pub enum Packed {
    Value(Dec, bool),
    Underflow,
    Overflow,
}

fn coefficient_at(x: &Dec, f: i64) -> BigInt {
    let c = if f >= x.e {
        shed(&x.c, (f - x.e) as u64)
    } else {
        &x.c * pow10(((x.e - f) as u64).min(80))
    };
    c.clamp(int224_min(), int224_max())
}

fn nearest_within(x: &Dec, lo: i64, hi: i64) -> Dec {
    let top = x.e + digits(&x.c) as i64;
    let mut best = Dec::zero();
    for f in (top - 70..=top - 66).chain([lo, hi]) {
        let f = f.clamp(lo, hi);
        let candidate = Dec::new(coefficient_at(x, f), f);
        if candidate.abs().cmp_value(&best.abs()) == Ordering::Greater {
            best = candidate;
        }
    }
    best
}

/// The Float closest to `x` not exceeding its magnitude, and whether it is
/// `x`.
pub fn pack(x: &Dec) -> Packed {
    if x.c.is_zero() {
        return Packed::Value(Dec::zero(), true);
    }
    let top = x.e + digits(&x.c) as i64;
    let unbounded = nearest_within(x, top - 70, top - 66);
    if unbounded.cmp_value(&max_positive()) == Ordering::Greater
        || unbounded.cmp_value(&min_negative()) == Ordering::Less
    {
        return Packed::Overflow;
    }
    let best = nearest_within(x, I32_MIN, I32_MAX);
    if best.is_zero() {
        return Packed::Underflow;
    }
    let lossless = best.eq_value(x);
    Packed::Value(best, lossless)
}

pub fn arithmetic(x: &Dec) -> Result<Dec, RefError> {
    match pack(x) {
        Packed::Value(v, _) => Ok(v),
        Packed::Underflow => Err(RefError::ExponentUnderflow),
        Packed::Overflow => Err(RefError::ExponentOverflow),
    }
}

pub fn add(a: &Dec, b: &Dec) -> Result<Dec, RefError> {
    let sum = a.add_exact(b).unwrap_or_else(|| {
        let (big, small) = if a.abs().cmp_value(&b.abs()) == Ordering::Less {
            (b, a)
        } else {
            (a, b)
        };
        let top = big.e + digits(&big.c) as i64;
        let unit = if small.is_negative() { -1 } else { 1 };
        big.add_exact(&Dec::new(unit, top - 200)).unwrap()
    });
    arithmetic(&sum)
}

pub fn sub(a: &Dec, b: &Dec) -> Result<Dec, RefError> {
    add(a, &b.neg())
}

pub fn mul(a: &Dec, b: &Dec) -> Result<Dec, RefError> {
    arithmetic(&Dec::new(&a.c * &b.c, a.e + b.e))
}

pub fn div(a: &Dec, b: &Dec) -> Result<Dec, RefError> {
    if b.is_zero() {
        return Err(RefError::DivisionByZero);
    }
    if a.is_zero() {
        return Ok(Dec::zero());
    }
    const EXTRA: u64 = 160;
    arithmetic(&Dec::new(
        &a.c * pow10(EXTRA) / &b.c,
        a.e - b.e - EXTRA as i64,
    ))
}

pub fn inv(a: &Dec) -> Result<Dec, RefError> {
    div(&Dec::new(1, 0), a)
}

pub fn minus(a: &Dec) -> Result<Dec, RefError> {
    arithmetic(&a.neg())
}

pub fn abs(a: &Dec) -> Result<Dec, RefError> {
    arithmetic(&a.abs())
}

pub fn integer(a: &Dec) -> Result<Dec, RefError> {
    arithmetic(&a.trunc())
}

pub fn frac(a: &Dec) -> Result<Dec, RefError> {
    if a.e >= 0 {
        return Ok(Dec::zero());
    }
    let t = a.trunc();
    if t.is_zero() {
        return arithmetic(a);
    }
    arithmetic(&Dec::new(&a.c - t.at_exponent(a.e), a.e))
}

pub fn floor(a: &Dec) -> Result<Dec, RefError> {
    let t = a.trunc();
    if a.is_negative() && !t.eq_value(a) {
        arithmetic(&Dec::new(t.c - 1, 0))
    } else {
        arithmetic(&t)
    }
}

pub fn max_positive() -> Dec {
    Dec::new(int224_max(), I32_MAX)
}

pub fn min_positive() -> Dec {
    Dec::new(1, I32_MIN)
}

pub fn max_negative() -> Dec {
    Dec::new(-1, I32_MIN)
}

pub fn min_negative() -> Dec {
    Dec::new(int224_min(), I32_MAX)
}

pub fn u256_to_big(v: U256) -> BigInt {
    BigInt::from_bytes_be(Sign::Plus, &v.to_be_bytes::<32>())
}

pub fn big_to_u256(v: &BigInt) -> U256 {
    U256::from_str_radix(&v.to_string(), 10).unwrap()
}

pub fn from_fixed_decimal_lossy(value: U256, decimals: u8) -> (Dec, bool) {
    match pack(&Dec::new(u256_to_big(value), -(decimals as i64))) {
        Packed::Value(v, lossless) => (v, lossless),
        other => unreachable!("{other:?}"),
    }
}

pub fn from_fixed_decimal_lossless(value: U256, decimals: u8) -> Result<Dec, RefError> {
    match from_fixed_decimal_lossy(value, decimals) {
        (v, true) => Ok(v),
        (_, false) => Err(RefError::LossyConversionToFloat),
    }
}

pub fn to_fixed_decimal_lossy(x: &Dec, decimals: u8) -> Result<(U256, bool), RefError> {
    if x.is_negative() {
        return Err(RefError::NegativeFixedDecimalConversion);
    }
    if x.is_zero() {
        return Ok((U256::ZERO, true));
    }
    let scaled = Dec::new(x.c.clone(), x.e + decimals as i64);
    if scaled.e > 80 {
        return Err(RefError::FixedDecimalOverflow);
    }
    let t = scaled.trunc();
    let int = if t.e > 0 { t.at_exponent(0) } else { t.c.clone() };
    if int > u256_to_big(U256::MAX) {
        return Err(RefError::FixedDecimalOverflow);
    }
    Ok((big_to_u256(&int), t.eq_value(&scaled)))
}

pub fn to_fixed_decimal_lossless(x: &Dec, decimals: u8) -> Result<U256, RefError> {
    match to_fixed_decimal_lossy(x, decimals)? {
        (v, true) => Ok(v),
        (_, false) => Err(RefError::LossyConversionFromFloat),
    }
}

/// `toDecimalString(x, true)`: one integral digit, the remaining significant
/// digits after a point, then the exponent unless it is zero.
pub fn format_scientific(x: &Dec) -> String {
    if x.is_zero() {
        return "0".to_string();
    }
    let n = x.normalized();
    let d = n.c.abs().to_string();
    let display = n.e + d.len() as i64 - 1;
    let mut out = String::new();
    if n.is_negative() {
        out.push('-');
    }
    out.push_str(&d[..1]);
    if d.len() > 1 {
        out.push('.');
        out.push_str(&d[1..]);
    }
    if display != 0 {
        out.push_str(&format!("e{display}"));
    }
    out
}

/// `toDecimalString(x, false)`: plain decimal for a normalized
/// `|exponent| <= 1000`.
pub fn format_plain(x: &Dec) -> Result<String, RefError> {
    if x.is_zero() {
        return Ok("0".to_string());
    }
    let n = x.normalized();
    if n.e.abs() > 1000 {
        return Err(RefError::UnformatableExponent);
    }
    let d = n.c.abs().to_string();
    let mut out = String::new();
    if n.is_negative() {
        out.push('-');
    }
    if n.e >= 0 {
        out.push_str(&d);
        out.push_str(&"0".repeat(n.e as usize));
    } else {
        let point = (-n.e) as usize;
        if d.len() > point {
            out.push_str(&d[..d.len() - point]);
            out.push('.');
            out.push_str(&d[d.len() - point..]);
        } else {
            out.push_str("0.");
            out.push_str(&"0".repeat(point - d.len()));
            out.push_str(&d);
        }
    }
    Ok(out)
}

/// Every int224, weighted towards its ends, zero, one, and powers of ten.
pub fn coefficient() -> BoxedStrategy<BigInt> {
    let sign = any::<bool>();
    prop_oneof![
        3 => any::<[u8; 28]>().prop_map(|b| BigInt::from_signed_bytes_be(&b)),
        3 => (1u64..=67, any::<[u8; 32]>(), sign.clone()).prop_map(|(n, b, neg)| {
            let lo = pow10(n - 1);
            let span = pow10(n) - &lo;
            let c = lo + BigInt::from_bytes_be(Sign::Plus, &b) % span;
            if neg { -c } else { c }
        }),
        2 => prop_oneof![
            Just(int224_min()),
            Just(int224_max()),
            Just(BigInt::zero()),
            Just(BigInt::from(1)),
            Just(BigInt::from(-1)),
        ],
        2 => (0u64..=67, -3i64..=3, sign).prop_map(|(k, d, neg)| {
            let c = pow10(k) + d;
            if neg { -c } else { c }
        }),
    ]
    .prop_filter("int224", fits_int224)
    .boxed()
}

pub fn exponent() -> BoxedStrategy<i64> {
    prop_oneof![
        2 => any::<i32>().prop_map(i64::from),
        1 => I32_MIN..=I32_MIN + 160,
        1 => I32_MAX - 160..=I32_MAX,
        4 => -80i64..=80,
    ]
    .boxed()
}

pub fn float() -> BoxedStrategy<Dec> {
    (coefficient(), exponent())
        .prop_map(|(c, e)| Dec::new(c, e))
        .boxed()
}

/// Two operands, often at nearby exponents so their digits interact.
pub fn pair() -> BoxedStrategy<(Dec, Dec)> {
    prop_oneof![
        1 => (float(), float()),
        2 => (float(), coefficient(), -80i64..=80).prop_map(|(a, c, off)| {
            let e = (a.e + off).clamp(I32_MIN, I32_MAX);
            (a, Dec::new(c, e))
        }),
    ]
    .boxed()
}

pub fn show(a: &Dec) -> String {
    format!("{}e{}", a.c, a.e)
}

impl RefError {
    pub fn signature(self) -> &'static str {
        match self {
            Self::ExponentOverflow => "ExponentOverflow(int256,int256)",
            Self::ExponentUnderflow => "ExponentUnderflow(int256,int256)",
            Self::DivisionByZero => "DivisionByZero(int256,int256)",
            Self::LossyConversionToFloat => "LossyConversionToFloat(int256,int256)",
            Self::LossyConversionFromFloat => "LossyConversionFromFloat(int256,int256)",
            Self::NegativeFixedDecimalConversion => "NegativeFixedDecimalConversion(int256,int256)",
            Self::FixedDecimalOverflow => "FixedDecimalOverflow(int256,int256,uint8)",
            Self::UnformatableExponent => "UnformatableExponent(int256)",
        }
    }

    pub fn selector(self) -> [u8; 4] {
        selector(self.signature())
    }
}

pub fn selector(signature: &str) -> [u8; 4] {
    alloy::primitives::keccak256(signature)[..4].try_into().unwrap()
}

/// The revert a crate call returned is exactly `want`, decoded as the
/// concrete's error.
pub fn is_revert(got: &FloatError, want: RefError) -> bool {
    match got {
        FloatError::DecimalFloat(e) => e.selector() == want.selector(),
        _ => false,
    }
}

pub fn float(d: &Dec) -> Float {
    Float::from_raw(d.to_bytes())
}

pub fn dec(f: Float) -> Dec {
    Dec::from_bytes(f.get_inner())
}

pub fn check_float(
    case: &str,
    got: Result<Float, FloatError>,
    want: Result<Dec, RefError>,
) -> Result<(), TestCaseError> {
    match (got, want) {
        (Ok(g), Ok(w)) => {
            let g = dec(g);
            prop_assert!(g.eq_value(&w), "{case}: crate {}, reference {}", show(&g), show(&w));
        }
        (Err(g), Err(w)) => prop_assert!(is_revert(&g, w), "{case}: crate {g:?}, reference {w:?}"),
        (g, w) => prop_assert!(false, "{case}: crate {g:?}, reference {w:?}"),
    }
    Ok(())
}

pub fn check<T: PartialEq + core::fmt::Debug>(
    case: &str,
    got: Result<T, FloatError>,
    want: Result<T, RefError>,
) -> Result<(), TestCaseError> {
    match (got, want) {
        (Ok(g), Ok(w)) => prop_assert_eq!(g, w, "{}", case),
        (Err(g), Err(w)) => prop_assert!(is_revert(&g, w), "{case}: crate {g:?}, reference {w:?}"),
        (g, w) => prop_assert!(false, "{case}: crate {g:?}, reference {w:?}"),
    }
    Ok(())
}

pub fn run(r: Result<(), TestCaseError>) {
    if let Err(e) = r {
        panic!("{e}");
    }
}
