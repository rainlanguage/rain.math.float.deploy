// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RainDeployVerifySnapshot} from "rain-deploy-0.1.25/src/abstract/RainDeployVerifySnapshot.sol";
import {DecimalFloatDeploySuites} from "src/abstract/DecimalFloatDeploySuites.sol";

/// @title DecimalFloatDeploySnapshotTest
/// @notice Binds this repo's declaration to `RainDeployVerifySnapshot`: every
/// deploy-pin assertion over the `log-tables` and `decimal-float` suites that
/// needs no network.
contract DecimalFloatDeploySnapshotTest is DecimalFloatDeploySuites, RainDeployVerifySnapshot {}
