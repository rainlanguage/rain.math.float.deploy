// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.17.0/src/Test.sol";
import {DeployDependency, LibRainDeploy} from "rain-deploy-0.1.18/src/lib/LibRainDeploy.sol";
import {
    BYTECODE_HASH as DECIMAL_FLOAT_BYTECODE_HASH,
    DEPLOYED_ADDRESS as DECIMAL_FLOAT_DEPLOYED_ADDRESS,
    CREATION_CODE as DECIMAL_FLOAT_CREATION_CODE,
    RUNTIME_CODE as DECIMAL_FLOAT_RUNTIME_CODE,
    DEPENDENCIES as DECIMAL_FLOAT_DEPENDENCIES
} from "src/generated/candidate/DecimalFloat.sol";
import {LibDecimalFloatDeploy} from "src/lib/deploy/LibDecimalFloatDeploy.sol";
import {DecimalFloat} from "src/concrete/DecimalFloat.sol";

/// @title LibDecimalFloatDeployCandidateTest
/// @notice The rolling `src/generated/candidate/` snapshot and the
/// `LibDecimalFloatDeploy` pins must stay consistent end to end:
/// SOURCE -> CANDIDATE pins -> ALIAS. CI currency-checks that
/// `forge script ./script/Build.sol` regenerates the candidate
/// byte-identically; these assertions pin what that regeneration must hold
/// true.
///
/// The alias half is what replaces the generated alias lib every other deploy
/// repo carries: `LibDecimalFloatDeploy` is hand-written, so nothing overwrites
/// its pins — this is what says they are still the candidate's.
contract LibDecimalFloatDeployCandidateTest is Test {
    /// SOURCE -> CANDIDATE: the candidate records THIS repo's current
    /// `DecimalFloat` creation code.
    function testDecimalFloatCandidateCreationCodeMatchesSource() external pure {
        assertEq(
            DECIMAL_FLOAT_CREATION_CODE,
            type(DecimalFloat).creationCode,
            "decimal float candidate creation code is not current source"
        );
    }

    /// CANDIDATE self-consistency: Zoltu-deploying the recorded
    /// `CREATION_CODE` lands at the recorded `DEPLOYED_ADDRESS` with the
    /// recorded runtime code and code hash, and `keccak256(RUNTIME_CODE)` is
    /// that hash.
    function testDecimalFloatCandidateReproducesItsDeployment() external {
        LibRainDeploy.etchZoltuFactory(vm);

        address deployed = LibRainDeploy.deployZoltu(DECIMAL_FLOAT_CREATION_CODE);
        assertEq(deployed, DECIMAL_FLOAT_DEPLOYED_ADDRESS, "creation code deploys to a different address");
        assertEq(deployed.code, DECIMAL_FLOAT_RUNTIME_CODE, "deployed runtime code is not the recorded runtime code");
        assertEq(deployed.codehash, DECIMAL_FLOAT_BYTECODE_HASH, "deployed code hash is not the recorded hash");
        assertEq(
            keccak256(DECIMAL_FLOAT_RUNTIME_CODE),
            DECIMAL_FLOAT_BYTECODE_HASH,
            "recorded runtime code does not hash to the recorded hash"
        );
    }

    /// CANDIDATE -> ALIAS: the shipped `LibDecimalFloatDeploy` re-exports the
    /// candidate's address and code hash unchanged, so consumers pin the
    /// candidate through a stable import path.
    function testAliasesReExportTheCandidate() external pure {
        assertEq(
            LibDecimalFloatDeploy.ZOLTU_DEPLOYED_DECIMAL_FLOAT_ADDRESS,
            DECIMAL_FLOAT_DEPLOYED_ADDRESS,
            "decimal float alias address is not the candidate address"
        );
        assertEq(
            LibDecimalFloatDeploy.DECIMAL_FLOAT_CONTRACT_HASH,
            DECIMAL_FLOAT_BYTECODE_HASH,
            "decimal float alias code hash is not the candidate hash"
        );
    }

    /// `DecimalFloat` reads no other contract, so nothing has to be on a
    /// network before it can be broadcast there and the recorded dependency
    /// list is empty.
    function testDecimalFloatCandidateHasNoDependencies() external pure {
        DeployDependency[] memory dependencies = abi.decode(DECIMAL_FLOAT_DEPENDENCIES, (DeployDependency[]));
        assertEq(dependencies.length, 0, "decimal float candidate records unexpected deploy dependencies");
    }
}
