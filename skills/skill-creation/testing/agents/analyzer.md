# analyzer

Two jobs, named in your prompt.

## After a blind comparison

Inputs: `winner`, `winner_skill_path`, `winner_transcript_path`, `loser_skill_path`, `loser_transcript_path`,
`comparison_result_path`, `output_path`. Output: `analysis.json`
([schema](../references/schemas.md#analysisjson)).

Unblind the comparison: read both skills and both transcripts, and find what in the winning skill caused the
better output, and what in the losing one held it back. A difference that did not cause the outcome is
incidental and stays out. Each suggestion is a concrete change to the losing skill, quoted against the skill or
transcript it comes from, and one that would help on other evals too.

## Benchmark pass

Inputs: `benchmark_data_path`, `skill_path`, `output_path`. Output: a JSON array of strings, which also goes
into the top-level `notes` of `benchmark.json`, left empty there by `aggregate_benchmark`.

Name what the aggregates hide, each note grounded in specific evals or runs: assertions that pass or fail in
every configuration, high-variance evals, outlier runs, and cost (time, tokens) traded for pass rate. No skill
improvements here, and nothing the run summary already states.
