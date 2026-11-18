// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {BuildScript, GeneratedContract, RetiredContract} from "rain-deploy-0.1.35/src/abstract/BuildScript.sol";
import {LibRainDeploySnapshot} from "rain-deploy-0.1.35/src/lib/LibRainDeploySnapshot.sol";
import {DeployDeclaration} from "../src/abstract/RainDeploySuitesBase.sol";
import {DecimalFloatDeploySuites} from "../src/abstract/DecimalFloatDeploySuites.sol";

/// @title Build
/// @notice Generates the deploy pins for every contract this repo deploys.
///
/// `run()` (what CI regenerates against) rewrites the rolling
/// `src/generated/candidate/` snapshot and the released-suites libs.
/// `cutRelease()` freezes the candidate into `src/generated/<tag>/` first.
/// Frozen snapshots are append-only historical records, never regenerated here.
///
/// `DecimalFloat` is the one generated contract. `LogTables` is retired: a data
/// contract frozen by the releases that shipped it, whose entries stay released.
///
/// There is no generated alias lib. `src/lib/deploy/LibDecimalFloatDeploy.sol`
/// is the stable import path over the candidate snapshot and is hand-written,
/// so it aliases the snapshot by import instead, and
/// `LibDecimalFloatDeployCandidateTest` pins that it still does.
contract Build is BuildScript, DecimalFloatDeploySuites {
    /// @inheritdoc BuildScript
    function generatedContracts(DeployDeclaration memory d)
        internal
        pure
        override
        returns (GeneratedContract[] memory)
    {
        GeneratedContract[] memory contracts = new GeneratedContract[](1);
        contracts[0] = GeneratedContract({
            contractName: "DecimalFloat",
            constantPrefix: "",
            candidate: declaredCandidate(d, decimalFloatCandidate().snapshot.suite)
        });
        return contracts;
    }

    /// @inheritdoc BuildScript
    function retiredContracts() internal pure override returns (RetiredContract[] memory) {
        RetiredContract[] memory retired = new RetiredContract[](1);
        retired[0] = RetiredContract({contractName: "LogTables", template: releasedLogTablesTemplate()});
        return retired;
    }

    /// @inheritdoc BuildScript
    function regenerateSnapshots() internal override {
        GeneratedContract[] memory contracts = checkedGeneratedContracts(declaration());
        for (uint256 i = 0; i < contracts.length; i++) {
            LibRainDeploySnapshot.writeSnapshot(
                vm,
                recordRoot(),
                LibRainDeploySnapshot.CANDIDATE,
                contracts[i].contractName,
                vm.getCode(contracts[i].candidate.snapshot.artifactPath),
                contracts[i].candidate.snapshot.dependencies
            );
        }
    }
}
