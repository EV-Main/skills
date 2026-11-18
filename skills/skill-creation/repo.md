# repo

A repo that others build against ships `docs/skill/`: one `SKILL.md` plus the `references/` it links. It is a
**router, not a summary**: it says which file in the repo answers which question, and supplies only the
mechanical shape the prose there leaves implicit. Documentation that belongs next to what it covers stays there.

Worked example: <https://github.com/EV-Quant/trading_data/tree/main/docs/skill>

## Install

Installed, the directory is named after the frontmatter `name`, not `skill/`. It releases like any skill:
`metadata.version`, and `skills = { };` in the repo's v_flakes `github { … }`, whose CI tags `<name>--v<version>`
on main. A `<name> = { git, version }` line in the registry's
[`skargo.toml`](https://github.com/EV-Main/skills/blob/main/skargo.toml) lets consumers write `<name> = "<req>"`;
whoever edits the skill links their checkout live: `<name> = { path = "<checkout>/docs/skill" }`.

## Paths resolve against a stated base

Installed, the directory sits outside the repo, so `../ARCHITECTURE.md` resolves to nothing. State the base once,
cheapest first, then write every repo path against it:

1. a local checkout, found mechanically: e.g. `cargo metadata --format-version 1 | jq -r
   '.packages[]|select(.name=="<crate>").manifest_path'`, then the directory above it;
2. `https://github.com/<org>/<repo>/blob/main/<path>` (raw: `raw.githubusercontent.com/…/main/…`).

Links within `docs/skill/` stay relative: the tree moves as a unit.

## Sections

In order; one that would be empty is dropped.

1. **Hook**: three sentences on what the thing is, in its own vocabulary, then the one boundary a consumer
   breaks first, with the spec id that forbids it.
2. **Resolve the sources first**: the base above, then the table, the primary artifact:

   | path | what it is | read it |
   |---|---|---|
   | `docs/ARCHITECTURE.md` | the claim everything rests on | **always, first** |
   | `<crate>/model.typ` | the mechanism | **before writing a node** |
   | `docs/spec/` | the normative requirements | before trading anything off |
   | `src/lib.rs` | the entire public vocabulary | when an import will not resolve |
   | `examples/simple/` | the fastest complete reading | first contact |

   The third column makes it a router; a path with no distinct trigger does not belong. The section closes
   with: *nothing in this skill restates those.*
3. **What you actually author**: the smallest complete declaration that compiles, and what is derived and
   therefore absent from it.
4. **The loop**: numbered, each step ending in a `references/` link. The last step is the build, where whatever
   the project enforces mechanically is checked.
5. **Rules that get broken**: each enforced already, listed because knowing it first saves a rewrite. One bold
   clause, two lines of why, the spec id in parentheses.
6. **References**: what each `references/*.md` covers. A section moves there once it outgrows a screen.
7. **Verify**: commands that run from the workspace root, then the project's kind (app / helper lib /
   framework) and the judgement that follows from it.

## Testing

The baseline is an agent in a repo that has never seen the source tree, with the `path` requirement installed
(`skargo install`), asked for the task the skill exists for. Every path it fails to resolve is a base stated
wrong. Loop: [testing/README.md](testing/README.md).
