---
name: balance-test-binaries
description: Cut the number of executables a Rust workspace links for its tests, keeping tests narrowable by name. Use when asked to speed up `cargo test` / `cargo nextest` builds, reduce link time, target/ size or disk writes from test builds, merge or consolidate integration tests, or apply matklad's "delete cargo integration tests" to a repo.
metadata:
  version: "0.1.1"
---

# balance-test-binaries

Every test target links into its own executable, re-linking all its dependencies. Test build time, `target/`
size and disk writes scale with the number of targets, not of tests. The aim is fewer targets, each still one
coherent unit; tests inside a target stay addressable by module path.
Background: [matklad, Delete Cargo Integration Tests](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html).

## Measure

`cargo test --workspace --no-run` prints one `Executable` line per target. Count before and after, and diff
`cargo nextest list --workspace` to confirm the set of tests is unchanged.

| target | linked when |
|---|---|
| a lib's unit tests | always, unless `[lib] test = false`, even with zero `#[test]` in it |
| a bin's unit tests | always, unless `[[bin]] test = false` |
| each `tests/x.rs` or `tests/x/main.rs` | always |
| examples | built by a plain `cargo test` |
| doctests | edition 2024: one per crate; older editions: one per doctest |

## Moves, by yield

1. **One integration target per crate**: each `tests/*.rs` becomes a module of the same name under one
   `tests/<name>/main.rs`.
2. **`test = false` on targets with no tests.** A test added there later compiles and never runs, so this ships
   with a check failing on `#[test]` inside such a target; without the check, leave the target on.
3. **Examples out of test runs**, by selecting targets in the repo's test alias (`--lib --bins --tests`).
4. **One test crate for several small crates** that share fixtures, a module per crate.

## Limits

- Merge only along boundaries that change together: one target for the whole workspace relinks on any change,
  and one compile error blocks every test.
- `harness = false` targets (criterion, trybuild, a custom `main`) stay separate.
- `cargo test` runs a target's tests as threads of one process; nextest gives each its own. Tests that set env
  vars, cwd, panic hooks or global subscribers were isolated by separate binaries and race once merged, under
  `cargo test` only. Merged targets pass under both, or those tests get `serial_test`.
- Under move 4, tests that run a binary of another package lose `CARGO_BIN_EXE_<name>`, which exists only in the
  owning package, and `cargo test` does not build that binary. They locate it in the profile dir next to their
  own executable, with an explicit build of it before the run (e.g. a nextest setup script).
