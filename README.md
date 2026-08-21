# rain.math.float.deploy

The **deployment** half of
[`rain.math.float`](https://github.com/rainlanguage/rain.math.float): the
concrete `DecimalFloat` contract, its deterministic Zoltu deploy pins
(`src/lib/deploy/LibDecimalFloatDeploy.sol`), the frozen log-tables snapshot
(`src/generated/LogTables.pointers.sol`), and the deploy scripts + tests.

The **library** half — `LibDecimalFloat`, `LibDecimalFloatImplementation`,
`LibFormatDecimalFloat`, `LibParseDecimalFloat`, `LibLogTable` and the errors —
lives in `rain.math.float` and is imported here as the `rain-math-float` Soldeer
package. Consumers that need only the pure math depend on `rain-math-float`;
consumers that need the deployed address, codehash or the deploy pins
(`LibDecimalFloatDeploy.ZOLTU_DEPLOYED_*`) depend on `rain-math-float-deploy`.

## The deploy surface

- `src/concrete/DecimalFloat.sol` — the deployed contract. Wraps the library and
  reads the log tables from the Zoltu address; its constructor asserts the
  tables are present via `LibDecimalFloatDeploy.checkLogTablesDeployed()`.
- `src/lib/deploy/LibDecimalFloatDeploy.sol` — the pins: the log-tables and
  DecimalFloat Zoltu addresses + codehashes, and the per-release frozen suites
  the tagged-constants test enforces.
- `src/generated/LogTables.pointers.sol` — the AOT-compiled log/anti-log table
  bytes, regenerated (not hand-written) by `script/Build.sol`. The bytes are a
  pure function of `LibLogTable`, so this snapshot is version-invariant. NEVER
  edit by hand.
- `script/Deploy.sol` — deploys either the `log-tables` suite or the
  `decimal-float` suite via the Zoltu deterministic deployer.

## Conventions

- Concrete, scripts and tests pin `=0.8.25`; the shipped deploy lib floats
  `^0.8.25`. Optimizer on at **1,000,000 runs** (see `foundry.toml`) — this is
  what the live deployment used, and the pins move if it changes.
- Cancun, no CBOR metadata. Soldeer deps carry the version in the import path;
  `recursive_deps` is off, so every transitively reached package is declared.

## Releases

Releases are manual `sol-v*` tags, never merges. `package-release.yaml` runs
`rainix-tag-release`, which regenerates the pointers snapshot, verifies the live
chains against the pins and publishes `rain-math-float-deploy` to Soldeer. The
on-chain deploy is separate and human-dispatched, run BEFORE tagging: the
`Manual sol artifacts` workflow runs `script/Deploy.sol`. The pinned addresses
are deterministic (Zoltu), so they are fixed before any broadcast and do not
move between releases — but a pin records what the bytecode deploys to, not that
it has been deployed. `LibDecimalFloatDeployProdTest` is what asserts a given
chain actually carries the code.

See rainlanguage/rain.factory#46 for the library/deploy split rationale.
