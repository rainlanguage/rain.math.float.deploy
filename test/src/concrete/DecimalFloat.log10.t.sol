// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {LibDecimalFloat, Float} from "rain-math-float-0.2.45/src/lib/LibDecimalFloat.sol";
import {Test} from "forge-std-1.17.0/src/Test.sol";
import {DecimalFloat} from "src/concrete/DecimalFloat.sol";

contract DecimalFloatLog10Test is Test {
    using LibDecimalFloat for Float;

    function log10External(Float a) external pure returns (Float) {
        return a.log10();
    }

    function testLog10Deployed(Float a) external {
        DecimalFloat deployed = new DecimalFloat();

        try this.log10External(a) returns (Float b) {
            Float deployedB = deployed.log10(a);

            assertEq(Float.unwrap(b), Float.unwrap(deployedB));
        } catch (bytes memory err) {
            vm.expectRevert(err);
            deployed.log10(a);
        }
    }
}
