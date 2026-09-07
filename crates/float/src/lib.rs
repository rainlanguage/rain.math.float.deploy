use alloy::hex::FromHex;
use alloy::primitives::{Bytes, B256};
use alloy::{sol, sol_types::SolCall};
use revm::primitives::{fixed_bytes, U256};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Sub};
use wasm_bindgen_utils::prelude::*;

#[cfg(any(test, feature = "test-harness"))]
use alloy::primitives::aliases::I224;

pub mod error;
mod evm;
pub mod js_api;
#[cfg(any(test, feature = "test-harness"))]
pub mod tables;

use error::DecimalFloatErrorSelector;
pub use error::FloatError;
use evm::execute_call;
#[cfg(any(test, feature = "test-harness"))]
use evm::execute_test_call;

sol!(
    #![sol(all_derives)]
    DecimalFloat,
    "abi/DecimalFloat.json"
);

#[cfg(any(test, feature = "test-harness"))]
sol!(
    #![sol(all_derives)]
    TestDecimalFloat,
    "abi/TestDecimalFloat.json"
);

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, Hash)]
#[wasm_bindgen]
pub struct Float(B256);

impl Float {
    /// Creates a new `Float` from the given 32-byte value `B256`.
    pub const fn from_raw(value: B256) -> Self {
        Float(value)
    }

    /// Getter for inner 32-bytes value of this Float instance as `B256`.
    pub fn get_inner(&self) -> B256 {
        self.0
    }

    /// Sets the inner 32-byte value of this float from the given `B256`.
    pub fn set_inner(&mut self, value: B256) {
        self.0 = value;
    }

    /// Converts a fixed-point decimal value to a `Float` using the specified number of decimals.
    ///
    /// # Arguments
    ///
    /// * `value` - The fixed-point decimal value as a `U256`.
    /// * `decimals` - The number of decimals in the fixed-point representation.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The resulting `Float` value.
    /// * `Err(FloatError)` - If the conversion fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    /// use alloy::primitives::U256;
    ///
    /// // 123.45 with 2 decimals is represented as 12345
    /// let value = U256::from(12345u64);
    /// let decimals = 2u8;
    /// let float = Float::from_fixed_decimal(value, decimals)?;
    /// assert_eq!(float.format()?, "123.45");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn from_fixed_decimal(value: U256, decimals: u8) -> Result<Self, FloatError> {
        let calldata = DecimalFloat::fromFixedDecimalLosslessCall { value, decimals }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded =
                DecimalFloat::fromFixedDecimalLosslessCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Converts a `Float` to a fixed-point decimal value using the specified number of decimals.
    ///
    /// # Arguments
    ///
    /// * `decimals` - The number of decimals in the fixed-point representation.
    ///
    /// # Returns
    ///
    /// * `Ok(U256)` - The resulting fixed-point decimal value.
    /// * `Err(FloatError)` - If the conversion fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    /// use alloy::primitives::U256;
    ///
    /// // 123.45 with 2 decimals becomes 12345
    /// let float = Float::parse("123.45".to_string())?;
    /// let fixed = float.to_fixed_decimal(2)?;
    /// assert_eq!(fixed, U256::from(12345u64));
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn to_fixed_decimal(self, decimals: u8) -> Result<U256, FloatError> {
        let Float(float) = self;
        let calldata = DecimalFloat::toFixedDecimalLosslessCall { float, decimals }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded =
                DecimalFloat::toFixedDecimalLosslessCall::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }

