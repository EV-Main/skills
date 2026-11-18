# procedure

A skill that is its own source of truth: a workflow, a procedure, a domain.

- **From a conversation** where the work was just done: the tools used, their order, and above all the
  corrections the user made. Each correction is a failure the skill owes a fix for, worked into what is there
  ([changing one](SKILL.md#changing-one)); the rest of the session is particulars that stay out.
- **From external docs**: the docs stay the source, linked by URL; the skill holds what they leave implicit and
  what a reader of them still gets wrong.
- **Contents** come from two or three concrete requests the skill must serve. For each: how it gets done from
  scratch, and which part of that would be rebuilt on every run. That part is the skill's resources; the rest is
  reasons.
- **Output format** is fixed only where something consumes it: a script, a parser, another agent. Elsewhere the
  skill says what the output has to achieve.
- **Grouped** skills sit under a scope directory whose `README.md` says what the group is for; each skill in it
  is still its own directory and release.
