use crate::DecimalFloat;
use alloy::hex::FromHexError;
use alloy::primitives::{Bytes, FixedBytes};
use alloy::sol_types::SolError;
use revm::context::result::{EVMError, HaltReason, Output, SuccessReason};
use std::thread::AccessError;
use thiserror::Error;
use wasm_bindgen_utils::prelude::js_sys::{Error as JsError, RangeError};
use wasm_bindgen_utils::result::WasmEncodedError;

#[derive(Debug, Error)]
pub enum FloatError {
    #[error("EVM error: {0}")]
    Evm(#[from] EVMError<std::convert::Infallible>),
    #[error("Float execution reverted with output: {0}")]
    Revert(Bytes),
    #[error("Float execution halted with reason: {0:?}")]
    Halt(HaltReason),
    #[error("Execution ended for non-return reason. Reason: {0:?}. Output: {1:?}")]
    UnexpectedSuccess(SuccessReason, Output),
    #[error(transparent)]
    AlloySolTypes(#[from] alloy::sol_types::Error),
    #[error("Decimal Float error: {0:?}")]
    DecimalFloat(Box<DecimalFloat::DecimalFloatErrors>),
    #[error("Decimal Float error selector: {0:?}")]
    DecimalFloatSelector(Result<DecimalFloatErrorSelector, FixedBytes<4>>),
    #[error(transparent)]
    Access(#[from] AccessError),
    #[error("Invalid hex string: {0}")]
    InvalidHex(String),
    #[error(transparent)]
    AlloyFromHexError(#[from] FromHexError),
    #[error(transparent)]
    AlloyParseError(#[from] alloy::primitives::ruint::ParseError),
    #[error(transparent)]
    AlloyParseSignedError(#[from] alloy::primitives::ParseSignedError),
    #[error("Wasm bindgen js_sys threw error: {0}")]
    JsSysError(String),
}

mod parse_errors {
    alloy::sol! {
        error ParseEmptyDecimalString(uint256 position);
        error MalformedDecimalPoint(uint256 position);
        error MalformedExponentDigits(uint256 position);
        error ParseDecimalPrecisionLoss(uint256 position);
        error ParseDecimalFloatExcessCharacters();
    }
}

/// The error selectors `DecimalFloat.parse` returns instead of reverting.
#[derive(Debug, PartialEq, Eq)]
pub enum DecimalFloatErrorSelector {
    ExponentOverflow,
    ParseEmptyDecimalString,
    MalformedDecimalPoint,
    MalformedExponentDigits,
    ParseDecimalPrecisionLoss,
    ParseDecimalFloatExcessCharacters,
}

impl DecimalFloatErrorSelector {
    /// A detailed, human-readable description of the parse failure this
    /// selector reports.
    pub fn to_readable_msg(&self) -> &'static str {
        match self {
            Self::ExponentOverflow => {
                "The number is too large to represent as a Float (its exponent exceeds the maximum supported magnitude)."
            }
            Self::ParseEmptyDecimalString => "There are no digits where the number should start.",
            Self::MalformedDecimalPoint => "The decimal point is not followed by any digits.",
            Self::MalformedExponentDigits => "The exponent has no digits after the `e`.",
            Self::ParseDecimalPrecisionLoss => {
                "The number has more precision than a Float can represent exactly."
            }
            Self::ParseDecimalFloatExcessCharacters => {
                "There are unexpected characters after the number."
            }
        }
    }
}

impl TryFrom<FixedBytes<4>> for DecimalFloatErrorSelector {
    type Error = FixedBytes<4>;

    fn try_from(error_selector: FixedBytes<4>) -> Result<Self, Self::Error> {
        let FixedBytes(bytes) = error_selector;
        match bytes {
            <DecimalFloat::ExponentOverflow as SolError>::SELECTOR => Ok(Self::ExponentOverflow),
            <parse_errors::ParseEmptyDecimalString as SolError>::SELECTOR => {
                Ok(Self::ParseEmptyDecimalString)
            }
            <parse_errors::MalformedDecimalPoint as SolError>::SELECTOR => {
                Ok(Self::MalformedDecimalPoint)
            }
            <parse_errors::MalformedExponentDigits as SolError>::SELECTOR => {
                Ok(Self::MalformedExponentDigits)
            }
            <parse_errors::ParseDecimalPrecisionLoss as SolError>::SELECTOR => {
                Ok(Self::ParseDecimalPrecisionLoss)
            }
            <parse_errors::ParseDecimalFloatExcessCharacters as SolError>::SELECTOR => {
                Ok(Self::ParseDecimalFloatExcessCharacters)
            }
            _ => Err(error_selector),
        }
    }
}

impl FloatError {
    /// A detailed, human-readable description of the error, suitable for
    /// surfacing to end users (e.g. via [WasmEncodedError::readable_msg]).
    pub fn to_readable_msg(&self) -> String {
        match self {
            Self::Evm(e) => {
                format!("An error occurred while executing the Float operation in the EVM: {e}")
            }
            Self::Revert(bytes) => {
                format!("The Float operation reverted with output: {bytes}")
            }
            Self::Halt(reason) => {
                format!("The Float operation halted unexpectedly with reason: {reason:?}")
            }
            Self::UnexpectedSuccess(reason, output) => {
                format!(
                    "The Float operation ended for an unexpected non-return reason: {reason:?}. Output: {output:?}"
                )
            }
            Self::AlloySolTypes(e) => {
                format!("Failed to encode or decode the Float operation's ABI data: {e}")
            }
            Self::DecimalFloat(e) => {
                format!("The Float operation failed with a decimal float error: {e:?}")
            }
            Self::DecimalFloatSelector(selector) => match selector {
                Ok(selector) => selector.to_readable_msg().to_string(),
                Err(unknown) => format!(
                    "The Float operation reverted with an unrecognised error selector: {unknown}"
                ),
            },
            Self::Access(e) => {
                format!("Failed to access the thread-local EVM used to run Float operations: {e}")
            }
            Self::InvalidHex(s) => {
                format!("The provided value is not a valid hex string: {s}")
            }
            Self::AlloyFromHexError(e) => {
                format!("Failed to decode the provided hex string: {e}")
            }
            Self::AlloyParseError(e) => {
                format!("Failed to parse the provided number: {e}")
            }
            Self::AlloyParseSignedError(e) => {
                format!("Failed to parse the provided signed number: {e}")
            }
            Self::JsSysError(s) => {
                format!("A JavaScript error occurred while running the Float operation: {s}")
            }
        }
    }
}

impl From<FloatError> for WasmEncodedError {
    fn from(value: FloatError) -> Self {
        WasmEncodedError {
            msg: value.to_string(),
            readable_msg: value.to_readable_msg(),
        }
    }
}

impl From<JsError> for FloatError {
    fn from(value: JsError) -> Self {
        FloatError::JsSysError(value.to_string().into())
    }
}

impl From<RangeError> for FloatError {
    fn from(value: RangeError) -> Self {
        FloatError::JsSysError(value.to_string().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_float_error_selector_readable_msgs() {
        use DecimalFloatErrorSelector as S;
        let cases = [
            (
                S::ExponentOverflow,
                "The number is too large to represent as a Float (its exponent exceeds the maximum supported magnitude).",
            ),
            (
                S::ParseEmptyDecimalString,
                "There are no digits where the number should start.",
            ),
            (
                S::MalformedDecimalPoint,
                "The decimal point is not followed by any digits.",
            ),
            (
                S::MalformedExponentDigits,
                "The exponent has no digits after the `e`.",
            ),
            (
                S::ParseDecimalPrecisionLoss,
                "The number has more precision than a Float can represent exactly.",
            ),
            (
                S::ParseDecimalFloatExcessCharacters,
                "There are unexpected characters after the number.",
            ),
        ];
        for (selector, msg) in cases {
            assert_eq!(selector.to_readable_msg(), msg);
        }
    }

    #[test]
    fn test_decimal_float_error_selector_from_signature() {
        use DecimalFloatErrorSelector as S;
        let cases = [
            ("ExponentOverflow(int256,int256)", S::ExponentOverflow),
            (
                "ParseEmptyDecimalString(uint256)",
                S::ParseEmptyDecimalString,
            ),
            ("MalformedDecimalPoint(uint256)", S::MalformedDecimalPoint),
            (
                "MalformedExponentDigits(uint256)",
                S::MalformedExponentDigits,
            ),
            (
                "ParseDecimalPrecisionLoss(uint256)",
                S::ParseDecimalPrecisionLoss,
            ),
            (
                "ParseDecimalFloatExcessCharacters()",
                S::ParseDecimalFloatExcessCharacters,
            ),
        ];
        for (signature, want) in cases {
            let selector =
                FixedBytes::<4>::from_slice(&alloy::primitives::keccak256(signature)[..4]);
            assert_eq!(
                DecimalFloatErrorSelector::try_from(selector),
                Ok(want),
                "{signature}"
            );
        }
        for unknown in [
            "Log10Zero()",
            "ExponentUnderflow(int256,int256)",
            "ParseDecimalNoSignificantDigits(uint256)",
        ] {
            let selector = FixedBytes::<4>::from_slice(&alloy::primitives::keccak256(unknown)[..4]);
            assert_eq!(
                DecimalFloatErrorSelector::try_from(selector),
                Err(selector),
                "{unknown}"
            );
        }
    }

    #[test]
    fn test_float_error_readable_msg_invalid_hex() {
        let err = FloatError::InvalidHex("zz".to_string());
        assert_eq!(
            err.to_readable_msg(),
            "The provided value is not a valid hex string: zz"
        );
    }

    #[test]
    fn test_float_error_readable_msg_js_sys() {
        let err = FloatError::JsSysError("boom".to_string());
        assert_eq!(
            err.to_readable_msg(),
            "A JavaScript error occurred while running the Float operation: boom"
        );
    }

    #[test]
    fn test_float_error_readable_msg_decimal_float_selector_known() {
        let err =
            FloatError::DecimalFloatSelector(Ok(DecimalFloatErrorSelector::MalformedDecimalPoint));
        assert_eq!(
            err.to_readable_msg(),
            "The decimal point is not followed by any digits."
        );
    }

    #[test]
    fn test_float_error_readable_msg_decimal_float_selector_unknown() {
        let unknown = FixedBytes::<4>::from([0xde, 0xad, 0xbe, 0xef]);
        let err = FloatError::DecimalFloatSelector(Err(unknown));
        assert_eq!(
            err.to_readable_msg(),
            "The Float operation reverted with an unrecognised error selector: 0xdeadbeef"
        );
    }

    #[test]
    fn test_wasm_encoded_error_uses_readable_msg() {
        let err = FloatError::InvalidHex("zz".to_string());
        let short = err.to_string();
        let readable = err.to_readable_msg();
        let encoded: WasmEncodedError = err.into();
        assert_eq!(encoded.msg, short);
        assert_eq!(encoded.readable_msg, readable);
        // The detailed readable message is distinct from the short message.
        assert_ne!(encoded.msg, encoded.readable_msg);
    }
}
