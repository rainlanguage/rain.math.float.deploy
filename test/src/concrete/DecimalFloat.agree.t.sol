// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {LibDecimalFloat, Float} from "rain-math-float-0.2.4/src/lib/LibDecimalFloat.sol";
import {AgreeToleranceNegative, AgreeNoPositiveTolerance} from "rain-math-float-0.2.4/src/error/ErrDecimalFloat.sol";
import {LogTest} from "test/abstract/LogTest.sol";
import {DecimalFloat} from "src/concrete/DecimalFloat.sol";

contract DecimalFloatAgreeTest is LogTest {
    using LibDecimalFloat for Float;

    function f(int256 signedCoefficient, int256 exponent) internal pure returns (Float) {
        return LibDecimalFloat.packLossless(signedCoefficient, exponent);
    }

    function agreeExternal(Float absolute, Float proportional, Float lowest, Float highest)
        external
        pure
        returns (bool)
    {
        return LibDecimalFloat.agree(absolute, proportional, lowest, highest);
    }

    /// The deployed concrete answers what the library answers, or reverts with
    /// the same data. Fuzzed over arbitrary packed words in all four operands,
    /// so the tolerance guard's rejections are exercised alongside the ordinary
    /// answers rather than only the happy path.
    function testAgreeDeployed(Float absolute, Float proportional, Float lowest, Float highest) external {
        DecimalFloat deployed = new DecimalFloat();

        try this.agreeExternal(absolute, proportional, lowest, highest) returns (bool c) {
            bool deployedC = deployed.agree(absolute, proportional, lowest, highest);

            assertEq(c, deployedC);
        } catch (bytes memory err) {
            vm.expectRevert(err);
            deployed.agree(absolute, proportional, lowest, highest);
        }
    }

    /// The guard reaches an offchain caller as a revert, not a silent answer.
    /// The fuzz above would pass if both sides returned the same wrong thing, so
    /// these pin the specific errors with their tolerances.
    function testAgreeDeployedRejectsBadTolerances() external {
        DecimalFloat deployed = new DecimalFloat();

        vm.expectRevert(abi.encodeWithSelector(AgreeToleranceNegative.selector, f(-1, 0), f(1, -2)));
        deployed.agree(f(-1, 0), f(1, -2), f(99, 0), f(100, 0));

        vm.expectRevert(abi.encodeWithSelector(AgreeNoPositiveTolerance.selector, f(0, 0), f(0, 0)));
        deployed.agree(f(0, 0), f(0, 0), f(100, 0), f(100, 0));
    }

    /// A concrete answer either side of the limit, so the exposed function is
    /// pinned to the formula and not merely to whatever the library returns.
    /// 100 - 99 == 1 == 0.01 * 100, so the boundary is met and accepted.
    function testAgreeDeployedBoundary() external {
        DecimalFloat deployed = new DecimalFloat();

        assertTrue(deployed.agree(f(0, 0), f(1, -2), f(99, 0), f(100, 0)));
        assertFalse(deployed.agree(f(0, 0), f(1, -2), f(98999, -3), f(100, 0)));
    }
}
