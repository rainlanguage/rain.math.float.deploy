// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {
    LOG_TABLES,
    LOG_TABLES_SMALL,
    LOG_TABLES_SMALL_ALT,
    ANTI_LOG_TABLES,
    ANTI_LOG_TABLES_SMALL
} from "../../generated/LogTables.pointers.sol";
import {
    DEPLOYED_ADDRESS as LOG_TABLES_CANDIDATE_ADDRESS,
    BYTECODE_HASH as LOG_TABLES_CANDIDATE_HASH
} from "../../generated/candidate/LogTables.sol";
import {
    DEPLOYED_ADDRESS as DECIMAL_FLOAT_CANDIDATE_ADDRESS,
    BYTECODE_HASH as DECIMAL_FLOAT_CANDIDATE_HASH
} from "../../generated/candidate/DecimalFloat.sol";
import {LOG_TABLE_DISAMBIGUATOR} from "rain-math-float-0.1.7/src/lib/table/LibLogTable.sol";
import {LogTablesNotDeployed} from "rain-math-float-0.1.7/src/error/ErrDecimalFloat.sol";

/// @title LibDecimalFloatDeploy
/// @notice The consumer-facing deploy surface of this package: the Zoltu
/// addresses and code hashes of the log tables and of `DecimalFloat`, plus the
/// two helpers that read them.
///
/// The four pins are ALIASES of the rolling snapshots under
/// `src/generated/candidate/`, which `script/Build.sol` writes by running each
/// creation code through the Zoltu factory. The snapshot is the single source
/// of truth and this file is the stable import path over it, so a consumer's
/// import never moves and nothing here can drift from what the repo compiles.
/// A frozen release is NOT aliased here — released pins live in their own
/// `src/generated/<tag>/` snapshot and reach `releasedSuites()` through the
/// generated `LibReleasedSuites`, so the names below always mean "current".
library LibDecimalFloatDeploy {
    /// @dev Address of the log tables deployed via Zoltu's deterministic
    /// deployment proxy. This address is the same across all EVM-compatible
    /// networks.
    address constant ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS = LOG_TABLES_CANDIDATE_ADDRESS;

    /// @dev The expected codehash of the log tables deployed via Zoltu's
    /// deterministic deployment proxy.
    bytes32 constant LOG_TABLES_DATA_CONTRACT_HASH = LOG_TABLES_CANDIDATE_HASH;

    /// @dev Address of the DecimalFloat contract deployed via Zoltu's
    /// deterministic deployment proxy.
    /// This address is the same across all EVM-compatible networks.
    address constant ZOLTU_DEPLOYED_DECIMAL_FLOAT_ADDRESS = DECIMAL_FLOAT_CANDIDATE_ADDRESS;

    /// @dev The expected codehash of the DecimalFloat contract deployed via
    /// Zoltu's deterministic deployment proxy.
    bytes32 constant DECIMAL_FLOAT_CONTRACT_HASH = DECIMAL_FLOAT_CANDIDATE_HASH;

    /// Combines all log and anti-log tables into a single bytes array for
    /// deployment. These are using packed encoding to minimize size and remove
    /// the complexity of full ABI encoding.
    /// @return The combined tables.
    function combinedTables() internal pure returns (bytes memory) {
        return abi.encodePacked(
            LOG_TABLES,
            LOG_TABLES_SMALL,
            LOG_TABLES_SMALL_ALT,
            ANTI_LOG_TABLES,
            ANTI_LOG_TABLES_SMALL,
            LOG_TABLE_DISAMBIGUATOR
        );
    }

    /// Revert if the log tables data contract is not deployed at the
    /// Zoltu-deterministic address with the expected codehash. Call this
    /// from the constructor of any contract that integrates with the
    /// production `DecimalFloat` (or otherwise reads from
    /// `ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS`) so deployment fails loudly on
    /// chains where Zoltu has not dropped the tables, instead of silent
    /// `extcodecopy`-from-empty corruption at the first transcendental call.
    function checkLogTablesDeployed() internal view {
        bytes32 actualCodehash = ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS.codehash;
        if (actualCodehash != LOG_TABLES_DATA_CONTRACT_HASH) {
            revert LogTablesNotDeployed(
                ZOLTU_DEPLOYED_LOG_TABLES_ADDRESS, LOG_TABLES_DATA_CONTRACT_HASH, actualCodehash
            );
        }
    }
}
