<!--Reference: https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html-->
# Architecture

## Overview

The team's agent skills, published from one repo, each versioned on its own, and installable by any agent
that reads the [Agent Skills](https://agentskills.io/specification) format: Claude Code, Codex, Kimi Code,
opencode, DeepSeek through any of those.

```
 skills/<name>/SKILL.md  ── metadata.version ──► tag <name>--v<version>
                                                      │
          ┌───────────────────────────────────────────┼─────────────────────────────┐
          ▼                                           ▼                             ▼
 consumer's requirement (cargo syntax)       Claude plugin marketplace        gh skill / npx skills
 "1.1" · "^1.1" · "~1.1.0" · "=1.1.0"          (other plugins' deps,           (exact tag)
          │  resolve: highest tag that matches   npm ranges)
          ▼
 lock: name · version · commit
          │  install
          ▼
 ~/.claude/skills/<name>   Claude Code, DeepSeek via Claude Code, opencode
 ~/.agents/skills/<name>   Codex, Kimi Code, opencode, gh skill
```

Background study, with sources: [valeratrades/nix#23](https://github.com/valeratrades/nix/issues/23).

## Layout

```
skills/<name>/            one published unit
├── SKILL.md              entry; frontmatter `name` = directory name
└── …                     whatever SKILL.md links, relatively
docs/refs/                third-party skills we take ideas from; never published, see its README
```

## Codemap

`skargo`, the binary at the repo root: `check` for this (or any) skill repo, `lock`/`install` for consumers.

- `src/check.rs`: validates every tracked `SKILL.md`, lists unreleased tags, owns the `plugins` array of
  `.claude-plugin/marketplace.json` (the rest of that file is hand-written).
- `src/lock.rs`: `skargo.toml` (`name = { git, version }`) → `skargo.lock` (`name · git · path · version · commit`).
  The skill's path in its repo is derived from the frontmatter `name` at the resolved tag.
- `src/install.rs`: store in `$XDG_DATA_HOME/skargo`, directory symlinks from both agent dirs; touches only its own links.
- `nix/home-manager.nix`: `homeManagerModules.default`, the same layout as `install`, built from a lock.
- `.github/workflows/publish.yml`: `check` on PRs; on main, also tags what `check` reports unreleased.

## Invariants

- **A skill's version lives in exactly one place**: `metadata.version` in its `SKILL.md` frontmatter. Every
  other place a version appears (tags, plugin manifests, locks) is derived from it.
- **A released tag is immutable.** `<name>--v<version>` is the naming Claude Code's plugin dependency resolver
  reads, so both resolvers see one history. `skargo` refuses a moved or deleted tag it has locked, but only
  the forge can stop one from moving.

Every other rule on skill content (spec-only frontmatter, one `SKILL.md` per skill, links staying inside the
skill, no content change without a bump, versions only growing) is `skargo check`; Cargo requirement
semantics are pinned by `tests/skargo.rs`.
