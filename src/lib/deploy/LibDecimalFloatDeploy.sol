// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity ^0.8.25;

import {
    DEPLOYED_ADDRESS as DECIMAL_FLOAT_CANDIDATE_ADDRESS,
    BYTECODE_HASH as DECIMAL_FLOAT_CANDIDATE_HASH
} from "../../generated/candidate/DecimalFloat.sol";

/// @title LibDecimalFloatDeploy
/// @notice The consumer-facing deploy surface of this package: the Zoltu
/// address and code hash of `DecimalFloat`.
///
/// The two pins are ALIASES of the rolling snapshot under
/// `src/generated/candidate/`, which `script/Build.sol` writes by running the
/// creation code through the Zoltu factory. The snapshot is the single source
/// of truth and this file is the stable import path over it, so a consumer's
/// import never moves and nothing here can drift from what the repo compiles.
/// A frozen release is NOT aliased here — released pins live in their own
/// `src/generated/<tag>/` snapshot and reach `releasedSuites()` through the
/// generated `LibReleasedSuites`, so the names below always mean "current".
library LibDecimalFloatDeploy {
    /// @dev Address of the DecimalFloat contract deployed via Zoltu's
    /// deterministic deployment proxy.
    /// This address is the same across all EVM-compatible networks.
    address constant ZOLTU_DEPLOYED_DECIMAL_FLOAT_ADDRESS = DECIMAL_FLOAT_CANDIDATE_ADDRESS;

    /// @dev The expected codehash of the DecimalFloat contract deployed via
    /// Zoltu's deterministic deployment proxy.
    bytes32 constant DECIMAL_FLOAT_CONTRACT_HASH = DECIMAL_FLOAT_CANDIDATE_HASH;
}