    /// Converts a fixed-point decimal value to a `Float` using the specified number of decimals lossy.
    ///
    /// # Arguments
    ///
    /// * `value` - The fixed-point decimal value as a `U256`.
    /// * `decimals` - The number of decimals in the fixed-point representation.
    ///
    /// # Returns
    ///
    /// * `Ok((Float, bool))` - The resulting `Float` value and a boolean indicating if the conversion was lossless.
    /// * `Err(FloatError)` - If the conversion fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    /// use alloy::primitives::U256;
    ///
    /// // 123.45 with 2 decimals is represented as 12345
    /// let value = U256::from(12345u64);
    /// let decimals = 2u8;
    /// let (float, lossless) = Float::from_fixed_decimal_lossy(value, decimals)?;
    /// assert_eq!(float.format()?, "123.45");
    /// assert!(lossless);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn from_fixed_decimal_lossy(value: U256, decimals: u8) -> Result<(Self, bool), FloatError> {
        let calldata = DecimalFloat::fromFixedDecimalLossyCall { value, decimals }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded =
                DecimalFloat::fromFixedDecimalLossyCall::abi_decode_returns(output.as_ref())?;
            Ok((Float(decoded._0), decoded._1))
        })
    }

    /// Converts a `Float` to a fixed-point decimal value using the specified number of decimals lossy.
    ///
    /// # Arguments
    ///
    /// * `decimals` - The number of decimals in the fixed-point representation.
    ///
    /// # Returns
    ///
    /// * `Ok((U256, bool))` - The resulting fixed-point decimal value and a boolean indicating if the conversion was lossless.
    /// * `Err(FloatError)` - If the conversion fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    /// use alloy::primitives::U256;
    ///
    /// // 123.45 with 2 decimals becomes 12345
    /// let float = Float::from_fixed_decimal(U256::from(12345), 3)?;
    /// let (fixed, lossless) = float.to_fixed_decimal_lossy(2)?;
    /// assert_eq!(fixed, U256::from(1234u64));
    /// assert!(!lossless);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn to_fixed_decimal_lossy(self, decimals: u8) -> Result<(U256, bool), FloatError> {
        let Float(float) = self;
        let calldata = DecimalFloat::toFixedDecimalLossyCall { float, decimals }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded =
                DecimalFloat::toFixedDecimalLossyCall::abi_decode_returns(output.as_ref())?;
            Ok((decoded._0, decoded._1))
        })
    }

    /// Packs a coefficient and exponent into a `Float` in a lossless manner.
    ///
    /// # Arguments
    ///
    /// * `coefficient` - The coefficient as an `I224`.
    /// * `exponent` - The exponent as an `i32`.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The packed float.
    /// * `Err(FloatError)` - If the packing fails (e.g., overflow).
    ///
    /// # Example
    ///
    /// ```
    /// use std::str::FromStr;
    /// use alloy::primitives::aliases::I224;
    /// use rain_math_float::{Float, FloatError};
    ///
    /// let coefficient = I224::from_str("314")?;
    /// let exponent = -2;
    /// let float = Float::pack_lossless(coefficient, exponent)?;
    /// assert_eq!(float.format()?, "3.14");
    ///
    /// anyhow::Ok(())
    /// ```
    #[cfg(any(test, feature = "test-harness"))]
    pub fn pack_lossless(coefficient: I224, exponent: i32) -> Result<Self, FloatError> {
        let calldata = TestDecimalFloat::packLosslessCall {
            coefficient,
            exponent,
        }
        .abi_encode();

        execute_test_call(Bytes::from(calldata), |output| {
            let decoded = TestDecimalFloat::packLosslessCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    #[cfg(any(test, feature = "test-harness"))]
    pub fn unpack(self) -> Result<(alloy::primitives::I256, alloy::primitives::I256), FloatError> {
        let Float(float) = self;
        let calldata = TestDecimalFloat::unpackCall { float }.abi_encode();

        execute_test_call(Bytes::from(calldata), |output| {
            let TestDecimalFloat::unpackReturn {
                _0: coefficient,
                _1: exponent,
            } = TestDecimalFloat::unpackCall::abi_decode_returns(output.as_ref())?;

            Ok((coefficient, exponent))
        })
    }

    #[cfg(any(test, feature = "test-harness"))]
    pub fn show_unpacked(self) -> Result<String, FloatError> {
        let (coefficient, exponent) = self.unpack()?;
        Ok(format!("{coefficient}e{exponent}"))
    }

    /// Parses a decimal string into a `Float`.
    ///
    /// # Arguments
    ///
    /// * `str` - The string to parse.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The parsed float.
    /// * `Err(FloatError)` - If parsing fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let float = Float::parse("3.1415".to_string())?;
    /// assert_eq!(float.format()?, "3.1415");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn parse(str: String) -> Result<Self, FloatError> {
        let calldata = DecimalFloat::parseCall { str }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let DecimalFloat::parseReturn {
                _0: error_selector,
                _1: parsed_float,
            } = DecimalFloat::parseCall::abi_decode_returns(output.as_ref())?;

            if error_selector != fixed_bytes!("00000000") {
                let selector = DecimalFloatErrorSelector::try_from(error_selector);
                return Err(FloatError::DecimalFloatSelector(selector));
            }

            Ok(Float(parsed_float))
        })
    }

    /// Returns the 32-byte hexadecimal string representation of the float.
    ///
    /// # Returns
    ///
    /// * `String` - The 32-byte hex string.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    /// let float = Float::from_hex("0x0000000000000000000000000000000000000000000000000000000000000005").unwrap();
    /// assert_eq!(float.as_hex(), "0x0000000000000000000000000000000000000000000000000000000000000005");
    /// ```
    pub fn as_hex(self) -> String {
        alloy::hex::encode_prefixed(self.0)
    }

    /// Constructs a `Float` from a 32-byte hexadecimal string.
    ///
    /// # Arguments
    ///
    /// * `hex` - The 32-byte hex string to parse.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The float parsed from the hex string.
    /// * `Err(FloatError)` - If the hex string is not valid or not 32 bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    /// let float = Float::from_hex("0x0000000000000000000000000000000000000000000000000000000000000005")?;
    /// assert_eq!(float.as_hex(), "0x0000000000000000000000000000000000000000000000000000000000000005");
    /// anyhow::Ok(())
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, FloatError> {
        let bytes = B256::from_hex(hex).map_err(|_| FloatError::InvalidHex(hex.to_string()))?;
        Ok(Float(bytes))
    }

    /// Returns the maximum positive value that can be represented as a `Float`.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The maximum positive value.
    /// * `Err(FloatError)` - If the EVM call fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let max_pos = Float::max_positive_value()?;
    /// let zero = Float::parse("0".to_string())?;
    ///
    /// // Max positive is greater than zero
    /// assert!(max_pos.gt(zero)?);
    ///
    /// // Max positive is greater than any normal large number
    /// let big_number = Float::parse("999999999999999999999".to_string())?;
    /// assert!(max_pos.gt(big_number)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn max_positive_value() -> Result<Self, FloatError> {
        let calldata = DecimalFloat::maxPositiveValueCall {}.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::maxPositiveValueCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the minimum positive value that can be represented as a `Float`.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The minimum positive value.
    /// * `Err(FloatError)` - If the EVM call fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let min_pos = Float::min_positive_value()?;
    /// let zero = Float::parse("0".to_string())?;
    ///
    /// // Min positive is greater than zero but smaller than any other positive number
    /// assert!(min_pos.gt(zero)?);
    ///
    /// let small_number = Float::parse("0.000000000000000001".to_string())?;
    /// assert!(min_pos.lt(small_number)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn min_positive_value() -> Result<Self, FloatError> {
        let calldata = DecimalFloat::minPositiveValueCall {}.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::minPositiveValueCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the maximum negative value that can be represented as a `Float`.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The maximum negative value (closest to zero).
    /// * `Err(FloatError)` - If the EVM call fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let max_neg = Float::max_negative_value()?;
    /// let zero = Float::parse("0".to_string())?;
    ///
    /// // Max negative is less than zero but greater than any other negative number
    /// assert!(max_neg.lt(zero)?);
    ///
    /// let small_negative = Float::parse("-0.000000000000000001".to_string())?;
    /// assert!(max_neg.gt(small_negative)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn max_negative_value() -> Result<Self, FloatError> {
        let calldata = DecimalFloat::maxNegativeValueCall {}.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::maxNegativeValueCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the minimum negative value that can be represented as a `Float`.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The minimum negative value (furthest from zero).
    /// * `Err(FloatError)` - If the EVM call fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let min_neg = Float::min_negative_value()?;
    /// let zero = Float::parse("0".to_string())?;
    ///
    /// // Min negative is less than zero
    /// assert!(min_neg.lt(zero)?);
    ///
    /// // Min negative is less than any normal negative number
    /// let big_negative = Float::parse("-999999999999999999999".to_string())?;
    /// assert!(min_neg.lt(big_negative)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn min_negative_value() -> Result<Self, FloatError> {
        let calldata = DecimalFloat::minNegativeValueCall {}.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::minNegativeValueCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the zero value of a `Float` in its maximized representation.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The zero value.
    /// * `Err(FloatError)` - If the EVM call fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let zero = Float::zero()?;
    /// assert!(zero.is_zero()?);
    /// assert_eq!(zero.format()?, "0");
    ///
    /// // Should be equal to parsed zero
    /// let parsed_zero = Float::parse("0".to_string())?;
    /// assert!(zero.eq(parsed_zero)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn zero() -> Result<Self, FloatError> {
        let calldata = DecimalFloat::zeroCall {}.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::zeroCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the default minimum value for scientific notation formatting (1e-4).
    ///
    /// Values smaller than this (in absolute value) will be formatted in scientific notation.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The default minimum (1e-4).
    /// * `Err(FloatError)` - If the EVM call fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let min = Float::format_default_scientific_min()?;
    /// assert_eq!(min.format()?, "0.0001");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn format_default_scientific_min() -> Result<Self, FloatError> {
        let calldata = DecimalFloat::FORMAT_DEFAULT_SCIENTIFIC_MINCall {}.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::FORMAT_DEFAULT_SCIENTIFIC_MINCall::abi_decode_returns(
                output.as_ref(),
            )?;
            Ok(Float(decoded))
        })
    }

    /// Returns the default maximum value for scientific notation formatting (1e9).
    ///
    /// Values larger than this (in absolute value) will be formatted in scientific notation.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The default maximum (1e9).
    /// * `Err(FloatError)` - If the EVM call fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let max = Float::format_default_scientific_max()?;
    /// assert_eq!(max.format()?, "1000000000");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn format_default_scientific_max() -> Result<Self, FloatError> {
        let calldata = DecimalFloat::FORMAT_DEFAULT_SCIENTIFIC_MAXCall {}.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::FORMAT_DEFAULT_SCIENTIFIC_MAXCall::abi_decode_returns(
                output.as_ref(),
            )?;
            Ok(Float(decoded))
        })
    }

    /// Formats the float as a decimal string using default scientific notation range (1e-4 to 1e9).
    ///
    /// Values within the range [1e-4, 1e9] will use decimal notation.
    /// Values outside this range will use scientific notation.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The formatted string.
    /// * `Err(FloatError)` - If formatting fails.
    ///
    /// # Examples
    ///
    /// Values within the default range use decimal notation:
    /// ```
    /// use rain_math_float::Float;
    ///
    /// // At the boundaries (inclusive)
    /// assert_eq!(Float::parse("0.0001".to_string())?.format()?, "0.0001");  // 1e-4
    /// assert_eq!(Float::parse("1000000000".to_string())?.format()?, "1000000000");  // 1e9
    ///
    /// // Within range
    /// assert_eq!(Float::parse("2.5".to_string())?.format()?, "2.5");
    /// assert_eq!(Float::parse("123.456".to_string())?.format()?, "123.456");
    /// assert_eq!(Float::parse("0.001".to_string())?.format()?, "0.001");
    /// assert_eq!(Float::parse("1000000".to_string())?.format()?, "1000000");
    ///
    /// anyhow::Ok(())
    /// ```
    ///
    /// Values outside the default range use scientific notation:
    /// ```
    /// use rain_math_float::Float;
    ///
    /// // Smaller than 1e-4
    /// assert_eq!(Float::parse("0.00001".to_string())?.format()?, "1e-5");
    /// assert_eq!(Float::parse("0.000001".to_string())?.format()?, "1e-6");
    ///
    /// // Larger than 1e9
    /// assert_eq!(Float::parse("10000000000".to_string())?.format()?, "1e10");
    /// assert_eq!(Float::parse("123000000000".to_string())?.format()?, "1.23e11");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn format(self) -> Result<String, FloatError> {
        let Float(a) = self;
        let calldata = DecimalFloat::format_1Call { a }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::format_1Call::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }

    /// Formats the float as a decimal string with explicit scientific notation control.
    ///
    /// # Arguments
    ///
    /// * `scientific` - If true, always use scientific notation. If false, use decimal notation.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The formatted string.
    /// * `Err(FloatError)` - If formatting fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let float = Float::parse("3.14".to_string())?;
    /// assert_eq!(float.format_with_scientific(false)?, "3.14");
    /// assert_eq!(float.format_with_scientific(true)?, "3.14");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn format_with_scientific(self, scientific: bool) -> Result<String, FloatError> {
        let Float(a) = self;
        let calldata = DecimalFloat::format_0Call { a, scientific }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::format_0Call::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }

    /// Formats the float as a decimal string with a custom scientific notation range.
    ///
    /// # Arguments
    ///
    /// * `scientific_min` - Values smaller than this (in absolute value) use scientific notation.
    /// * `scientific_max` - Values larger than this (in absolute value) use scientific notation.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The formatted string.
    /// * `Err(FloatError)` - If formatting fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let float = Float::parse("0.001".to_string())?;
    /// let min = Float::parse("0.01".to_string())?;
    /// let max = Float::parse("100".to_string())?;
    /// assert_eq!(float.format_with_range(min, max)?, "1e-3");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn format_with_range(
        self,
        scientific_min: Self,
        scientific_max: Self,
    ) -> Result<String, FloatError> {
        let Float(a) = self;
        let Float(scientific_min_inner) = scientific_min;
        let Float(scientific_max_inner) = scientific_max;
        let calldata = DecimalFloat::format_2Call {
            a,
            scientificMin: scientific_min_inner,
            scientificMax: scientific_max_inner,
        }
        .abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::format_2Call::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }

    /// Returns `true` if `self` is less than `b`.
    ///
    /// # Arguments
    ///
    /// * `b` - The `Float` value to compare with `self`.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if `self` is less than `b`.
    /// * `Ok(false)` if `self` is not less than `b`.
    /// * `Err(FloatError)` if the comparison fails due to an error in the underlying EVM call or decoding.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("1.0".to_string())?;
    /// let b = Float::parse("2.0".to_string())?;
    /// assert!(a.lt(b)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn lt(self, b: Self) -> Result<bool, FloatError> {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::ltCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::ltCall::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }

    /// Returns `true` if `self` is equal to `b`.
    ///
    /// # Arguments
    ///
    /// * `b` - The `Float` value to compare with `self`.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if `self` is equal to `b`.
    /// * `Ok(false)` if `self` is not equal to `b`.
    /// * `Err(FloatError)` if the comparison fails due to an error in the underlying EVM call or decoding.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("3.14".to_string())?;
    /// let b = Float::parse("3.14".to_string())?;
    /// assert!(a.eq(b)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn eq(self, b: Self) -> Result<bool, FloatError> {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::eqCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::eqCall::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }

    /// Returns `true` if `self` is greater than `b`.
    ///
    /// # Arguments
    ///
    /// * `b` - The `Float` value to compare with `self`.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if `self` is greater than `b`.
    /// * `Ok(false)` if `self` is not greater than `b`.
    /// * `Err(FloatError)` if the comparison fails due to an error in the underlying EVM call or decoding.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("5.0".to_string())?;
    /// let b = Float::parse("2.0".to_string())?;
    /// assert!(a.gt(b)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn gt(self, b: Self) -> Result<bool, FloatError> {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::gtCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::gtCall::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }

    /// Returns the multiplicative inverse of the float.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The inverse.
    /// * `Err(FloatError)` - If inversion fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let x = Float::parse("2.0".to_string())?;
    /// let inv = x.inv()?;
    /// assert!(inv.format()?.starts_with("0.5"));
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn inv(self) -> Result<Self, FloatError> {
        let Float(a) = self;
        let calldata = DecimalFloat::invCall { a }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::invCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the absolute value of the float.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The absolute value.
    /// * `Err(FloatError)` - If the operation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let x = Float::parse("-3.14".to_string())?;
    /// let abs = x.abs()?;
    /// assert_eq!(abs.format()?, "3.14");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn abs(self) -> Result<Float, FloatError> {
        let Float(a) = self;
        let calldata = DecimalFloat::absCall { a }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::absCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns `true` if `self` is less than or equal to `b`.
    ///
    /// # Arguments
    ///
    /// * `b` - The `Float` value to compare with `self`.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if `self` is less than or equal to `b`.
    /// * `Ok(false)` if `self` is not less than or equal to `b`.
    /// * `Err(FloatError)` if the comparison fails due to an error in the underlying EVM call or decoding.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("1.0".to_string())?;
    /// let b = Float::parse("2.0".to_string())?;
    /// assert!(a.lte(b)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn lte(self, b: Self) -> Result<bool, FloatError> {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::lteCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::lteCall::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }

    /// Returns `true` if `self` is greater than or equal to `b`.
    ///
    /// # Arguments
    ///
    /// * `b` - The `Float` value to compare with `self`.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if `self` is greater than or equal to `b`.
    /// * `Ok(false)` if `self` is not greater than or equal to `b`.
    /// * `Err(FloatError)` if the comparison fails due to an error in the underlying EVM call or decoding.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("2.0".to_string())?;
    /// let b = Float::parse("1.0".to_string())?;
    /// assert!(a.gte(b)?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn gte(self, b: Self) -> Result<bool, FloatError> {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::gteCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::gteCall::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }
}

