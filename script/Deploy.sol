// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {RainDeployBroadcast} from "rain-deploy-0.1.7/src/abstract/RainDeployBroadcast.sol";
import {DecimalFloatDeploySuites} from "../src/abstract/DecimalFloatDeploySuites.sol";

/// @title Deploy
/// @notice The declaration plus `RainDeployBroadcast`; the `Manual sol
/// artifacts` workflow dispatches the `log-tables` suite and then the
/// `decimal-float` suite.
///
/// The order is enforced rather than remembered: `decimal-float` records the
/// log-tables address as a dependency, and `LibRainDeploy` refuses to broadcast
/// on any network where a dependency has no code. Nothing here plants the
/// tables locally to get a simulation through — a chain without them is a chain
/// this suite must not be deployed to, because `DecimalFloat`'s constructor
/// reverts there.
contract Deploy is DecimalFloatDeploySuites, RainDeployBroadcast {}
