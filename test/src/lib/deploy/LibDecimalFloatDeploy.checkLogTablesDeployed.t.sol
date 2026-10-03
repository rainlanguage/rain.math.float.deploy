// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {LibDecimalFloatDeploy} from "src/lib/deploy/LibDecimalFloatDeploy.sol";
import {LibDataContract} from "rain-datacontract-0.2.0/src/lib/LibDataContract.sol";
import {LibRainDeploy} from "rain-deploy-0.1.12/src/lib/LibRainDeploy.sol";
import {LogTablesNotDeployed} from "rain-math-float-0.2.4/src/error/ErrDecimalFloat.sol";

/// Direct tests for `LibDecimalFloatDeploy.checkLogTablesDeployed`. These
/// deliberately do NOT inherit `LogTest` so the table address starts empty.
contract LibDecimalFloatDeployCheckLogTablesDeployedTest is Test {
    /// Empty address → revert with the expected error and arguments.
    function testCheckLogTablesDeployedRevertsWhenMissing() external {
        bytes32 actualCodehash = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS.codehash;
        vm.expectRevert(
            abi.encodeWithSelector(
                LogTablesNotDeployed.selector,
                LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS,
                LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH,
                actualCodehash
            )
        );
        this.callCheckLogTablesDeployed();
    }

    /// Wrong bytecode at the address → codehash mismatch → revert.
    function testCheckLogTablesDeployedRevertsOnWrongCodehash() external {
        bytes memory junk = hex"deadbeef";
        vm.etch(LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS, junk);
        bytes32 actualCodehash = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS.codehash;
        assertTrue(actualCodehash != LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH);
        vm.expectRevert(
            abi.encodeWithSelector(
                LogTablesNotDeployed.selector,
                LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS,
                LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH,
                actualCodehash
            )
        );
        this.callCheckLogTablesDeployed();
    }

    /// Correct table runtime at the expected address → no revert.
    ///
    /// Through the factory, which lands the deployment on the pin as a
    /// consequence of the creation code. An earlier version deployed to a
    /// nonce-dependent address and etched the runtime across to the pin, which
    /// is what the factory does — reproduced by hand, and weaker for it: an
    /// etch writes the runtime wherever it is told, so it would keep passing if
    /// the creation code and the pinned address had diverged. The factory not
    /// landing on the pin is a failure here instead.
    function testCheckLogTablesDeployedSucceedsWhenPresent() external {
        LibRainDeploy.etchZoltuFactory(vm);
        bytes memory tables = LibDecimalFloatDeploy.combinedTables();
        address deployed = LibRainDeploy.deployZoltu(LibDataContract.contractCreationCode(tables));
        assertEq(deployed, LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS, "tables off their pinned address");
        // Should not revert.
        this.callCheckLogTablesDeployed();
    }

    /// Mutation: with correct etch the call succeeds; replacing the runtime
    /// with same-length zeroes flips the codehash and the call reverts.
    /// Confirms the test is keyed on the codehash check, not anything else.
    function testCheckLogTablesDeployedMutation() external {
        LibRainDeploy.etchZoltuFactory(vm);
        bytes memory tables = LibDecimalFloatDeploy.combinedTables();
        address deployed = LibRainDeploy.deployZoltu(LibDataContract.contractCreationCode(tables));
        assertEq(deployed, LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS, "tables off their pinned address");
        this.callCheckLogTablesDeployed();

        bytes memory zeros = new bytes(deployed.code.length);
        vm.etch(LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS, zeros);
        vm.expectRevert();
        this.callCheckLogTablesDeployed();
    }

    /// `vm.expectRevert` only catches reverts in external calls, so wrap the
    /// internal lib call.
    function callCheckLogTablesDeployed() external view {
        LibDecimalFloatDeploy.checkLogTablesDeployed();
    }
}
