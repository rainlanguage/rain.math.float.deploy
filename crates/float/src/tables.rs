//! The log tables `LibLogTable` ships, read from the harness contract so a
//! test can check them against an independent derivation without copying
//! them.

use alloy::primitives::Bytes;
use alloy::sol_types::SolCall;

use crate::{execute_test_call, FloatError, TestDecimalFloat};

fn read<C: SolCall>(call: C) -> Result<C::Return, FloatError> {
    execute_test_call(Bytes::from(call.abi_encode()), |output| {
        Ok(C::abi_decode_returns(output.as_ref())?)
    })
}

/// `ALT_TABLE_FLAG`: the bit a main log table entry sets to select the
/// alternative small table.
pub fn alt_table_flag() -> Result<u16, FloatError> {
    read(TestDecimalFloat::altTableFlagCall {})
}

/// `LibLogTable.logTableDec()`.
pub fn log_table_dec() -> Result<[[u16; 10]; 90], FloatError> {
    read(TestDecimalFloat::logTableDecCall {})
}

/// `LibLogTable.logTableDecSmall()`.
pub fn log_table_dec_small() -> Result<[[u8; 10]; 90], FloatError> {
    read(TestDecimalFloat::logTableDecSmallCall {})
}

/// `LibLogTable.logTableDecSmallAlt()`.
pub fn log_table_dec_small_alt() -> Result<[[u8; 10]; 10], FloatError> {
    read(TestDecimalFloat::logTableDecSmallAltCall {})
}

/// `LibLogTable.antiLogTableDec()`.
pub fn anti_log_table_dec() -> Result<[[u16; 10]; 100], FloatError> {
    read(TestDecimalFloat::antiLogTableDecCall {})
}

/// `LibLogTable.antiLogTableDecSmall()`.
pub fn anti_log_table_dec_small() -> Result<[[u8; 10]; 100], FloatError> {
    read(TestDecimalFloat::antiLogTableDecSmallCall {})
}
