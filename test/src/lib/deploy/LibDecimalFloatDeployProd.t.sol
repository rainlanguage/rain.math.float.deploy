// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.2/src/Test.sol";
import {LibDecimalFloatDeploy} from "src/lib/deploy/LibDecimalFloatDeploy.sol";
import {LibRainDeploy} from "rain-deploy-0.1.11/src/lib/LibRainDeploy.sol";

/// @title LibDecimalFloatDeployProdTest
/// @notice Verifies that both the log tables data contract and the DecimalFloat
/// contract are deployed on every supported production network with the expected
/// addresses and code hashes.
contract LibDecimalFloatDeployProdTest is Test {
    function checkProdDeployment(string memory network) internal {
        vm.createSelectFork(network);

        address logTables = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS;
        assertTrue(logTables.code.length > 0, string.concat(network, ": log tables not deployed"));
        assertEq(
            logTables.codehash,
            LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH,
            string.concat(network, ": log tables code hash mismatch")
        );

        address decimalFloat = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_DECIMAL_FLOAT_ADDRESS;
        assertTrue(decimalFloat.code.length > 0, string.concat(network, ": DecimalFloat not deployed"));
        assertEq(
            decimalFloat.codehash,
            LibDecimalFloatDeploy.DECIMAL_FLOAT_CONTRACT_HASH,
            string.concat(network, ": DecimalFloat code hash mismatch")
        );
    }

    /// EVERY supported network, taken from `LibRainDeploy.supportedNetworks()`
    /// rather than named here.
    ///
    /// The five names this file used to list were a subset: `bsc`, `ethereum`,
    /// `hyperevm` and `robinhood` are supported, are configured in
    /// `[rpc_endpoints]` and `[etherscan]`, and were never checked. A hardcoded
    /// list cannot notice that, which is why the list is the source and this
    /// walks it — a network added upstream is covered here without an edit, and
    /// one removed stops being checked without a stale test failing.
    ///
    /// Upstream's `testSupportedNetworkChainIdsAreBound`, inherited through
    /// `test/src/abstract/DecimalFloatDeployChain.t.sol`, guards the config
    /// against the same list, so config and coverage now derive from one place.
    /// Each network is checked through an external call so a revert on one does
    /// not abort the walk. The five test functions this replaced reported per
    /// network; a bare loop would hide every network after the first failure,
    /// which is the opposite of useful when the question is WHICH chains are
    /// missing a deployment.
    function checkProdDeploymentExternal(string memory network) external {
        checkProdDeployment(network);
    }

    /// The revert reason as text, so the failure message names what went wrong
    /// rather than handing a CI reader a hex blob to decode.
    ///
    /// A failed assertion and a cheatcode error both carry a 4 byte selector
    /// then an ABI encoded string, so both decode the same way. Anything with a
    /// different shape falls back to hex rather than reverting inside the
    /// handler and losing every other network's result with it.
    function reasonOf(bytes memory err) internal pure returns (string memory) {
        if (err.length < 68) {
            return vm.toString(err);
        }
        bytes memory payload = new bytes(err.length - 4);
        for (uint256 i = 0; i < payload.length; i++) {
            payload[i] = err[i + 4];
        }
        // The offset word of an ABI encoded string is always 0x20; a payload
        // that does not start with it is not one.
        //
        // Truncating to the first word is the entire point of the cast: the
        // rest of the payload is the string this is deciding whether to decode.
        // forge-lint: disable-next-line(unsafe-typecast)
        bytes32 offsetWord = bytes32(payload);
        if (uint256(offsetWord) != 0x20) {
            return vm.toString(err);
        }
        return abi.decode(payload, (string));
    }

    function testProdDeploymentEverySupportedNetwork() external {
        string[] memory networks = LibRainDeploy.supportedNetworks();
        // A list that came back empty would pass every assertion below by
        // never running one.
        assertTrue(networks.length > 0, "no supported networks");

        string memory failures = "";
        for (uint256 i = 0; i < networks.length; i++) {
            try this.checkProdDeploymentExternal(networks[i]) {}
            catch (bytes memory err) {
                failures = string.concat(failures, "\n", networks[i], ": ", reasonOf(err));
            }
        }
        assertEq(failures, "", failures);
    }
}
