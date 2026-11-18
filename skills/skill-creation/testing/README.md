<!-- Modified from anthropics/claude-plugins-official@2a40fd2e plugins/skill-creator/skills/skill-creator/SKILL.md (Apache-2.0, LICENSE.txt); changes are in this repo's git history. -->
# testing

An author cannot see what their own skill fails to say; an agent without their context can. Paths below are
relative to this directory, and `python -m scripts.<x>` runs from it; every script and the viewer take `--help`.

## Runs

- **Prompts**: two or three requests as a user would type them, with the detail and mess of real ones, in
  `evals/evals.json` ([schema](references/schemas.md#evalsjson)).
- **Baseline**: for a new skill, no skill; for a revision, a copy of the old one taken before editing.
- **Spawn** each prompt's skill and baseline runs in the same turn, as subagents. A run's tokens and duration
  arrive only in its completion notification, so each goes to its `timing.json` as it arrives.
- **Layout**, which the scripts and viewer read:

  ```
  <skill>-workspace/iteration-<n>/eval-<name>/
    eval_metadata.json          prompt, eval_id, assertions
    <config>/run-<k>/           config: with_skill, without_skill or old_skill
      outputs/  timing.json  grading.json
  ```

## Reading them

- The transcript, not only the output: where the agent spent its time and what it got wrong shows what the
  skill owes. A failure is fixed in general; a fix that only makes the test prompts pass is overfitting.
- When a human judges quality, `eval-viewer/generate_review.py <iteration dir>` goes up before your own
  revision, so their read is not anchored on yours. They leave feedback per run into `feedback.json`; empty
  feedback means fine.
- When the question is whether a new version beats the old and there is no number to settle it,
  [agents/comparator.md](agents/comparator.md) judges two outputs blind, and
  [agents/analyzer.md](agents/analyzer.md) then reads both skills and transcripts for why the winner won.

Each revision reruns every prompt into `iteration-<n+1>/`, baselines included, until feedback is empty or
revisions stop moving it.
