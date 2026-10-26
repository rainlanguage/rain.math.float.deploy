// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {
    DeployCandidate,
    DeployDependency,
    DeployMechanism,
    DeploySuite,
    RainDeploySuitesBase
} from "./RainDeploySuitesBase.sol";
import {
    CREATION_CODE as DECIMAL_FLOAT_CREATION_CODE_CANDIDATE,
    RUNTIME_CODE as DECIMAL_FLOAT_RUNTIME_CODE_CANDIDATE
} from "../generated/candidate/DecimalFloat.sol";
import {LibDecimalFloatDeploy} from "../lib/deploy/LibDecimalFloatDeploy.sol";
import {LibReleasedSuites} from "../lib/LibReleasedSuites.sol";

/// @title DecimalFloatDeploySuites
/// @notice Everything this repo deploys, declared ONCE: the hand-written
/// `decimal-float` candidate below — the key `Manual sol artifacts`
/// dispatches — and the released side read from the generated
/// `LibReleasedSuites`, which `script/Build.sol` emits from the frozen record.
///
/// The released side includes the released `log-tables` suites frozen by the
/// releases that shipped them. No candidate declares log tables: the current
/// `rain-math-float` computes logarithms without them, so a new release
/// freezes `DecimalFloat` alone.
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
    function candidateSuites() internal pure override returns (DeployCandidate[] memory) {
        DeployCandidate[] memory candidates = new DeployCandidate[](1);
        candidates[0] = decimalFloatCandidate();
        return candidates;
    }

    /// This repo's rolling `DecimalFloat` candidate. Named rather than reached
    /// by index into `candidateSuites`, because `script/Build.sol` emits the
    /// released-suites lib from THIS candidate specifically, and naming it
    /// keeps the suite key, the artifact path and the dependency list spelled
    /// once.
    ///
    /// `DecimalFloat` reads no other contract, so it has no dependency that
    /// must already be deployed.
    /// @return The candidate.
    function decimalFloatCandidate() internal pure returns (DeployCandidate memory) {
        return DeployCandidate({
            snapshot: DeploySuite({
                suite: "decimal-float",
                creationCode: DECIMAL_FLOAT_CREATION_CODE_CANDIDATE,
                storedDeployedAddress: LibDecimalFloatDeploy.ZOLTU_DEPLOYED_DECIMAL_FLOAT_ADDRESS,
                storedBytecodeHash: LibDecimalFloatDeploy.DECIMAL_FLOAT_CONTRACT_HASH,
                storedRuntimeCode: DECIMAL_FLOAT_RUNTIME_CODE_CANDIDATE,
                artifactPath: "src/concrete/DecimalFloat.sol:DecimalFloat",
                dependencies: new DeployDependency[](0),
                mechanism: DeployMechanism.Zoltu,
                mechanismData: ""
            }),
            unanchorableReason: ""
        });
    }

    /// The template the released `log-tables` entries take their key and
    /// artifact path from. Those releases were cut while this repo still
    /// generated the tables; their address, code and dependencies are read from
    /// each release's frozen `src/generated/<tag>/LogTables.sol`, and only the
    /// key and artifact path come from here. The artifact path is the one
    /// those releases were cut with: the tables were a data contract that no
    /// compiler produces, so no current source anchors them.
    /// @return The template.
    function releasedLogTablesTemplate() internal pure returns (DeploySuite memory) {
        return DeploySuite({
            suite: "log-tables",
            creationCode: "",
            storedDeployedAddress: address(0),
            storedBytecodeHash: bytes32(0),
            storedRuntimeCode: "",
            artifactPath: "src/generated/LogTables.bytes.sol:LogTables",
            dependencies: new DeployDependency[](0),
            mechanism: DeployMechanism.Zoltu,
            mechanismData: ""
        });
    }
}
