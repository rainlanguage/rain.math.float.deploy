// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Test} from "forge-std-1.16.2/src/Test.sol";

/// @title LibDecimalFloatDeployTaggedConstantsTest
/// @notice Every version published to the soldeer registry for `rain-math-float-deploy`
/// must have a full suite of pinned deploy constants in `LibDecimalFloatDeploy`:
/// a log-tables address + codehash and a DecimalFloat address + codehash for
/// each published version. `script/check-published-deploy-constants.sh` queries
/// the live registry (via FFI) and lists any missing constants, so publishing a
/// new tag without pinning its constants fails this test. Skips if the registry
/// is unreachable rather than failing on network flakiness.
contract LibDecimalFloatDeployTaggedConstantsTest is Test {
    function testAllPublishedSoldeerTagsHaveAFullConstantSuite() external {
        string[] memory cmd = new string[](2);
        cmd[0] = "bash";
        cmd[1] = "script/check-published-deploy-constants.sh";
        bytes memory out = vm.ffi(cmd);

        // "OK" means every published tag has its full pinned constant suite;
        // "SKIP" means the registry was unreachable (offline dev — nothing to
        // verify). Either passes. On a real gap the value is the list of missing
        // `*_<version>` constants, which fails here.
        assertTrue(
            _startsWith(out, bytes("OK")) || _startsWith(out, bytes("SKIP")),
            string.concat("a published soldeer tag is missing pinned deploy constants: ", string(out))
        );
    }

    function _startsWith(bytes memory s, bytes memory prefix) private pure returns (bool) {
        if (s.length < prefix.length) {
            return false;
        }
        for (uint256 i = 0; i < prefix.length; i++) {
            if (s[i] != prefix[i]) {
                return false;
            }
        }
        return true;
    }
}