impl Add for Float {
    type Output = Result<Self, FloatError>;

    /// Adds two floats.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The sum.
    /// * `Err(FloatError)` - If addition fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("1.5".to_string())?;
    /// let b = Float::parse("2.5".to_string())?;
    /// let sum = (a + b)?;
    /// assert_eq!(sum.format()?, "4");
    ///
    /// anyhow::Ok(())
    /// ```
    fn add(self, b: Self) -> Self::Output {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::addCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::addCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }
}

impl Sub for Float {
    type Output = Result<Self, FloatError>;

    /// Subtracts `b` from `self`.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The difference.
    /// * `Err(FloatError)` - If subtraction fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("5.0".to_string())?;
    /// let b = Float::parse("2.0".to_string())?;
    /// let diff = (a - b)?;
    /// assert_eq!(diff.format()?, "3");
    ///
    /// anyhow::Ok(())
    /// ```
    fn sub(self, b: Self) -> Self::Output {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::subCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::subCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }
}

impl Mul for Float {
    type Output = Result<Self, FloatError>;

    /// Multiplies two floats.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The product.
    /// * `Err(FloatError)` - If multiplication fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("2.0".to_string())?;
    /// let b = Float::parse("3.0".to_string())?;
    /// let product = (a * b)?;
    /// assert_eq!(product.format()?, "6");
    ///
    /// anyhow::Ok(())
    /// ```
    fn mul(self, b: Self) -> Self::Output {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::mulCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::mulCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }
}

