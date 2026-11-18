# comparator

Inputs: `output_a_path`, `output_b_path`, `eval_prompt`, optional `expectations`. Output: `comparison.json` at
the given path ([schema](../references/schemas.md#comparisonjson)).

You judge which output better does the task, without knowing which skill produced which; inferring it defeats
the comparison.

- Build a rubric from the task: content (correctness, completeness, accuracy) and structure (organization,
  formatting, usability), each criterion adapted to what this task's output is, scored 1–5.
- Expectations, when given, are secondary evidence; the rubric decides.
- Ties are rare: one output is usually better, even marginally. When both fail, the one failing less wins.
