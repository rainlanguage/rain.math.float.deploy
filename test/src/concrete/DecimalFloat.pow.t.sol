// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {LibDecimalFloat, Float} from "rain-math-float-0.2.45/src/lib/LibDecimalFloat.sol";
import {Test} from "forge-std-1.17.0/src/Test.sol";
import {DecimalFloat} from "src/concrete/DecimalFloat.sol";

contract DecimalFloatPowTest is Test {
    using LibDecimalFloat for Float;

    function powExternal(Float a, Float b) external pure returns (Float) {
        return a.pow(b);
    }

    function testPowDeployed(Float a, Float b) external {
        DecimalFloat deployed = new DecimalFloat();

        try this.powExternal(a, b) returns (Float c) {
            Float deployedC = deployed.pow(a, b);

            assertEq(Float.unwrap(c), Float.unwrap(deployedC));
        } catch (bytes memory err) {
            vm.expectRevert(err);
            deployed.pow(a, b);
        }
    }
}
