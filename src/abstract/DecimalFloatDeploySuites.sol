// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {DeployCandidate, DeployDependency, DeploySuite, RainDeploySuitesBase} from "./RainDeploySuitesBase.sol";
import {LibDataContract} from "rain-datacontract-0.2.0/src/lib/LibDataContract.sol";
import {
    CREATION_CODE as LOG_TABLES_CREATION_CODE_CANDIDATE,
    RUNTIME_CODE as LOG_TABLES_RUNTIME_CODE_CANDIDATE
} from "../generated/candidate/LogTables.sol";
import {
    CREATION_CODE as DECIMAL_FLOAT_CREATION_CODE_CANDIDATE,
    RUNTIME_CODE as DECIMAL_FLOAT_RUNTIME_CODE_CANDIDATE
} from "../generated/candidate/DecimalFloat.sol";
import {LibDecimalFloatDeploy} from "../lib/deploy/LibDecimalFloatDeploy.sol";
import {LibReleasedSuites} from "../lib/LibReleasedSuites.sol";

/// @title DecimalFloatDeploySuites
/// @notice Everything this repo deploys, declared ONCE: the two hand-written
/// candidates below — `log-tables` and `decimal-float`, the same two keys
/// `Manual sol artifacts` dispatches — and the released side read from the
/// generated `LibReleasedSuites`, which `script/Build.sol` emits from the
/// frozen record.
///
/// It lives in `src/` rather than `test/` because `.soldeerignore` excludes
/// `test/` from the published package, and in a deploy repo the deployment
/// process is the product.
abstract contract DecimalFloatDeploySuites is RainDeploySuitesBase {
    /// @inheritdoc RainDeploySuitesBase
    function releasedSuites() internal pure override returns (DeploySuite[] memory) {
        return LibReleasedSuites.releasedSuites();
    }

    /// @inheritdoc RainDeploySuitesBase
    /// @dev Log tables first, which is also the order they must be BROADCAST
    /// in — `DecimalFloat`'s constructor reverts on a chain the tables are not
    /// on yet — and the order `script/Build.sol` regenerates them in.
    function candidateSuites() internal pure override returns (DeployCandidate[] memory) {
        DeployCandidate[] memory candidates = new DeployCandidate[](2);
        candidates[0] = logTablesCandidate();
        candidates[1] = decimalFloatCandidate();
        return candidates;
    }

    /// This repo's rolling log-tables candidate: the AOT log/anti-log tables
    /// wrapped as a data contract. Named rather than reached by index into
    /// `candidateSuites`, because `script/Build.sol` emits the released-suites
    /// lib from THIS candidate specifically, and naming it keeps the suite key,
    /// the artifact path and the dependency list spelled once.
    ///
    /// There is no Solidity contract behind it, so it declares an
    /// `unanchorableReason` and the anchor check holds it to having no
    /// artifact. Its artifact path names the file the tables are generated
    /// into, which declares no contract. The creation code it is snapshotted
    /// from is `logTablesCreationCode`.
    ///
    /// A data contract's runtime is data behind a `STOP`; it calls nothing and
    /// has no dependency that must already be deployed.
    /// @return The candidate.
    function logTablesCandidate() internal pure returns (DeployCandidate memory) {
        return DeployCandidate({
            snapshot: DeploySuite({
                suite: "log-tables",
                creationCode: LOG_TABLES_CREATION_CODE_CANDIDATE,
                storedDeployedAddress: LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS,
                storedBytecodeHash: LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH,
                storedRuntimeCode: LOG_TABLES_RUNTIME_CODE_CANDIDATE,
                artifactPath: "src/generated/LogTables.bytes.sol:LogTables",
                dependencies: new DeployDependency[](0)
            }),
            unanchorableReason: "A data contract that LibDataContract assembles from the generated log tables; no compiler produces it."
        });
    }

    /// The creation code the log-tables snapshot is cut from:
    /// `LibDataContract`'s data-contract wrapper around the bytes
    /// `LibDecimalFloatDeploy.combinedTables()` concatenates out of
    /// `src/generated/LogTables.bytes.sol`.
    /// @return The creation code.
    function logTablesCreationCode() internal pure returns (bytes memory) {
        return LibDataContract.contractCreationCode(LibDecimalFloatDeploy.combinedTables());
    }

    /// This repo's rolling `DecimalFloat` candidate, named for the same reason
    /// the log-tables one is.
    ///
    /// The log tables are a hard dependency rather than a convenience: the
    /// constructor calls `LibDecimalFloatDeploy.checkLogTablesDeployed()`, so
    /// broadcasting `decimal-float` onto a chain without them does not produce
    /// a degraded deployment, it reverts. Recording the address and runtime
    /// code here is what makes `LibRainDeploy` refuse that chain before it
    /// broadcasts.
    /// @return The candidate.
    function decimalFloatCandidate() internal pure returns (DeployCandidate memory) {
        DeployDependency[] memory dependencies = new DeployDependency[](1);
        dependencies[0] = DeployDependency({
            deployedAddress: LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS,
            runtimeCode: LOG_TABLES_RUNTIME_CODE_CANDIDATE
        });
        return DeployCandidate({
            snapshot: DeploySuite({
                suite: "decimal-float",
                creationCode: DECIMAL_FLOAT_CREATION_CODE_CANDIDATE,
                storedDeployedAddress: LibDecimalFloatDeploy.ZOLTU_DEPLOYED_DECIMAL_FLOAT_ADDRESS,
                storedBytecodeHash: LibDecimalFloatDeploy.DECIMAL_FLOAT_CONTRACT_HASH,
                storedRuntimeCode: DECIMAL_FLOAT_RUNTIME_CODE_CANDIDATE,
                artifactPath: "src/concrete/DecimalFloat.sol:DecimalFloat",
                dependencies: dependencies
            }),
            unanchorableReason: ""
        });
    }
}
