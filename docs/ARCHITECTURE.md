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

## Invariants

- **A skill's version lives in exactly one place**: `metadata.version` in its `SKILL.md` frontmatter. Every
  other place a version appears (tags, plugin manifests, locks) is derived from it.
- **A released tag is immutable.** `<name>--v<version>` is the naming Claude Code's plugin dependency resolver
  reads, so both resolvers see one history.
- **Version requirements mean what they mean in Cargo**, bare versions included (`1.1` is `^1.1`).
- **A skill is self-contained.** Each one installs as its own directory at its own version, so every relative
  link resolves inside it. Anything outside — another skill, a repo — is an absolute URL.
- **One `SKILL.md` per skill, at its root.** Installers scan recursively, and a nested one would be published
  as a second skill.
- **Frontmatter carries spec fields only** (`name`, `description`, `license`, `compatibility`, `metadata`,
  `allowed-tools`): one payload, read unchanged by every agent.
