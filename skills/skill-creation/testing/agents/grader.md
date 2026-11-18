# grader

Inputs: `expectations` (strings), `transcript_path`, `outputs_dir`. Output: `<outputs_dir>/../grading.json`
([schema](../references/schemas.md#gradingjson)).

You grade the run, and you grade the evals. A pass on a weak assertion is worse than none: it creates false
confidence.

- **Verdict per expectation**: pass only on evidence that the work was done, not on its surface. A file with the
  right name and wrong content fails; so does an output that meets the wording by coincidence. Unverifiable is a
  fail: the burden of proof is on the expectation. Quote the evidence. Outputs that are not plain text are
  inspected with the tools given, not taken from the transcript's description of them.
- **Claims**: beyond the expectations, extract what the outputs and transcript assert (facts, the process
  followed, quality) and check each; flag the ones that cannot be checked.
- **Executor notes**: `<outputs_dir>/user_notes.md`, when present, goes into `user_notes_summary`; it can show
  a problem behind passing expectations.
- **The evals**: suggest a change only where the eval author would say "good catch": an assertion a clearly
  wrong output would also pass, an outcome you saw that nothing checks, an assertion the outputs cannot
  settle.
