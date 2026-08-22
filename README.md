# rain.math.float.deploy

The **deployment** half of
[`rain.math.float`](https://github.com/rainlanguage/rain.math.float): the
concrete `DecimalFloat` contract, the rolling `src/generated/candidate/`
snapshots of its deterministic Zoltu deploy records (address, codehash, creation
and runtime bytecode), the alias lib `src/lib/deploy/LibDecimalFloatDeploy.sol`
over those pins, the generated log-tables bytes
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
- `src/abstract/DecimalFloatDeploySuites.sol` — everything this repo deploys,
  declared once: the `log-tables` and `decimal-float` candidate suites that
  `script/Build.sol`, `script/Deploy.sol` and the deploy tests all bind to.
- `src/generated/candidate/{LogTables,DecimalFloat}.sol` — the rolling deploy
  records (address, codehash, creation and runtime bytecode), rewritten from
  what source compiles to by `script/Build.sol` and currency-checked by CI. A
  release cut freezes them into `src/generated/<tag>/`. NEVER edit by hand.
- `src/lib/deploy/LibDecimalFloatDeploy.sol` — the stable import path over the
  candidate pins (the `ZOLTU_DEPLOYED_*` addresses + codehashes are aliases of
  the generated constants), plus the hand-written `combinedTables()` and
  `checkLogTablesDeployed()`.
- `src/lib/LibReleasedSuites.sol` (+ the per-contract `Lib*Released.sol`) — the
  generated record of every released suite, currently empty because nothing is
  released yet. NEVER edit by hand.
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

Releases are manual `sol-v*` tags, never merges.

The on-chain deploy comes first and is human-dispatched: the
`Manual sol artifacts` workflow runs `script/Deploy.sol` for the `log-tables`
suite, then again for `decimal-float` (its constructor reverts unless the tables
are already on-chain). Where the addresses already exist (deterministic Zoltu),
a deploy attests existing code rather than deploying fresh.

Cutting the release is then a PR that runs
`forge script ./script/Build.sol --sig 'cutRelease()'` + `forge fmt`, freezing
the candidate snapshots into `src/generated/<tag>/` and regenerating the
released-suites libs, with `[external.package].version` naming that tag. After
merge, pushing the `sol-v*` tag runs `rainix-tag-release`
(`package-release.yaml`), which re-runs the non-freezing build, requires a clean
tree with the frozen snapshot byte-identical to the candidate, verifies the live
chains against the record and publishes `rain-math-float-deploy` to Soldeer.

See rainlanguage/rain.factory#46 for the library/deploy split rationale.