impl Div for Float {
    type Output = Result<Self, FloatError>;

    /// Divides `self` by `b`.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The quotient.
    /// * `Err(FloatError)` - If division fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("6.0".to_string())?;
    /// let b = Float::parse("2.0".to_string())?;
    /// let quotient = (a / b)?;
    /// assert_eq!(quotient.format()?, "3");
    ///
    /// anyhow::Ok(())
    /// ```
    fn div(self, b: Self) -> Self::Output {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::divCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::divCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }
}

impl Float {
    /// Returns the integer part of the float (truncation toward zero).
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The integer part.
    /// * `Err(FloatError)` - If the operation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let x = Float::parse("3.75".to_string())?;
    /// let int = x.integer()?;
    /// assert_eq!(int.format()?, "3");
    ///
    /// let y = Float::parse("-3.75".to_string())?;
    /// let int_y = y.integer()?;
    /// assert_eq!(int_y.format()?, "-3");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn integer(self) -> Result<Float, FloatError> {
        let Float(a) = self;
        let calldata = DecimalFloat::integerCall { a }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::integerCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the fractional part of the float.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The fractional part.
    /// * `Err(FloatError)` - If the operation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let x = Float::parse("3.75".to_string())?;
    /// let frac = x.frac()?;
    /// assert_eq!(frac.format()?, "0.75");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn frac(self) -> Result<Float, FloatError> {
        let Float(a) = self;
        let calldata = DecimalFloat::fracCall { a }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::fracCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the floor of the float.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The floored value.
    /// * `Err(FloatError)` - If the operation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let x = Float::parse("3.75".to_string())?;
    /// let floor = x.floor()?;
    /// assert_eq!(floor.format()?, "3");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn floor(self) -> Result<Float, FloatError> {
        let Float(a) = self;
        let calldata = DecimalFloat::floorCall { a }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::floorCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the minimum of `self` and `b`.
    ///
    /// # Arguments
    ///
    /// * `b` - The other `Float` to compare with.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The minimum value.
    /// * `Err(FloatError)` - If the operation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("1.0".to_string())?;
    /// let b = Float::parse("2.0".to_string())?;
    /// let min = a.min(b)?;
    /// assert_eq!(min.format()?, "1");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn min(self, b: Self) -> Result<Self, FloatError> {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::minCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::minCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Returns the maximum of `self` and `b`.
    ///
    /// # Arguments
    ///
    /// * `b` - The other `Float` to compare with.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The maximum value.
    /// * `Err(FloatError)` - If the operation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let a = Float::parse("1.0".to_string())?;
    /// let b = Float::parse("2.0".to_string())?;
    /// let max = a.max(b)?;
    /// assert_eq!(max.format()?, "2");
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn max(self, b: Self) -> Result<Self, FloatError> {
        let Float(a) = self;
        let Float(b) = b;
        let calldata = DecimalFloat::maxCall { a, b }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::maxCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }

