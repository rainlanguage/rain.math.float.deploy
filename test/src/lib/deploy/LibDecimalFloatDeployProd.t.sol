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
    /// The assertion messages do NOT name the network. The walk below prefixes
    /// every line it collects with the network, and naming it here too produced
    /// `flare: flare: DecimalFloat not deployed` in CI. The prefix belongs to the
    /// walk rather than here, because a cheatcode failure — an unset
    /// `*_RPC_URL`, say — reverts before any assertion and so can only be named
    /// by the caller.
    /// Takes a fork ID rather than a network name, and SELECTS ONLY. The fork
    /// must already exist. See `testProdDeploymentEverySupportedNetwork` for why
    /// creating it here would be wrong.
    function checkProdDeployment(uint256 forkId) internal {
        vm.selectFork(forkId);

        address logTables = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS;
        assertTrue(logTables.code.length > 0, "log tables not deployed");
        assertEq(
            logTables.codehash, LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH, "log tables code hash mismatch"
        );

        address decimalFloat = LibDecimalFloatDeploy.ZOLTU_DEPLOYED_DECIMAL_FLOAT_ADDRESS;
        assertTrue(decimalFloat.code.length > 0, "DecimalFloat not deployed");
        assertEq(
            decimalFloat.codehash, LibDecimalFloatDeploy.DECIMAL_FLOAT_CONTRACT_HASH, "DecimalFloat code hash mismatch"
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
    /// Two upstream tests guard the CONFIG against the same list, so config and
    /// coverage now derive from one place:
    ///
    /// - `testSupportedNetworksAreFullyConfigured`, in
    ///   `RainDeployVerifySnapshot` and inherited through
    ///   `test/src/abstract/DecimalFloatDeploySnapshot.t.sol`. It reads
    ///   `foundry.toml` and needs no fork, so it is the one that catches a
    ///   missing `[rpc_endpoints]` or `[etherscan]` entry locally.
    /// - `testSupportedNetworkChainIdsAreBound`, in `RainDeployVerifyChain` and
    ///   inherited through `test/src/abstract/DecimalFloatDeployChain.t.sol`. It
    ///   forks each network that states a chain id, so it needs `*_RPC_URL`.
    /// Each network is checked through an external call so a revert on one does
    /// not abort the walk. The five test functions this replaced reported per
    /// network; a bare loop would hide every network after the first failure,
    /// which is the opposite of useful when the question is WHICH chains are
    /// missing a deployment.
    function checkProdDeploymentExternal(uint256 forkId) external {
        checkProdDeployment(forkId);
    }

    /// Creates one fork, external so an unreachable endpoint is reported against
    /// its own network instead of aborting the run.
    function createForkExternal(string memory network) external returns (uint256) {
        return vm.createFork(network);
    }

    /// The revert reason as text, so the failure message names what went wrong
    /// rather than handing a CI reader a hex blob to decode.
    ///
    /// A failed assertion and a cheatcode error both carry a 4 byte selector
    /// then an ABI encoded string, so both decode the same way. Anything with a
    /// different shape falls back to hex rather than reverting inside the
    /// handler and losing every other network's result with it.
    ///
    /// The decode goes through an external call so that a malformed payload is
    /// CAUGHT rather than merely predicted. The cheap shape checks below cannot
    /// be complete: a payload can carry the right offset word and still declare
    /// a length its data does not cover, and `abi.decode` reverts on that. This
    /// function runs inside the walk's `catch`, so a revert here does not fall
    /// back — it propagates and takes every other network's result with it,
    /// which is the exact failure the fallback exists to prevent. Bounds
    /// arithmetic would have to be right about every malformed shape; `try` is
    /// right about all of them by construction.
    function decodeStringExternal(bytes memory payload) external pure returns (string memory) {
        return abi.decode(payload, (string));
    }

    /// See `decodeStringExternal` for why the decode is an external call.
    function reasonOf(bytes memory err) internal view returns (string memory) {
        if (err.length < 68) {
            return vm.toString(err);
        }
        bytes memory payload = new bytes(err.length - 4);
        for (uint256 i = 0; i < payload.length; i++) {
            payload[i] = err[i + 4];
        }
        // The offset word of an ABI encoded string is always 0x20; a payload
        // that does not start with it is not one. Kept as a shape check even
        // though the `try` below would catch the resulting revert, because a
        // custom error whose first word happens to be a valid offset could
        // otherwise decode into a garbage string instead of falling back.
        //
        // Truncating to the first word is the entire point of the cast: the
        // rest of the payload is the string this is deciding whether to decode.
        // forge-lint: disable-next-line(unsafe-typecast)
        bytes32 offsetWord = bytes32(payload);
        if (uint256(offsetWord) != 0x20) {
            return vm.toString(err);
        }
        try this.decodeStringExternal(payload) returns (string memory reason) {
            return reason;
        } catch {
            return vm.toString(err);
        }
    }

    /// A well formed `Error(string)` payload decodes to its text, which is the
    /// whole reason the walk reports reasons rather than hex.
    function testReasonOfDecodesRevertString() external view {
        bytes memory err = abi.encodeWithSignature("Error(string)", "arbitrum: nope");
        assertEq(reasonOf(err), "arbitrum: nope");
    }

    /// A payload carrying the right offset word but a length its data does not
    /// cover. `abi.decode` reverts on this. Before the decode was moved behind
    /// an external call that revert propagated out of the walk's `catch` and
    /// took every other network's result with it, so this asserts the fallback
    /// rather than the decode.
    function testReasonOfSurvivesMalformedStringPayload() external view {
        bytes memory err = abi.encodePacked(bytes4(0x08c379a0), uint256(0x20), type(uint256).max);
        assertEq(err.length, 68, "payload should be exactly the minimum accepted length");
        assertEq(reasonOf(err), vm.toString(err), "malformed payload should fall back to hex");
    }

    /// The other half of the test above, and the reason it is not vacuous: the
    /// same payload really does revert `abi.decode`. Without this, a change that
    /// made the payload decodable would leave the fallback test passing for the
    /// wrong reason and prove nothing about the `try`.
    function testMalformedStringPayloadDoesRevertAbiDecode() external {
        bytes memory payload = abi.encodePacked(uint256(0x20), type(uint256).max);
        vm.expectRevert();
        this.decodeStringExternal(payload);
    }

    /// EVERY fork is created before ANY fork is selected, in two passes.
    ///
    /// `LibRainDeploy.createForks` documents why: foundry captures the pre-fork
    /// account set when the first fork is SELECTED, and seeds every fork created
    /// after that capture with it, so such a fork reads any address the caller
    /// had already touched as the empty account a bare 31337 EVM has for it. A
    /// single `createSelectFork` loop creates fork N after the capture for every
    /// N above the first.
    ///
    /// This loop is correct today only by accident: nothing here touches the
    /// pinned addresses before the first select, so nothing is in the captured
    /// set to carry forward. A `setUp` that etched the tables — the obvious thing
    /// for someone to add — would put them in it and silently turn eight of the
    /// nine networks into `DecimalFloat not deployed`, naming real addresses on
    /// chains that really hold them. Two passes remove the accident.
    ///
    /// Not `LibRainDeploy.createForks` itself, which is this same creation in one
    /// call. It creates the forks in a plain loop, so the first unreachable
    /// endpoint reverts and the remaining networks are never reported. Creating
    /// them one at a time through an external call keeps an outage attributed to
    /// its own network, the same way a missing deployment is.
    function testProdDeploymentEverySupportedNetwork() external {
        string[] memory networks = LibRainDeploy.supportedNetworks();
        // A list that came back empty would pass every assertion below by
        // never running one.
        assertTrue(networks.length > 0, "no supported networks");

        string memory failures = "";

        // Pass 1: create every fork. Nothing is selected yet.
        uint256[] memory forkIds = new uint256[](networks.length);
        bool[] memory created = new bool[](networks.length);
        for (uint256 i = 0; i < networks.length; i++) {
            try this.createForkExternal(networks[i]) returns (uint256 forkId) {
                forkIds[i] = forkId;
                created[i] = true;
            } catch (bytes memory err) {
                failures = string.concat(failures, "\n", networks[i], ": ", reasonOf(err));
            }
        }

        // Pass 2: select each in turn and check it. A network whose fork could
        // not be created has already been reported above.
        for (uint256 i = 0; i < networks.length; i++) {
            if (!created[i]) {
                continue;
            }
            try this.checkProdDeploymentExternal(forkIds[i]) {}
            catch (bytes memory err) {
                failures = string.concat(failures, "\n", networks[i], ": ", reasonOf(err));
            }
        }

        assertEq(failures, "", failures);
    }
}
