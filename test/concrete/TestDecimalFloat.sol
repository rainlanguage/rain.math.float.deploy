// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {LibDecimalFloat, Float} from "rain-math-float-0.3.11/src/lib/LibDecimalFloat.sol";

/// Exposes the library internals the Rust bindings' tests need beside the
/// `DecimalFloat` ABI: packing.
contract TestDecimalFloat {
    using LibDecimalFloat for Float;

    function packLossless(int224 coefficient, int32 exponent) external pure returns (Float) {
        return LibDecimalFloat.packLossless(coefficient, exponent);
    }

    function unpack(Float float) external pure returns (int256, int256) {
        return LibDecimalFloat.unpack(float);
    }
}
