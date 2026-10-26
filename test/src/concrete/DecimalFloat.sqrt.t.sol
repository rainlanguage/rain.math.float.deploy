// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {LibDecimalFloat, Float} from "rain-math-float-0.3.11/src/lib/LibDecimalFloat.sol";
import {Test} from "forge-std-1.17.0/src/Test.sol";
import {DecimalFloat} from "src/concrete/DecimalFloat.sol";

contract DecimalFloatSqrtTest is Test {
    using LibDecimalFloat for Float;

    function sqrtExternal(Float a) external pure returns (Float) {
        return a.sqrt();
    }

    function testSqrtDeployed(Float a) external {
        DecimalFloat deployed = new DecimalFloat();

        try this.sqrtExternal(a) returns (Float c) {
            Float deployedC = deployed.sqrt(a);

            assertEq(Float.unwrap(c), Float.unwrap(deployedC));
        } catch (bytes memory err) {
            vm.expectRevert(err);
            deployed.sqrt(a);
        }
    }
}
