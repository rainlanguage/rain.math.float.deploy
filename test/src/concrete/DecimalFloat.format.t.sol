// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Float} from "rain-math-float-0.2.45/src/lib/LibDecimalFloat.sol";
import {Test} from "forge-std-1.17.0/src/Test.sol";
import {DecimalFloat} from "src/concrete/DecimalFloat.sol";
import {LibFormatDecimalFloat} from "rain-math-float-0.2.45/src/lib/format/LibFormatDecimalFloat.sol";

contract DecimalFloatFormatTest is Test {
    function formatBoolExternal(Float a, bool scientific) external pure returns (string memory) {
        return LibFormatDecimalFloat.toDecimalString(a, scientific);
    }

    function testFormatBoolDeployed(Float a, bool scientific) external {
        DecimalFloat deployed = new DecimalFloat();

        try this.formatBoolExternal(a, scientific) returns (string memory str) {
            string memory deployedStr = deployed.format(a, scientific);

            assertEq(str, deployedStr);
        } catch (bytes memory err) {
            vm.expectRevert(err);
            deployed.format(a, scientific);
        }
    }
}
