// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {Vm} from "forge-std-1.16.2/src/Vm.sol";
import {LibDataContract} from "rain-datacontract-0.2.0/src/lib/LibDataContract.sol";
import {LibDecimalFloatDeploy} from "src/lib/deploy/LibDecimalFloatDeploy.sol";
import {LibRainDeploy} from "rain-deploy-0.1.11/src/lib/LibRainDeploy.sol";

/// @notice Shared logic for planting the log-tables data contract at its
/// Zoltu-deterministic address inside a forge VM, for tests that need
/// `DecimalFloat` operations to work without a real on-chain deploy.
///
/// NOT used by `script/Deploy.sol`, which does not import this and must not:
/// its docstring states that nothing plants the tables locally to get a
/// simulation through, because a chain without them is a chain the suite must
/// not be deployed to — `DecimalFloat`'s constructor reverts there, and
/// `LibRainDeploy.deployToNetworks` raises `MissingDependency` rather than
/// papering over it. An earlier version of this comment claimed the broadcast
/// as a consumer; the broadcast is entirely `RainDeployBroadcast.run()` and has
/// no etch hook to call this from.
///
/// Callers are `test/abstract/LogTest.sol`,
/// `test/src/concrete/DecimalFloat.constructor.t.sol`,
/// `test/src/abstract/DecimalFloatDeployChain.t.sol` and
/// `test/src/abstract/DecimalFloatDeploySnapshot.t.sol`.
library LibEtchLogTables {
    /// @notice Deploys the log-tables data contract to a temporary address,
    /// copies its runtime code, and etches that runtime at
    /// `LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS` so the
    /// codehash matches `LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH`.
    /// Deployed through the Zoltu factory rather than a raw `create` into a
    /// temporary address followed by an etch across. The factory is what puts
    /// the contract at `ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS` in the first place,
    /// so using it makes the address a consequence of the creation code instead
    /// of something asserted after the fact — and the temp-address dance existed
    /// only because a raw `create` lands at a nonce-dependent address.
    ///
    /// The returned address is checked against the pin. A deployment that does
    /// not land on it means the creation code and the committed snapshot have
    /// diverged, which is worth a failure here rather than an etch that hides it
    /// by writing the runtime wherever the pin says regardless.
    function etchLogTables(Vm vm) internal {
        LibRainDeploy.etchZoltuFactory(vm);
        bytes memory tables = LibDecimalFloatDeploy.combinedTables();
        address deployed = LibRainDeploy.deployZoltu(LibDataContract.contractCreationCode(tables));
        require(
            deployed == LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS,
            "log tables deployed off their pinned Zoltu address"
        );
    }
}
