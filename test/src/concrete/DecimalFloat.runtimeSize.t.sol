// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {RUNTIME_CODE as DECIMAL_FLOAT_RUNTIME_CODE} from "src/generated/candidate/DecimalFloat.sol";
import {DecimalFloat} from "src/concrete/DecimalFloat.sol";

contract DecimalFloatRuntimeSizeTest is Test {
    /// EIP-170 runtime code size limit.
    uint256 internal constant EIP_170_MAX_CODE_SIZE = 0x6000;

    function testDecimalFloatRuntimeCodeFitsEip170() external pure {
        assertLe(type(DecimalFloat).runtimeCode.length, EIP_170_MAX_CODE_SIZE, "source runtime code exceeds EIP-170");
    }

    function testDecimalFloatCandidateRuntimeCodeFitsEip170() external pure {
        assertLe(DECIMAL_FLOAT_RUNTIME_CODE.length, EIP_170_MAX_CODE_SIZE, "candidate runtime code exceeds EIP-170");
    }
}
