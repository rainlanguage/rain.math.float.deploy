// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {Script} from "forge-std-1.16.2/src/Script.sol";
import {LibCodeGen} from "rain-sol-codegen-0.1.36/src/lib/LibCodeGen.sol";
import {LibLogTable} from "rain-math-float-0.1.7/src/lib/table/LibLogTable.sol";

/// @dev Committed path of the generated log tables. The `.pointers.sol` suffix
/// is the one the deploy pins and importers reference. The file is pure
/// log-table data with no contract instance behind it, so it carries no
/// bytecode-hash constant and is written directly rather than through
/// `LibFs.buildFileForContract`, which heads every file it writes with the hash
/// of an instance and reverts on the codeless `address(0)` this build has.
string constant GENERATED_LOG_TABLES = "src/generated/LogTables.pointers.sol";

contract Build is Script {
    function run() external {
        //forge-lint: disable-next-line(unsafe-cheatcode)
        vm.writeFile(
            GENERATED_LOG_TABLES,
            string.concat(
                LibCodeGen.filePrefix(),
                LibCodeGen.bytesConstantString(
                    vm, "/// @dev Log tables.", "LOG_TABLES", LibLogTable.toBytes(LibLogTable.logTableDec())
                ),
                LibCodeGen.bytesConstantString(
                    vm,
                    "/// @dev Log tables small.",
                    "LOG_TABLES_SMALL",
                    LibLogTable.toBytes(LibLogTable.logTableDecSmall())
                ),
                LibCodeGen.bytesConstantString(
                    vm,
                    "/// @dev Log tables small alt.",
                    "LOG_TABLES_SMALL_ALT",
                    LibLogTable.toBytes(LibLogTable.logTableDecSmallAlt())
                ),
                LibCodeGen.bytesConstantString(
                    vm,
                    "/// @dev Anti log tables.",
                    "ANTI_LOG_TABLES",
                    LibLogTable.toBytes(LibLogTable.antiLogTableDec())
                ),
                LibCodeGen.bytesConstantString(
                    vm,
                    "/// @dev Anti log tables small.",
                    "ANTI_LOG_TABLES_SMALL",
                    LibLogTable.toBytes(LibLogTable.antiLogTableDecSmall())
                )
            )
        );
    }
}
