// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {LibRainDeploy} from "rain-deploy-0.1.18/src/lib/LibRainDeploy.sol";
import {LibDecimalFloatDeploy} from "src/lib/deploy/LibDecimalFloatDeploy.sol";
import {DecimalFloat} from "src/concrete/DecimalFloat.sol";

contract LibDecimalFloatDeployTest is Test {
    /// `LibRainDeploy.etchZoltuFactory` puts the factory bytecode at the
    /// deterministic factory address using `vm.etch`. No fork required —
    /// the test runs entirely in-memory, which avoids RPC-flake and the
    /// "address already has code" collision when a deploy target is
    /// already populated on the chosen prod chain.
    function setUp() public {
        LibRainDeploy.etchZoltuFactory(vm);
    }

    function testDeployAddress() external {
        address deployedAddress = LibRainDeploy.deployZoltu(type(DecimalFloat).creationCode);

        assertEq(deployedAddress, LibDecimalFloatDeploy.ZOLTU_DEPLOYED_DECIMAL_FLOAT_ADDRESS);
        assertTrue(address(deployedAddress).code.length > 0, "Deployed address has no code");

        assertEq(address(deployedAddress).codehash, LibDecimalFloatDeploy.DECIMAL_FLOAT_CONTRACT_HASH);
    }

    function testExpectedCodeHashDecimalFloat() external {
        DecimalFloat decimalFloat = new DecimalFloat();

        assertEq(address(decimalFloat).codehash, LibDecimalFloatDeploy.DECIMAL_FLOAT_CONTRACT_HASH);
    }
}
