// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RainDeployVerifySnapshot} from "rain-deploy-0.1.7/src/abstract/RainDeployVerifySnapshot.sol";
import {DecimalFloatDeploySuites} from "src/abstract/DecimalFloatDeploySuites.sol";
import {LibEtchLogTables} from "script/lib/LibEtchLogTables.sol";

/// @title DecimalFloatDeploySnapshotTest
/// @notice Binds this repo's declaration to `RainDeployVerifySnapshot`: every
/// deploy-pin assertion over the `log-tables` and `decimal-float` suites that
/// needs no network.
contract DecimalFloatDeploySnapshotTest is DecimalFloatDeploySuites, RainDeployVerifySnapshot {
    /// The inherited derivation runs every suite's creation code through the
    /// Zoltu factory to read back its code hash, and `DecimalFloat`'s
    /// constructor reverts unless the log tables are already at their Zoltu
    /// address. Planting them here is what the on-chain deploy order does for
    /// real, and it survives each derivation: `deriveDeployment` clears only the
    /// address it is deriving, and restores the state snapshot it took before
    /// doing so.
    function setUp() public {
        LibEtchLogTables.etchLogTables(vm);
    }
}
