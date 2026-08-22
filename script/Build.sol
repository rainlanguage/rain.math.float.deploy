// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {BuildScript} from "rain-deploy-0.1.7/src/abstract/BuildScript.sol";
import {LibCodeGen} from "rain-sol-codegen-0.1.36/src/lib/LibCodeGen.sol";
import {LibLogTable} from "rain-math-float-0.1.7/src/lib/table/LibLogTable.sol";
import {LibRainDeploySnapshot} from "rain-deploy-0.1.7/src/lib/LibRainDeploySnapshot.sol";
import {DeployCandidate} from "../src/abstract/RainDeploySuitesBase.sol";
import {DecimalFloatDeploySuites} from "../src/abstract/DecimalFloatDeploySuites.sol";

/// @dev Committed path of the generated log tables. The `.pointers.sol` suffix
/// is the one the deploy pins and importers reference. The file is pure
/// log-table data with no contract instance behind it, so it carries no
/// bytecode-hash constant and is written directly rather than through
/// `LibFs.buildFileForContract`, which heads every file it writes with the hash
/// of an instance and reverts on the codeless `address(0)` this build has.
///
/// It sits at the root of `src/generated/` rather than in a snapshot directory
/// because it is not a deploy record: it holds the table BYTES, which are a
/// pure function of `LibLogTable` and carry no address, code hash or tag. The
/// deploy record of the data contract those bytes are wrapped in is
/// `src/generated/candidate/LogTables.sol`, and only tag-shaped directories
/// under this root are releases — a loose file here is one neither
/// `frozenSnapshotPaths` nor `release-guard` reads as a snapshot.
string constant GENERATED_LOG_TABLES = "src/generated/LogTables.pointers.sol";

/// One contract's generated files: the rolling snapshot and the released-suites
/// lib emitted from its record.
struct GeneratedContract {
    /// Places the snapshot inside `src/generated/<dir>/` and names the
    /// generated released-suites lib.
    string contractName;
    /// Snapshots are written from its `sourceCreationCode` and
    /// `snapshot.dependencies`; the released lib takes its suite key and
    /// artifact path from its `snapshot`.
    DeployCandidate candidate;
}

/// @title Build
/// @notice Generates the deploy pins for every contract this repo deploys, plus
/// the log-table bytes those pins are computed over. `generatedContracts()` is
/// the only list, read by every hook below.
///
/// `run()` (what CI regenerates against) rewrites the log tables, the rolling
/// `src/generated/candidate/` snapshots and the released-suites libs.
/// `cutRelease()` freezes the candidates into `src/generated/<tag>/` first.
/// Frozen snapshots are append-only historical records, never regenerated here.
///
/// There is no generated alias lib. `src/lib/deploy/LibDecimalFloatDeploy.sol`
/// is the stable import path over the candidate snapshots and it also carries
/// hand-written logic — `combinedTables()` and `checkLogTablesDeployed()` — so
/// it cannot be a file the generator overwrites. It aliases the snapshots by
/// import instead, which is the same single source of truth by a different
/// spelling, and `LibDecimalFloatDeployCandidateTest` pins that it still does.
contract Build is BuildScript, DecimalFloatDeploySuites {
    /// Every contract this repo generates deploy pins for.
    /// @return The generated contracts.
    function generatedContracts() internal pure returns (GeneratedContract[] memory) {
        GeneratedContract[] memory contracts = new GeneratedContract[](2);
        contracts[0] = GeneratedContract({contractName: "LogTables", candidate: logTablesCandidate()});
        contracts[1] = GeneratedContract({contractName: "DecimalFloat", candidate: decimalFloatCandidate()});
        return contracts;
    }

    /// @inheritdoc BuildScript
    /// @dev In declaration order — the order the aggregate emits its entries
    /// in.
    function snapshotContractNames() internal pure override returns (string[] memory) {
        GeneratedContract[] memory contracts = generatedContracts();
        string[] memory names = new string[](contracts.length);
        for (uint256 i = 0; i < contracts.length; i++) {
            names[i] = contracts[i].contractName;
        }
        return names;
    }

    /// Rewrites `src/generated/LogTables.pointers.sol` from `LibLogTable`.
    ///
    /// The bytes are a pure function of that library's mathematics, so this
    /// file is a single snapshot rather than a per-tag directory: there is no
    /// version of it to freeze, only a currency check that the committed bytes
    /// are the ones the pinned `rain-math-float` computes.
    function regenerateLogTables() internal {
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

    /// @inheritdoc BuildScript
    /// @dev The log tables are written first, and the log-tables snapshot
    /// before the `DecimalFloat` one, because both orderings are load bearing:
    ///
    /// - The tables are what the log-tables data contract is built out of, and
    ///   a freeze copies whatever this hook leaves behind, so regenerating them
    ///   anywhere but here would let a release freeze a snapshot cut from stale
    ///   tables. They are written from `LibLogTable` while the snapshot below
    ///   is cut from the COMPILED-IN `LogTables.pointers.sol`, so a run that
    ///   actually moves the tables converges on the second run — which is what
    ///   the currency check in CI reports rather than hides.
    /// - `writeSnapshot` runs each creation code through the Zoltu factory to
    ///   read back its runtime code, and `DecimalFloat`'s constructor reverts
    ///   unless the tables are already at their Zoltu address. Cutting the
    ///   log-tables snapshot first puts them there, in this same VM, for the
    ///   same reason a broadcast has to deploy `log-tables` first.
    function regenerateSnapshots() internal override {
        regenerateLogTables();

        GeneratedContract[] memory contracts = generatedContracts();
        for (uint256 i = 0; i < contracts.length; i++) {
            LibRainDeploySnapshot.writeSnapshot(
                vm,
                LibRainDeploySnapshot.CANDIDATE,
                contracts[i].contractName,
                contracts[i].candidate.sourceCreationCode,
                contracts[i].candidate.snapshot.dependencies
            );
        }
    }

    /// @inheritdoc BuildScript
    /// @dev Every released-suites lib and the aggregate over them.
    function regenerateLibs() internal override {
        GeneratedContract[] memory contracts = generatedContracts();
        for (uint256 i = 0; i < contracts.length; i++) {
            LibRainDeploySnapshot.writeReleasedSuitesLib(
                vm,
                LibRainDeploySnapshot.LIB_DIR,
                recordRoot(),
                contracts[i].contractName,
                contracts[i].candidate.snapshot
            );
        }
        LibRainDeploySnapshot.writeReleasedSuitesAggregate(vm, LibRainDeploySnapshot.LIB_DIR, snapshotContractNames());
    }
}
