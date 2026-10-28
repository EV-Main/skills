---
name: skill-creation
description: "Write, restructure, test or tune an agent skill (a SKILL.md and the files it links). Covers both a skill a repo ships about itself in `docs/skill/`, so agents in other repos can build against its library or framework, and a standalone skill that is its own source of truth: a workflow from the current conversation, a procedure, a domain. Use when asked to make, update, split, review or eval a skill, fix one that does not trigger, or make a project consumable by Claude Code, Codex, Kimi, opencode or other agents."
license: "BlueOak-1.0.0; standalone/ is Apache-2.0, see standalone/LICENSE.txt"
metadata:
  version: "0.1.0"
---

# skill-creation

| making | read |
|---|---|
| a skill a repo ships about itself, routing agents in other repos into its docs and code | [repo/README.md](repo/README.md) |
| a skill that is its own source of truth | [standalone/README.md](standalone/README.md) |

Testing a skill against a baseline, and tuning its description, is in
[standalone/README.md](standalone/README.md) for either.

## Principles

The reader is smarter than us and has less context. A skill gives the context and the reasons. It does not
babysit them on how to execute.

- A file holds only the whys, what we expect an agent to get wrong, and the broad strokes. What the reader can
  infer from what surrounds the task (the repo's docs, a script's `--help`, the input in front of them) is
  linked, not restated.
- Cite paths and ids, never paraphrase them. A moved path breaks loudly, a paraphrase rots silently, and
  duplicated skill logic is worse than duplicated code: nothing fails when two copies drift.
- What a hook, check or script enforces is not mentioned. Work every test run repeats by hand becomes a script
  the skill ships.
- Each idea takes the fewest words that carry it. That is not dropping context: it is dropping justifications,
  emphasis, capitals, and anything that only makes a rule sound convincing. Explain what happens; do not sell it.
- A prohibition carries its mechanism. A rule without one gets reasoned around.
- A file exists only where an agent has a distinct job. No stubs, no shims, nothing kept for the past: a skill
  describes what is, never what changed.
- A failure seen in testing is fixed in general. A fix that only makes the test prompts pass is overfitting, and
  the transcript, not the output, shows what made the agent waste its time.

## Shape

- **Loading** is in three stages: `name` + `description` always, the `SKILL.md` body when the skill fires, every
  other file when a link sends the agent there. The description is the only trigger: it names the literal words
  and identifiers an asker types, not a topic. A skill not firing is the most common failure.
- **Frontmatter** holds spec fields only: `name` (= directory name), `description`, `license`, `compatibility`,
  `metadata`, `allowed-tools` ([spec](https://agentskills.io/specification)). A vendor key is read by one agent
  and rejected by some others, e.g. the Claude API refuses Claude Code's.
- **Self-contained.** A skill installs as one directory, outside whatever repo it came from, so every relative
  link resolves inside it. Anything else is an absolute URL, or a path against a base the skill states.
- **One `SKILL.md`, at the root.** Installers scan recursively, so a nested one is published as a second skill.
  A skill too big for one entry routes instead: `SKILL.md` maps what arrived to the file that handles it, and an
  argument can name the procedure. [EV-Service-Arb/playbook `skills/`](https://github.com/EV-Service-Arb/playbook/tree/main/skills)
  does this at scale: `ingest/` routes by input, `service-arb/sections/` holds topic blocks.
- **Where agents read it**:

  | dir (user scope; project scope drops `~/`) | read by |
  |---|---|
  | `~/.claude/skills/<name>/` | Claude Code, DeepSeek through Claude Code, opencode |
  | `~/.agents/skills/<name>/` | Codex, Kimi Code, opencode, `gh skill` |

- **Versioned** skills carry `metadata.version`, and their repo tags each release `<name>--v<version>`:
  [EV-Main/skills](https://github.com/EV-Main/skills/blob/main/docs/ARCHITECTURE.md).
