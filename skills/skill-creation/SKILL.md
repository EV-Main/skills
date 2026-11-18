---
name: skill-creation
description: Write, restructure, test or tune an agent skill (a SKILL.md and the files it links) — one a repo ships about its own app, library or framework, one over an MCP server's tools, or a standalone procedure or domain. Use when asked to make, update, split, review, eval or benchmark a skill, fix one that does not trigger, or make a project or MCP server consumable by Claude Code, Codex, Kimi, opencode or other agents.
metadata:
  version: "0.3.0-alpha.2"
---

# skill-creation

A skill is two choices: what it is about, and how it is checked. This file holds what applies to every skill;
then read one file per choice.

| about | read |
|---|---|
| an app, library or framework, shipped from its own repo | [repo.md](repo.md) |
| an MCP server's tools | [mcp.md](mcp.md) |
| a procedure or domain the skill itself is the source of truth for | [procedure.md](procedure.md) |

| checked by | read |
|---|---|
| runs with and without the skill, judged by reading | [testing/README.md](testing/README.md) |
| a number: assertions passed, tokens or time spent, trigger rate | the above, then [testing/measured.md](testing/measured.md) |

## The reader

The reader is smarter than us and has none of the context the skill was written in: not the conversation, the
machine, the incident, nor the person who asked. A skill supplies that context and the reasons, and leaves the
execution to the reader.

- Everything stated is read as important. A sentence on what a capable reader would do anyway sends them
  looking for the catch it must hide. A skill holds what such a reader gets wrong without it, and why; what they
  can infer from the task, the repo, or a script's `--help` is linked, not restated.
- A skill speaks about its subject, never to whoever asked for it. Questions raised while writing it, and
  examples from that session, stay out unless an example shows what the statement alone cannot.
- Specificity follows fragility: where many approaches work, give the reasons and let the reader choose; where
  one wrong step breaks things, ship the exact script. Permissive wording where any way is valid is what lets a
  hard rule read as meant.
- Paths and ids are cited, never paraphrased. A moved path breaks loudly, a paraphrase rots silently.
- What a hook, check or script enforces is not mentioned. Work every run repeats by hand becomes a script the
  skill ships.
- Each idea takes the fewest words that carry it, with no justification, emphasis or capitals. A prohibition
  carries its mechanism; a rule without one gets reasoned around.
- A rule lives in one file, at the step where it is acted on, scoped to the case that needs it; others link it.
  A file exists only where an agent has a distinct job, and describes what is, never what changed.

## Changing one

A skill is code with no compiler, and its structure is all that holds it. A new understanding is integrated
from the top down: every part it bears on is rewritten, as far as it bears on it. Lines appended beside the old
ones dilute both. Then cut what it made redundant, and lift a principle that now covers several places to where
it covers them all; edge cases and sections that are already minimal stay.

## Shape

- **Loading** is in three stages: `name` + `description` always, the `SKILL.md` body when the skill fires,
  every other file when a link sends the agent there. The description is the only trigger: it names the literal
  words and identifiers an asker types, not a topic, and no "when to use" goes in the body.
- **Resources** hold what each run would otherwise rebuild: code to `scripts/`, schemas and specs to look up to
  `references/` (a large one gets grep patterns where it is linked), files copied into output to `assets/`.
- **Frontmatter** holds spec fields only: `name` (= directory name), `description`, `license`, `compatibility`,
  `metadata`, `allowed-tools` ([spec](https://agentskills.io/specification)). A vendor key is read by one agent
  and rejected by some others; the Claude API refuses Claude Code's.
- **Self-contained.** A skill installs as one directory, outside whatever repo it came from, so every relative
  link resolves inside it. Anything else is an absolute URL, or a path against a base the skill states.
- **One `SKILL.md`, at the root.** Installers scan recursively, so a nested one is published as a second skill.
  A skill too big for one entry routes instead: `SKILL.md` maps what arrived to the file that handles it.
  [EV-Service-Arb/playbook `skills/`](https://github.com/EV-Service-Arb/playbook/tree/main/skills) does this at
  scale.
- **Where agents read it**:

  | dir (user scope; project scope drops `~/`) | read by |
  |---|---|
  | `~/.claude/skills/<name>/` | Claude Code, DeepSeek through Claude Code, opencode |
  | `~/.agents/skills/<name>/` | Codex, Kimi Code, opencode, `gh skill` |

- **Versioned** skills carry `metadata.version`; their repo's CI tags each release `<name>--v<version>`, and
  [EV-Main/skills](https://github.com/EV-Main/skills/blob/main/docs/ARCHITECTURE.md), the registry, records which
  repo holds which skill.
