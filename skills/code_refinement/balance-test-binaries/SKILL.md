---
name: balance-test-binaries
description: Cut the number of executables a Rust workspace links for its tests, keeping tests narrowable by name. Use when asked to speed up `cargo test` / `cargo nextest` builds, reduce link time, target/ size or disk writes from test builds, merge or consolidate integration tests, or apply matklad's "delete cargo integration tests" to a repo.
metadata:
  version: "0.1.0"
---

# balance-test-binaries

The cost unit of a test build is the **test target**, not the test function: each one is linked into its own
executable, re-linking every dependency into it. Many targets means many links, a large `target/`, and disk
writes that compete with everything else on the drive; with a build cache on a filesystem without reflinks
(kache on ext4), each linked executable is also copied into the cache, roughly doubling its write. A target is
narrowed by module path just as well as by file, so merging targets loses nothing in finding what failed:
`cargo test --test it core::placement::rejects_overlap`, `cargo nextest run -E 'binary(it) & test(/^core::/)'`.
Background: [matklad, Delete Cargo Integration Tests](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html)
(Cargo's own: test compile 3× faster, artifacts 5× smaller).

## What links a test executable

`cargo test --workspace --no-run` prints one `Executable` line per linked test binary; count them before and
after, and diff `cargo nextest list --workspace` (or `cargo test -- --list`) to show every test still exists.

| target | links when |
|---|---|
| each lib's unit tests | always, unless `[lib] test = false` — even with zero `#[test]` in it |
| each bin's unit tests | always, unless `[[bin]] test = false` |
| each `tests/x.rs` or `tests/x/main.rs` | always |
| examples | compiled by a plain `cargo test`; check the listing for the repo's alias |
| doctests | edition 2024: one binary per crate; older editions: one per doctest |

## Moves, by yield

1. **One integration target per crate.** `tests/*.rs` files become modules of `tests/it/main.rs` (any name),
   keeping their names, so the semantic split survives as `it::<old_file>::<fn>`. Shared helpers become one
   `mod common`, which also ends the `tests/common/mod.rs` trick.
2. **Zero-test targets off**: `test = false` on a lib or bin with no `#[test]`. A test added there later then
   compiles and silently never runs, so this move ships together with a check that fails on `#[test]` inside a
   target marked `test = false` (pre-commit or CI); without that check, leave the target on.
3. **Examples out of test runs** where they need not be linked there: select targets in the repo's test alias
   (`--lib --bins --tests`) rather than relying on the default set.
4. **Across crates**, only for small crates whose tests share fixtures: one workspace test crate with a module
   per crate.

## Where agents over-merge

- **One binary for the whole workspace**: every change to any crate relinks it, and one compile error blocks
  every test. Merging stops at a boundary that changes together.
- **`harness = false` targets** (criterion, trybuild, datatest, a custom `main`) stay separate: their `main` is
  the harness.
- **Process-global state**: `cargo test` runs a binary's tests as threads of one process. Tests that set env
  vars, change cwd, install panic hooks or global subscribers, or need their own `#[tokio::main]` runtime were
  isolated by living in separate binaries; merged, they race. nextest runs each test in its own process and
  hides this, so merged targets must also pass under `cargo test`, or such tests get `serial_test`.
- **`CARGO_BIN_EXE_<name>`** is set only for integration tests of the package that owns the binary; a test that
  runs a bin stays in that package when crates are merged (move 4).
- **Visibility**: integration tests see only the `pub` API, so nothing becomes `pub` to make a move work; a test
  that needs internals was a unit test and goes back into the crate.
