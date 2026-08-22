// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RainDeployVerifyChain} from "rain-deploy-0.1.7/src/abstract/RainDeployVerifyChain.sol";
import {DecimalFloatDeploySuites} from "src/abstract/DecimalFloatDeploySuites.sol";
import {LibEtchLogTables} from "script/lib/LibEtchLogTables.sol";

/// @title DecimalFloatDeployChainTest
/// @notice Binds this repo's declaration to `RainDeployVerifyChain`: every
/// frozen release of the log tables and of `DecimalFloat` is live, with the
/// code it froze, on every supported network.
///
/// This repo has cut no release under the frozen-record model yet, so
/// `releasedSuites()` is empty and the inherited test has no subject and forks
/// nothing. It gains one the moment the first `src/generated/<tag>/` lands. The
/// live CURRENT pins are checked by `LibDecimalFloatDeployProdTest`, which is a
/// different claim: the candidate is what the NEXT release will be, and a
/// released suite is a deployment that already happened.
contract DecimalFloatDeployChainTest is DecimalFloatDeploySuites, RainDeployVerifyChain {
    /// Plants the log tables for the same reason the snapshot suite does: the
    /// derivations all run on the local EVM before anything forks, and
    /// `DecimalFloat`'s constructor reverts without the tables in place.
    function setUp() public {
        LibEtchLogTables.etchLogTables(vm);
    }
}
