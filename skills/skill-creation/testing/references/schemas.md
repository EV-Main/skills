# schemas

Field names are what the scripts and viewer read; a renamed field shows as empty or zero, not as an error.

## evals.json

`evals/evals.json` in the skill. `files` are relative to the skill root.

```json
{"skill_name": "pdf", "evals": [
  {"id": 1, "prompt": "…", "expected_output": "…", "files": ["evals/files/a.pdf"], "expectations": ["The output includes X"]}
]}
```

## eval_metadata.json

`eval-<name>/eval_metadata.json`; `eval_id` orders the viewer, `prompt` is what it shows.

```json
{"eval_id": 0, "eval_name": "multi-page-form", "prompt": "…", "assertions": ["…"]}
```

## timing.json

`run-<k>/timing.json`, from the run's completion notification; `aggregate_benchmark` reads tokens only from here.

```json
{"total_tokens": 84852, "duration_ms": 23332, "total_duration_seconds": 23.3}
```

## grading.json

`run-<k>/grading.json`, written by [the grader](../agents/grader.md). `user_notes_summary` becomes the run's
`notes` in `benchmark.json`; `claims` and `eval_feedback` are read by people only.

```json
{
  "expectations": [{"text": "…", "passed": true, "evidence": "quote or observation"}],
  "summary": {"passed": 2, "failed": 1, "total": 3, "pass_rate": 0.67},
  "execution_metrics": {"total_tool_calls": 15, "errors_encountered": 0},
  "claims": [{"claim": "The form has 12 fields", "type": "factual|process|quality", "verified": true, "evidence": "…"}],
  "user_notes_summary": {"uncertainties": [], "needs_review": [], "workarounds": []},
  "eval_feedback": {"suggestions": [{"assertion": "…", "reason": "…"}], "overall": "…"}
}
```

## benchmark.json

Written by `scripts/aggregate_benchmark.py`; by hand only if the script cannot run. `configuration` is the config
directory name; `result` is nested.

```json
{
  "metadata": {"skill_name": "pdf", "timestamp": "…", "evals_run": [1, 2], "runs_per_configuration": 3},
  "runs": [{"eval_id": 1, "eval_name": "…", "configuration": "with_skill", "run_number": 1,
            "result": {"pass_rate": 0.85, "passed": 6, "failed": 1, "total": 7, "time_seconds": 42.5, "tokens": 3800, "tool_calls": 18, "errors": 0},
            "expectations": [{"text": "…", "passed": true, "evidence": "…"}], "notes": []}],
  "run_summary": {
    "with_skill": {"pass_rate": {"mean": 0.85, "stddev": 0.05, "min": 0.8, "max": 0.9}, "time_seconds": {"…": 0}, "tokens": {"…": 0}},
    "without_skill": {"…": {}},
    "delta": {"pass_rate": "+0.50", "time_seconds": "+13.0", "tokens": "+1700"}
  },
  "notes": ["from the analyzer's benchmark pass"]
}
```

## feedback.json

Written by the viewer; empty `feedback` means the reviewer had no complaint.

```json
{"reviews": [{"run_id": "eval-0-with_skill-run-1", "feedback": "…", "timestamp": "…"}], "status": "complete"}
```

## trigger eval set

Input to `scripts/run_loop.py` and `scripts/run_eval.py`.

```json
[{"query": "…", "should_trigger": true}]
```

## comparison.json

Written by [the comparator](../agents/comparator.md). Scores are 1–5 per criterion, `overall_score` 1–10;
`expectation_results` only when expectations were given.

```json
{
  "winner": "A|B|TIE",
  "reasoning": "…",
  "rubric": {"A": {"content": {"correctness": 5}, "structure": {"organization": 4}, "content_score": 4.7, "structure_score": 4.3, "overall_score": 9.0}, "B": {}},
  "output_quality": {"A": {"score": 9, "strengths": [], "weaknesses": []}, "B": {}},
  "expectation_results": {"A": {"passed": 4, "total": 5, "pass_rate": 0.8, "details": [{"text": "…", "passed": true}]}, "B": {}}
}
```

## analysis.json

Written by [the analyzer](../agents/analyzer.md) after a comparison. `category`: instructions, tools, examples,
error_handling, structure or references; `priority`: high (would have changed the outcome), medium, low.

```json
{
  "comparison_summary": {"winner": "A", "winner_skill": "…", "loser_skill": "…", "comparator_reasoning": "…"},
  "winner_strengths": ["…"],
  "loser_weaknesses": ["…"],
  "instruction_following": {"winner": {"score": 9, "issues": []}, "loser": {"score": 6, "issues": []}},
  "improvement_suggestions": [{"priority": "high", "category": "instructions", "suggestion": "…", "expected_impact": "…"}],
  "transcript_insights": {"winner_execution_pattern": "…", "loser_execution_pattern": "…"}
}
```
