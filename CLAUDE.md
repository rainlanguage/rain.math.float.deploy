# CLAUDE.md

Only what a capable agent would get wrong from this repo alone. Layout, dev
shells, build/test commands and dependency lists are discoverable and
deliberately absent (rainlanguage/rainix#298).

## What this repo is

The **deploy half** of `rain.math.float`: the concrete `DecimalFloat`, its
generated deploy records and the deploy scripts + tests. The pure math
(`LibDecimalFloat*`, the errors) is NOT here — it arrives as the
`rain-math-float` Soldeer package.

## Conventions an agent would get wrong

- `DecimalFloat` alone compiles `via_ir` at **399 runs** (a
  `compilation_restrictions` profile): the highest count keeping it 256 bytes
  under EIP-170. Everything else stays legacy at 1,000,000 runs; rain-deploy
  cannot compile `via_ir`. Deterministic (Zoltu) deploy: the address is a pure
  function of the creation bytecode, so this, solc `=0.8.25`,
  `evm_version = "cancun"`, no CBOR metadata, and the pinned `rain-math-float`
  sources all move the pins if changed.
- Pragma: concretes, scripts and tests pin `=0.8.25`; shipped libs and generated
  files float `^0.8.25`.
- All source files carry SPDX headers (LicenseRef-DCL-1.0).
- No skipped tests. Comments describe current behaviour only.
- `recursive_deps` is off: every package an import resolves through is declared
  in `foundry.toml`, including ones reached only via `rain-math-float`.

## Deploy-pin invariants (the hazards)

- `src/generated/candidate/` is the rolling record, rewritten by
  `script/Build.sol` and currency-checked by CI;
  `src/lib/deploy/LibDecimalFloatDeploy.sol` aliases it. `src/generated/<tag>/`
  snapshots are frozen: a release only ADDS one. Never hand-edit generated files
  (`src/generated/`, `src/lib/Lib*Released*.sol`).
- `[external.package].version` moves only in a release cut, in lockstep with the
  frozen `src/generated/<tag>/` snapshot; a normal PR never bumps it.
- `rain-math-float` >= 0.2.45 has no log tables. `DecimalFloat` depends on
  nothing; the `log-tables@<tag>` suites exist only as frozen releases. Never
  add a log-tables candidate.

## Release / deploy shape

- The on-chain deploy is a human-dispatched `Manual sol artifacts` run, BEFORE
  tagging, never on merge: `decimal-float`.
- A manual `sol-v*` tag is the sole Soldeer release trigger
  (`rainix-tag-release`); the Rust crates + npm wrapper publish on merge instead
  (`crate-npm-release.yaml`). See README.md.
