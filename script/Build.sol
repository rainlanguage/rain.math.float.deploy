// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {BuildScript} from "rain-deploy-0.1.25/src/abstract/BuildScript.sol";
import {LibRainDeploySnapshot} from "rain-deploy-0.1.25/src/lib/LibRainDeploySnapshot.sol";
import {DeployCandidate, DeploySuite} from "../src/abstract/RainDeploySuitesBase.sol";
import {DecimalFloatDeploySuites} from "../src/abstract/DecimalFloatDeploySuites.sol";

/// One contract's released record: the released-suites lib emitted from its
/// frozen snapshots.
struct ReleasedContract {
    /// Names the snapshot file inside each `src/generated/<tag>/` and the
    /// generated released-suites lib.
    string contractName;
    /// The suite the released entries take their key and artifact path from.
    DeploySuite template;
}

/// @title Build
/// @notice Generates the deploy pins for every contract this repo deploys.
///
/// `run()` (what CI regenerates against) rewrites the rolling
/// `src/generated/candidate/` snapshot and the released-suites libs.
/// `cutRelease()` freezes the candidate into `src/generated/<tag>/` first.
/// Frozen snapshots are append-only historical records, never regenerated here.
///
/// Two lists, because the record holds a contract this repo no longer builds.
/// `snapshotContractNames()` is what is regenerated and frozen: `DecimalFloat`
/// alone. `releasedContracts()` is what has a released-suites lib: that, plus
/// the `LogTables` data contract frozen by the releases that shipped it, whose
/// entries stay in `releasedSuites()` so every frozen snapshot stays released.
///
/// There is no generated alias lib. `src/lib/deploy/LibDecimalFloatDeploy.sol`
/// is the stable import path over the candidate snapshot and is hand-written,
/// so it aliases the snapshot by import instead, which is the same single
/// source of truth by a different spelling, and
/// `LibDecimalFloatDeployCandidateTest` pins that it still does.
contract Build is BuildScript, DecimalFloatDeploySuites {
    /// Every contract with a released-suites lib, in the order the aggregate
    /// emits its entries in.
    /// @return The released contracts.
    function releasedContracts() internal pure returns (ReleasedContract[] memory) {
        ReleasedContract[] memory contracts = new ReleasedContract[](2);
        contracts[0] = ReleasedContract({contractName: "LogTables", template: releasedLogTablesTemplate()});
        contracts[1] = ReleasedContract({contractName: "DecimalFloat", template: decimalFloatCandidate().snapshot});
        return contracts;
    }

    /// @inheritdoc BuildScript
    function snapshotContractNames() internal pure override returns (string[] memory) {
        string[] memory names = new string[](1);
        names[0] = "DecimalFloat";
        return names;
    }

    /// @inheritdoc BuildScript
    function regenerateSnapshots() internal override {
        DeployCandidate memory decimalFloat = decimalFloatCandidate();
        LibRainDeploySnapshot.writeSnapshot(
            vm,
            recordRoot(),
            LibRainDeploySnapshot.CANDIDATE,
            "DecimalFloat",
            vm.getCode(decimalFloat.snapshot.artifactPath),
            decimalFloat.snapshot.dependencies
        );
    }

    /// @inheritdoc BuildScript
    /// @dev Every released-suites lib and the aggregate over them.
    function regenerateLibs() internal override {
        ReleasedContract[] memory contracts = releasedContracts();
        string[] memory names = new string[](contracts.length);
        for (uint256 i = 0; i < contracts.length; i++) {
            LibRainDeploySnapshot.writeReleasedSuitesLib(
                vm, LibRainDeploySnapshot.LIB_DIR, recordRoot(), contracts[i].contractName, contracts[i].template
            );
            names[i] = contracts[i].contractName;
        }
        LibRainDeploySnapshot.writeReleasedSuitesAggregate(vm, LibRainDeploySnapshot.LIB_DIR, names);
    }
}
