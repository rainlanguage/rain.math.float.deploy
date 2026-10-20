# rain.math.float.deploy

The **deployment** half of
[`rain.math.float`](https://github.com/rainlanguage/rain.math.float): the
concrete `DecimalFloat` contract, the rolling `src/generated/candidate/`
snapshot of its deterministic Zoltu deploy records (address, codehash, creation
and runtime bytecode), the alias lib `src/lib/deploy/LibDecimalFloatDeploy.sol`
over those pins, and the deploy scripts + tests.

The **library** half — `LibDecimalFloat`, `LibDecimalFloatImplementation`,
`LibFormatDecimalFloat`, `LibParseDecimalFloat` and the errors — lives in
`rain.math.float` and is imported here as the `rain-math-float` Soldeer package.
Consumers that need only the pure math depend on `rain-math-float`; consumers
that need the deployed address, codehash or the deploy pins
(`LibDecimalFloatDeploy.ZOLTU_DEPLOYED_*`) depend on `rain-math-float-deploy`.

## The deploy surface

- `src/concrete/DecimalFloat.sol` — the deployed contract. Wraps the library; it
  reads no other contract.
- `src/abstract/DecimalFloatDeploySuites.sol` — everything this repo deploys,
  declared once: the `decimal-float` candidate suite that `script/Build.sol`,
  `script/Deploy.sol` and the deploy tests all bind to.
- `src/generated/candidate/DecimalFloat.sol` — the rolling deploy record
  (address, codehash, creation and runtime bytecode), rewritten from what source
  compiles to by `script/Build.sol` and currency-checked by CI. A release cut
  freezes it into `src/generated/<tag>/`. NEVER edit by hand.
- `src/lib/deploy/LibDecimalFloatDeploy.sol` — the stable import path over the
  candidate pins (the `ZOLTU_DEPLOYED_*` addresses + codehashes are aliases of
  the generated constants).
- `src/lib/LibReleasedSuites.sol` (+ the per-contract `Lib*Released.sol`) — the
  generated record of every released suite, including the `log-tables@<tag>`
  data contracts that releases 0.1.1 to 0.1.3 froze and their `DecimalFloat`
  depends on. NEVER edit by hand.
- `script/Deploy.sol` — deploys the `decimal-float` suite via the Zoltu
  deterministic deployer.

## Conventions

- Concrete, scripts and tests pin `=0.8.25`; the shipped deploy lib floats
  `^0.8.25`. `via_ir` with the optimizer at **399 runs** (see `foundry.toml`),
  the highest count that keeps `DecimalFloat` under EIP-170; the pins move if it
  changes.
- Cancun, no CBOR metadata. Soldeer deps carry the version in the import path;
  `recursive_deps` is off, so every transitively reached package is declared.

## Releases

Releases are manual `sol-v*` tags, never merges.

The on-chain deploy comes first and is human-dispatched: the
`Manual sol artifacts` workflow runs `script/Deploy.sol` for the `decimal-float`
suite. Where the addresses already exist (deterministic Zoltu), a deploy attests
existing code rather than deploying fresh.

Cutting the release is then a PR that runs
`forge script ./script/Build.sol --sig 'cutRelease()'` + `forge fmt`, freezing
the candidate snapshot into `src/generated/<tag>/` and regenerating the
released-suites libs, with `[external.package].version` naming that tag. After
merge, pushing the `sol-v*` tag runs `rainix-tag-release`
(`package-release.yaml`), which re-runs the non-freezing build, requires a clean
tree with the frozen snapshot byte-identical to the candidate, verifies the live
chains against the record and publishes `rain-math-float-deploy` to Soldeer.

See rainlanguage/rain.factory#46 for the library/deploy split rationale.
