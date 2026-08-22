// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.2/src/Test.sol";
import {LibRainDeploy} from "rain-deploy-0.1.7/src/lib/LibRainDeploy.sol";
import {LibDataContract} from "rain-datacontract-0.1.0/src/lib/LibDataContract.sol";
import {
    BYTECODE_HASH as LOG_TABLES_BYTECODE_HASH,
    DEPLOYED_ADDRESS as LOG_TABLES_DEPLOYED_ADDRESS,
    CREATION_CODE as LOG_TABLES_CREATION_CODE,
    RUNTIME_CODE as LOG_TABLES_RUNTIME_CODE,
    DEPENDENCIES as LOG_TABLES_DEPENDENCIES
} from "src/generated/candidate/LogTables.sol";
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
/// @notice The rolling `src/generated/candidate/` snapshots and the
/// `LibDecimalFloatDeploy` pins must stay consistent end to end, for both
/// deployed contracts: SOURCE -> CANDIDATE pins -> ALIAS. CI currency-checks
/// that `forge script ./script/Build.sol` regenerates the candidates
/// byte-identically; these assertions pin what that regeneration must hold
/// true.
///
/// The alias half is what replaces the generated alias lib every other deploy
/// repo carries. `LibDecimalFloatDeploy` is hand-written because it also holds
/// `combinedTables()` and `checkLogTablesDeployed()`, so nothing overwrites its
/// four pins — this is what says they are still the candidate's.
contract LibDecimalFloatDeployCandidateTest is Test {
    /// SOURCE -> CANDIDATE, log tables: the candidate records the data-contract
    /// creation code this repo's own `combinedTables()` produces, so the pins
    /// describe the tables in `src/generated/LogTables.pointers.sol` rather
    /// than a stale payload.
    function testLogTablesCandidateCreationCodeMatchesSource() external pure {
        assertEq(
            LOG_TABLES_CREATION_CODE,
            LibDataContract.contractCreationCode(LibDecimalFloatDeploy.combinedTables()),
            "log tables candidate creation code is not current source"
        );
    }

    /// SOURCE -> CANDIDATE, `DecimalFloat`: the candidate records THIS repo's
    /// current `DecimalFloat` creation code.
    function testDecimalFloatCandidateCreationCodeMatchesSource() external pure {
        assertEq(
            DECIMAL_FLOAT_CREATION_CODE,
            type(DecimalFloat).creationCode,
            "decimal float candidate creation code is not current source"
        );
    }

    /// CANDIDATE self-consistency, log tables: Zoltu-deploying the recorded
    /// `CREATION_CODE` lands at the recorded `DEPLOYED_ADDRESS` with the
    /// recorded runtime code and code hash, and `keccak256(RUNTIME_CODE)` is
    /// that hash.
    function testLogTablesCandidateReproducesItsDeployment() external {
        LibRainDeploy.etchZoltuFactory(vm);
        address deployed = LibRainDeploy.deployZoltu(LOG_TABLES_CREATION_CODE);
        assertEq(deployed, LOG_TABLES_DEPLOYED_ADDRESS, "creation code deploys to a different address");
        assertEq(deployed.code, LOG_TABLES_RUNTIME_CODE, "deployed runtime code is not the recorded runtime code");
        assertEq(deployed.codehash, LOG_TABLES_BYTECODE_HASH, "deployed code hash is not the recorded hash");
        assertEq(
            keccak256(LOG_TABLES_RUNTIME_CODE),
            LOG_TABLES_BYTECODE_HASH,
            "recorded runtime code does not hash to the recorded hash"
        );
    }

    /// CANDIDATE self-consistency, `DecimalFloat`. The log tables are deployed
    /// through the factory first because the constructor reverts without them,
    /// which is the same ordering the broadcast is held to.
    function testDecimalFloatCandidateReproducesItsDeployment() external {
        LibRainDeploy.etchZoltuFactory(vm);
        LibRainDeploy.deployZoltu(LOG_TABLES_CREATION_CODE);

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

    /// CANDIDATE -> ALIAS: the shipped `LibDecimalFloatDeploy` re-exports both
    /// candidates' addresses and code hashes unchanged, so consumers pin the
    /// candidates through a stable import path.
    function testAliasesReExportTheCandidates() external pure {
        assertEq(
            LibDecimalFloatDeploy.ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS,
            LOG_TABLES_DEPLOYED_ADDRESS,
            "log tables alias address is not the candidate address"
        );
        assertEq(
            LibDecimalFloatDeploy.LOG_TABLES_DATA_CONTRACT_HASH,
            LOG_TABLES_BYTECODE_HASH,
            "log tables alias code hash is not the candidate hash"
        );
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

    /// A data contract's runtime is data behind a `STOP`; it calls nothing, so
    /// nothing has to be on a network before the log tables can be broadcast
    /// there and the recorded dependency list is empty.
    function testLogTablesCandidateHasNoDependencies() external pure {
        address[] memory dependencies = abi.decode(LOG_TABLES_DEPENDENCIES, (address[]));
        assertEq(dependencies.length, 0, "log tables candidate records unexpected deploy dependencies");
    }

    /// `DecimalFloat`'s constructor reads the log tables' codehash, so the
    /// tables MUST already be on a network before it is broadcast there. That
    /// precondition is frozen into the snapshot rather than left to the order
    /// somebody dispatches the two suites in.
    function testDecimalFloatCandidateDependsOnTheLogTables() external pure {
        address[] memory dependencies = abi.decode(DECIMAL_FLOAT_DEPENDENCIES, (address[]));
        assertEq(dependencies.length, 1, "decimal float candidate does not record exactly one dependency");
        assertEq(
            dependencies[0], LOG_TABLES_DEPLOYED_ADDRESS, "decimal float candidate's dependency is not the log tables"
        );
    }
}
