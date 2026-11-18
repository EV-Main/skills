# measured

When the outcome has a number, the runs in [README.md](README.md) are graded and aggregated, and a revision
counts as better only when the number moves beyond its run-to-run spread.

## Assertions

Each eval's `assertions` are checked by [agents/grader.md](agents/grader.md), or by a script where the check is
mechanical, since a script is reused across iterations. An assertion is worth keeping when a wrong output fails
it; one that passes in every configuration measures nothing.

## Cost

A skill whose point is the same task done cheaper (fewer tokens, less time, fewer tool calls) is measured on
cost, and its assertions only guard that the task still gets done. The spread between identical runs is often as
large as the effect, so each configuration gets several runs (`run-1` … `run-<k>`), compared on mean against
stddev.

## Aggregating

`python -m scripts.aggregate_benchmark <iteration dir> --skill-name <name>` writes `benchmark.json` and
`benchmark.md`: pass rate, time and tokens per configuration, with the delta; the viewer shows it with
`--benchmark`. The benchmark pass in [agents/analyzer.md](agents/analyzer.md) then names what the aggregates
hide: assertions that never discriminate, flaky evals, cost traded for pass rate.

## Trigger rate

The description is measured by how often it fires on queries that should and should not trigger it.

- About twenty queries, half each. Positives phrase the same need differently, some never naming the skill.
  Negatives are near misses that share its keywords but need something else; an unrelated negative tests
  nothing.
- Queries are substantive: agents consult skills only for what they cannot do directly, so a one-step query
  does not trigger whatever the description says.
- [assets/eval_review.html](assets/eval_review.html) lets a person edit the set before the run: its
  `__EVAL_DATA_PLACEHOLDER__`, `__SKILL_NAME_PLACEHOLDER__` and `__SKILL_DESCRIPTION_PLACEHOLDER__` are filled
  in, and its export is the set.
- `python -m scripts.run_loop --eval-set <set> --skill-path <skill> --model <this session's model>` runs each
  query three times, holds out 40%, proposes descriptions from the failures, and returns the
  `best_description` by held-out score. It drives `claude -p` through a temporary `.claude/commands/` entry, so
  it measures Claude Code's triggering only.