    /// Checks if the float is zero.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the float is zero.
    /// * `Ok(false)` if the float is not zero.
    /// * `Err(FloatError)` if the operation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let zero = Float::parse("0".to_string())?;
    /// assert!(zero.is_zero()?);
    /// let nonzero = Float::parse("1.23".to_string())?;
    /// assert!(!nonzero.is_zero()?);
    ///
    /// anyhow::Ok(())
    /// ```
    pub fn is_zero(self) -> Result<bool, FloatError> {
        let Float(a) = self;
        let calldata = DecimalFloat::isZeroCall { a }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::isZeroCall::abi_decode_returns(output.as_ref())?;
            Ok(decoded)
        })
    }
}

impl Neg for Float {
    type Output = Result<Self, FloatError>;

    /// Returns the negation of the float.
    ///
    /// # Returns
    ///
    /// * `Ok(Float)` - The negated value.
    /// * `Err(FloatError)` - If the operation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use rain_math_float::Float;
    ///
    /// let x = Float::parse("3.14".to_string())?;
    /// let neg = (-x)?;
    /// assert_eq!(neg.format()?, "-3.14");
    ///
    /// anyhow::Ok(())
    /// ```
    fn neg(self) -> Self::Output {
        let Float(a) = self;
        let calldata = DecimalFloat::minusCall { a }.abi_encode();

        execute_call(Bytes::from(calldata), |output| {
            let decoded = DecimalFloat::minusCall::abi_decode_returns(output.as_ref())?;
            Ok(Float(decoded))
        })
    }
}

