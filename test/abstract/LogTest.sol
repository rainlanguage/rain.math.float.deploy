// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

// Re-export console2 here for convenience.
// forge-lint: disable-next-line(unused-import)
import {Test, console2} from "forge-std-1.16.2/src/Test.sol";
import {LibDataContract} from "rain-datacontract-0.2.0/src/lib/LibDataContract.sol";
import {LibDecimalFloatDeploy} from "src/lib/deploy/LibDecimalFloatDeploy.sol";
import {LibEtchLogTables} from "script/lib/LibEtchLogTables.sol";

abstract contract LogTest is Test {
    address sTables;

    /// Etch the log tables runtime at the Zoltu-deterministic deployment
    /// address used by the production `DecimalFloat` contract. Without this,
    /// the deployed tests below would `extcodecopy` from an empty address
    /// while the external helper does the same, both agreeing on garbage.
    function setUp() public virtual {
        LibEtchLogTables.etchLogTables(vm);
        assertEq(
            LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS.codehash,
            LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH,
            "etched tables codehash mismatch"
        );
    }

    /// A SECOND copy of the tables, at a nonce-dependent address that is
    /// deliberately not `ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS` — `setUp` has
    /// already put a copy there.
    ///
    /// The raw `create` is the point and not an un-migrated call site. Tests
    /// that take a log-tables address as an argument use this one so that
    /// passing it is distinguishable from reading the pinned constant: if both
    /// addresses were the pin, a function that ignored its argument entirely
    /// would still pass. `LibRainDeploy.deployZoltu` cannot serve here, because
    /// the address it lands on is a function of the creation code alone, so for
    /// this creation code it is the pin — which `setUp` has occupied, making the
    /// second deployment fail rather than yield a distinct address.
    function logTables() internal returns (address) {
        if (sTables == address(0)) {
            bytes memory tables = LibDecimalFloatDeploy.combinedTables();
            bytes memory creationCode = LibDataContract.contractCreationCode(tables);
            address tablesAddress;
            assembly ("memory-safe") {
                tablesAddress := create(0, add(creationCode, 0x20), mload(creationCode))
            }
            assertTrue(tablesAddress != address(0), "Failed to deploy tables");
            assertEq(
                tablesAddress.codehash,
                LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH,
                "Deployed tables codehash does not match expected value"
            );
            sTables = tablesAddress;
        }
        return sTables;
    }
}
