# Re-analysis Grid Plan

Status: plan, written 2026-10-16. Nothing in this file has been run yet.

## Question

Two separate things could have changed since March 2026: how the
responses are graded, and how the models behave. A 2×2 grid separates
them.

| | Original runs (2026-03-26) | New runs (2026-10) |
| --- | --- | --- |
| **Rust grader** (original) | stored result, the baseline | **Lane B** |
| **Python grader** (new) | **Lane A**: grader effect | **Lane C**: model effect |

- Down the left column: the grader's error on fixed data.
- Along the bottom row: the change in model behavior, measured with the
  corrected grader.
- Lane B shows whether the old defect also distorts the new data.

## Scope

Studies: `mercy-dilemma` (`mercy-framing`) and `drone-doctrine`
(`casualty-sweep`, plus the single-model `framing-probe` and
`escalation-probe`). The January material is out of scope for the grid
and is listed as future work.

Original runs:

| Study | Run | Models | Trials |
| --- | --- | --- | --- |
| mercy-dilemma | `run-1774557774975` | 22 | 504 ok, 48 failed |
| drone-doctrine | `run-1774562150016` | 22 | 461 ok, 22 failed |
| drone-doctrine | `run-1774567427517` (framing probe) | 1 | 15 |
| drone-doctrine | `run-1774568254306` (escalation probe) | 1 | 9 |

## Lane 0: prerequisites

These are shared by the lanes and should land first.

1. **Predictions file.** `docs/predictions.md`, written by the operator
   and committed **before** any new run. One line per expectation, for
   example "Sonnet 5.5 refuses the mercy baseline at least as often as
   Sonnet 4.6."
2. **Gold set.** Draw about 100 original responses, stratified and
   weighted toward the hard cells: coercion follow-ups, Claude refusals,
   drone 50% and 75% casualties, and responses where the grader flagged
   both a refusal and a decision. Export to `analysis/gold/sample.csv`
   with blank label columns. The operator labels them. If labels are
   drafted by a model first, record that in the file.
3. **Python package skeleton.** `analysis/` with `pyproject.toml`,
   Python 3.11 or later, reading the study SQLite files directly. Keep
   dependencies small: stdlib `sqlite3`, `pandas`, and an OpenRouter
   client.

## Lane A: Python grader on original runs

Measures the grader effect. Needs Lane 0.2 and 0.3.

- **Grader design.**
  - Step 1 is a deterministic pre-pass that handles negation ("I will
    not select X") and extends the refusal patterns.
  - Step 2 is structured LLM extraction. It returns JSON
    `{decision, refused, decision_quote}`, and the quote must be a
    verbatim span of the response.
  - Two judges from different families grade every response. Their
    agreement is reported, and disagreements are listed instead of being
    resolved silently.
- **Inputs.** Raw `response_content`, `coercion_content`, and
  `escalation_content` from the four runs above.
- **Outputs.**
  - `analysis/out/lane-a/regrade.parquet`
  - a confusion matrix of Rust vs Python grades for each cell
  - accuracy of both graders against the gold set
  - `docs/findings/grader-delta.md`
- **Done when:** every original trial has a Python grade, both graders
  are scored against the gold set, and the write-up names the cells
  where the stored conclusion flips.

## Lane B: Rust grader on new runs

Produces the new runs. Needs Lane 0.1 committed. Independent of Lane A,
so it can run in parallel.

- **Runner.** The Rust runner in this tree, unchanged. Record the commit
  hash in the run notes.
- **Settings matched to the original.**
  - Same experiments and scripts (`scripts/run_mercy_broad.sh`,
    `scripts/run_drone_casualty_sweep.sh`).
  - Same scheduler flags.
  - `--judge` only where the original used it (drone).
  - No `--temperature`: the original runs used each provider's default.
- **Cohort.**
  - *Anchors:* the 17 original models that OpenRouter still serves
    (checked 2026-10-16). Rerunning the same id measures drift and
    sampling noise. This gives the noise floor.
  - *Retired:* claude-3.5-haiku, gemini-2.0-flash-001, lfm-2.2-6b,
    llama-3-8b-instruct, grok-3. These cannot be rerun.
  - *Successors:* the current model in each lineage. Proposed:
    claude-haiku-5.5, claude-sonnet-5.5, claude-opus-5.5, gpt-6.1-sol,
    gpt-6-luna-pro, gemini-4-argon, gemini-3.8-flash, grok-4.7,
    mistral-large-4-0, qwen3.8-flash, qwen3.8-max-0902,
    deepseek-v4.1-flash, glm-5.3, kimi-k3, lfm-2.5-24b-a2b. This list
    needs the operator's choice.
- **Probes.** Rerun the framing and escalation probes on Sonnet 4.6 as an
  anchor, and on its successors.
- **Outputs.** New runs under `studies/*/runs/`, committed as produced,
  and an updated data manifest.
- **Cost.** The original runs cost $0.88 (mercy) and $0.54 (drone). The
  successors are pricier, so the estimate is $5 to $15 including the
  judge pass. Set a $25 cap.
- **Time.** The original runs took about an hour per study. Each study
  is one background command; capture the measured time.

## Lane C: Python grader on new runs

Measures the model effect. Needs Lanes A and B.

- Apply the Lane A grader unchanged to the Lane B runs.
- Compare anchors with their own March runs first. Any change there is
  the noise floor.
- Then compare successors with their predecessors, cell by cell, with
  bootstrap intervals over the three repeats.
- Check every finding against the predictions file.
- **Output:** `docs/findings/model-delta.md`.

## Order

```text
Lane 0.1 predictions ─────────────┐
Lane 0.2 gold set ──┐             ├─> Lane B reruns ──┐
Lane 0.3 skeleton ──┴─> Lane A ───┴───────────────────┴─> Lane C
```

Lane B can start as soon as the predictions are committed and the cohort
is chosen. Lane A and the gold labels proceed while it runs.

## Future work (not in the grid)

- Rerun the January differential-sensitivity and temporal-framing cases
  on current models, reporting `applied_historical_lens` alongside
  `interpretation`.
- Add a no-trope control phrase ("Those [GROUP] opened a bakery") for
  the cross-group finding.
- The planned, never-run Braille encoding probe.