impl From<B256> for Float {
    fn from(value: B256) -> Self {
        Float(value)
    }
}

impl From<Float> for B256 {
    fn from(value: Float) -> Self {
        value.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    /// Float::default() equals parsed "0".
    #[test]
    fn test_default() {
        let zero = Float::parse("0".to_string()).unwrap();
        assert!(zero.eq(Float::default()).unwrap());
    }

    prop_compose! {
        fn arb_float()(
            coefficient in any::<I224>(),
            exponent in any::<i32>(),
        ) -> Float {
            Float::pack_lossless(coefficient, exponent).unwrap()
        }
    }

    /// JSON serialize then deserialize preserves equality and hex representation.
    #[test]
    fn test_serde() {
        let float = Float::parse("1.1341234234625468391".to_string()).unwrap();
        let serialized = serde_json::to_string(&float).unwrap();
        assert_eq!(
            serialized,
            json!("0xffffffed00000000000000000000000000000000000000009d642872ad59a7e7").to_string()
        );
        let deserialized: Float = serde_json::from_str(&serialized).unwrap();
        assert!(float.eq(deserialized).unwrap());
    }

    proptest! {
        #[test]
        /// JSON round-trip preserves equality and serialized form for all floats.
        fn proptest_serde(float in arb_float()) {
            let serialized = serde_json::to_string(&float).unwrap();
            let deserialized: Float = serde_json::from_str(&serialized).unwrap();
            prop_assert!(float.eq(deserialized).unwrap());
            let re_serialized = serde_json::to_string(&deserialized).unwrap();
            prop_assert_eq!(serialized, re_serialized);
        }
    }

    proptest! {
        #[test]
        /// as_hex() then from_hex() round-trips to identical hex.
        fn test_as_from_hex(float in arb_float()) {
            let hex = float.as_hex();
            let parsed = Float::from_hex(&hex).unwrap();
            prop_assert_eq!(parsed.as_hex(), hex);
        }
    }
}
