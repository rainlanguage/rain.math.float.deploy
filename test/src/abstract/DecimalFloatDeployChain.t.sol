// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RainDeployVerify} from "rain-deploy-0.1.11/src/abstract/RainDeployVerify.sol";
import {DecimalFloatDeploySuites} from "src/abstract/DecimalFloatDeploySuites.sol";
import {LibEtchLogTables} from "script/lib/LibEtchLogTables.sol";

/// @title DecimalFloatDeployChainTest
/// @notice Binds this repo's declaration to `RainDeployVerify`: every frozen
/// release of the log tables and of `DecimalFloat` is live, with the code it
/// froze, on every supported network.
///
/// The live CURRENT pins are checked by `LibDecimalFloatDeployProdTest`, which
/// is a different claim: the candidate is what the NEXT release will be, and a
/// released suite is a deployment that already happened.
///
/// `RainDeployVerify` rather than `RainDeployVerifyChain`, because upstream
/// states it is "the ONE contract a deploy repo binds" and reserves the right to
/// put a new check directly on it: a check on the union reaches a repo that
/// bound only the two halves never, and nothing red-lines the repo that misses
/// it. Binding the union here is what makes a version bump deliver it.
///
/// `DecimalFloatDeploySnapshotTest` still binds `RainDeployVerifySnapshot`
/// alone, which is why that half is bound twice. That is deliberate and is the
/// cheap side of the trade: the snapshot group forks nothing, so binding it in
/// its own contract is what lets a job with no RPC credentials select it with
/// `--match-contract` — selection happens at a contract boundary, so a single
/// combined contract could not offer it. The duplicated runs are network-free
/// and fast; losing the credential-free run, or losing a future union-level
/// check, would both cost more.
contract DecimalFloatDeployChainTest is DecimalFloatDeploySuites, RainDeployVerify {
    /// Plants the log tables for the same reason the snapshot suite does: the
    /// derivations all run on the local EVM before anything forks, and
    /// `DecimalFloat`'s constructor reverts without the tables in place.
    function setUp() public {
        LibEtchLogTables.etchLogTables(vm);
    }
}
