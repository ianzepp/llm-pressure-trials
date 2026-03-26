# Runner

## What Exists Today

The current runner is a real execution backend, not just a design placeholder.

It currently does all of the following:

- loads one study package from `studies/<study>/`
- validates scenario, experiment, variant, condition, and rubric references before execution
- expands one scenario or one experiment into concrete `TrialUnit` values
- resolves models from explicit ids or from the cached OpenRouter catalog
- executes live requests through the OpenRouter-compatible client
- normalizes and deterministically grades each response
- writes run artifacts incrementally
- updates a study-local SQLite index
- computes experiment comparison rows after execution
- emits structured JSONL lifecycle events during live runs

## Current Phases

### 1. Load

The CLI loads a `LoadedStudy` from:

- `scenario.yml`
- `experiments/*.yml`
- `rubrics/*.yml`

inside one study directory.

### 2. Validate

The current validation pass checks:

- non-empty scenario id
- condition references to existing variants
- experiment references to the study scenario
- experiment references to existing conditions and variants
- scenario and experiment references to existing rubrics

Validation happens before provider execution.

### 3. Expand

The runner currently supports two expansion modes:

- one scenario across one or more models
- one experiment across one or more models

Expansion produces `TrialUnit` records carrying:

- study id
- scenario id
- experiment id when present
- condition id and label
- variant id when present
- model id
- output format
- resolved prompt
- repetition index

### 4. Schedule and Execute

Live runs are scheduled with:

- `launch_interval_ms`
- `high_water`
- `low_water`

The current scheduler groups work by model id, launches worker threads, tracks in-flight count, and emits backpressure on/off events.

This is bounded execution. The earlier docs that described bounded parallelism as future work were stale.

What it does not do yet:

- provider-specific key pools
- per-key lease accounting
- replay-mode execution reuse

### 5. Normalize and Grade

The current grading path is intentionally narrow and deterministic.

It currently derives:

- parsed JSON when requested
- inferred decision
- refusal-present flag
- format-valid flag
- compliance level
- harmful specificity
- hedging level
- grading notes

The current implementation is built for the `simple-mercy` and `smoke-latency` studies, not as a general rubric engine yet.

### 6. Persist

Each live run writes into:

`studies/<study>/runs/<run-id>/`

Current artifact files:

- `run.json`
- `trial_units.json`
- `trial_results.jsonl`
- `trial_failures.jsonl` when failures occur
- `comparisons.json`
- `summary.json`

The runner also updates:

`studies/<study>/sqlite.db`

Current SQLite tables:

- `runs`
- `trial_results`
- `comparisons`
- `failed_trials`

The `failed_trials` table may be absent in older study databases created before that column set existed.

### 7. Compare

After execution, experiment comparisons are computed from the stored in-memory trial results and written to both JSON and SQLite.

The current comparison output tracks:

- left and right labels
- optional metric
- left and right decisions
- decision shift
- refusal shift
- hedging delta
- specificity delta

## Event Stream

With `ethics-trials run --jsonl`, the runner currently emits structured events including:

- `run_started`
- `trial_launched`
- `trial_completed`
- `trial_failed`
- `backpressure_on`
- `backpressure_off`
- `run_completed`

## Error Model

Current behavior:

- validation and configuration errors fail the command immediately
- per-trial provider failures can either stop the run or be recorded and skipped, depending on `--continue-on-error`
- scheduler channel breakage is treated as a run error
- artifact and SQLite write failures stop the run

## Study Scope

The runner is firmly study-scoped today.

It expects each study to own:

- one `scenario.yml`
- zero or more experiment definitions
- zero or more rubric definitions
- one `sqlite.db`
- one `runs/` artifact tree

That study-local shape is part of the implementation, not just a preference.

## Not Implemented Yet

The runner does not yet support:

- replay
- TUI integration
- richer rubric interpretation
- provider-aware key leasing
- family-level or cross-study reporting
- cost accounting
- explicit request artifact capture
